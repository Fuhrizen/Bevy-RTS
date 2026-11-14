use crate::operation::{RESERVE_COST, RESERVE_UNLOCK, RESOLVE, influence, territory};
use crate::{
    model::*,
    operation::{CONTROL_TARGET, DEPOTS, Finish, Mode, TIMED_LIMIT, in_cover, leader},
};
use bevy::prelude::*;
use std::collections::{BTreeSet, VecDeque};

pub struct BattlePlugin;
impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Battle), setup_battle)
            .add_systems(
                FixedUpdate,
                (crate::ai::direct_red, simulate)
                    .chain()
                    .run_if(in_state(Screen::Battle)),
            );
    }
}

fn make_squad(team: Team, index: usize, callsign: String, kit: Kit, manpower: u8) -> Squad {
    let soldiers = (0..manpower.max(1))
        .map(|id| Soldier {
            id,
            hp: 100.0,
            role: if id == 0 {
                Role::Leader
            } else if id == 1 && matches!(kit, Kit::Support) {
                Role::Support
            } else if id == 1 && matches!(kit, Kit::Medic) {
                Role::Medic
            } else {
                Role::Rifle
            },
            veterancy: 0,
            cooldown: 0.0,
        })
        .collect();
    Squad {
        team,
        pos: team.spawn() + Vec2::new(0.0, (index as f32 - 1.5) * 55.0),
        callsign,
        kit,
        soldiers,
        supplies: 100.0,
        orders: VecDeque::new(),
        entrenched: false,
        hold_fire: false,
        cautious_move: false,
        suppression: 0.0,
        visible_to_blue: team == Team::Blue,
        last_contact: None,
        smoke: 0.0,
        focus: None,
        ai: AiState::Idle,
    }
}

pub(crate) fn setup_battle(mut battle: ResMut<Battle>, roster: Res<Roster>) {
    *battle = new_battle(&roster);
}

fn new_battle(roster: &Roster) -> Battle {
    let mut squads = Vec::with_capacity(8);
    for (i, config) in roster.squads.iter().enumerate() {
        squads.push(make_squad(
            Team::Blue,
            i,
            config.callsign.clone(),
            config.kit,
            config.manpower,
        ));
    }
    for i in 0..4 {
        let kit = match i {
            1 => Kit::Support,
            2 => Kit::Scout,
            3 => Kit::Medic,
            _ => Kit::Rifle,
        };
        squads.push(make_squad(Team::Red, i, format!("Red {}", i + 1), kit, 6));
    }
    let objectives = [
        "Northwest",
        "North",
        "Northeast",
        "Southwest",
        "South",
        "Southeast",
    ]
    .into_iter()
    .zip(roster.map.points())
    .map(|(name, pos)| Objective {
        name,
        pos,
        owner: None,
        capture: 0.0,
        contested: false,
    })
    .collect();
    Battle {
        squads,
        objectives,
        winner: None,
        draw: false,
        elapsed: 0.0,
        effects: Vec::new(),
        map: roster.map,
        mode: roster.mode,
        score: [0.0; 2],
        resolve: [RESOLVE; 2],
        requisition: [30.0; 2],
        reserve_used: [false; 2],
        reserve_kit: Some(roster.reserve_kit),
        finish: None,
        volley_count: 0,
        order_count: 0,
        paused: false,
        pace: Pace::default(),
        resume_pace: Pace::default(),
        selected: BTreeSet::new(),
    }
}

/// Replace or append movement orders for the selected blue squads.
pub fn issue_move(battle: &mut Battle, destination: Vec2, append: bool) {
    let destination =
        destination.clamp(-MAP_HALF + Vec2::splat(20.0), MAP_HALF - Vec2::splat(20.0));
    let living: Vec<_> = battle
        .selected
        .iter()
        .copied()
        .filter(|&i| {
            battle
                .squads
                .get(i)
                .is_some_and(|s| s.team == Team::Blue && s.alive())
        })
        .collect();
    let group = living.len() > 1;
    for (slot, i) in living.into_iter().enumerate() {
        let squad = &mut battle.squads[i];
        if !append {
            squad.cautious_move = false;
            squad.orders.clear();
        }
        let offset = if group {
            Vec2::new((slot % 2) as f32 - 0.5, (slot / 2) as f32 - 0.5) * 38.0
        } else {
            Vec2::ZERO
        };
        squad
            .orders
            .push_back((destination + offset).clamp(-MAP_HALF, MAP_HALF));
        squad.entrenched = false;
        squad.focus = None;
    }
}

pub fn toggle_entrench(battle: &mut Battle) {
    for &i in &battle.selected {
        if let Some(squad) = battle.squads.get_mut(i)
            && squad.team == Team::Blue
            && squad.alive()
        {
            squad.entrenched = !squad.entrenched;
            if squad.entrenched {
                squad.orders.clear();
            }
        }
    }
}

pub fn toggle_fire_control(battle: &mut Battle) {
    let hold = battle.selected.iter().any(|&i| {
        battle
            .squads
            .get(i)
            .is_some_and(|squad| squad.team == Team::Blue && squad.alive() && !squad.hold_fire)
    });
    for &i in &battle.selected {
        if let Some(squad) = battle.squads.get_mut(i)
            && squad.team == Team::Blue
            && squad.alive()
        {
            squad.hold_fire = hold;
        }
    }
}

pub fn deploy_smoke(battle: &mut Battle) {
    for &i in &battle.selected {
        if let Some(squad) = battle.squads.get_mut(i)
            && squad.team == Team::Blue
            && squad.alive()
            && squad.supplies >= 20.0
            && squad.smoke <= 0.0
        {
            squad.supplies -= 20.0;
            squad.smoke = 12.0;
        }
    }
}

/// Route selected squads to their closest usable supply source.
pub fn resupply_selected(battle: &mut Battle) {
    for &i in &battle.selected {
        if let Some(squad) = battle.squads.get_mut(i)
            && squad.team == Team::Blue
            && squad.alive()
        {
            let destination = supply_destination(&battle.objectives, squad.team, squad.pos);
            squad.cautious_move = false;
            squad.orders.clear();
            squad.focus = None;
            if squad.pos.distance(destination) > SUPPLY_RADIUS {
                squad.orders.push_back(destination);
                squad.entrenched = false;
            }
        }
    }
}

pub(crate) fn supply_destination(objectives: &[Objective], team: Team, point: Vec2) -> Vec2 {
    DEPOTS
        .into_iter()
        .filter_map(|index| objectives.get(index))
        .filter(|objective| objective.owner == Some(team) && !objective.contested)
        .map(|objective| objective.pos)
        .chain(std::iter::once(team.spawn()))
        .min_by(|a, b| {
            point
                .distance_squared(*a)
                .total_cmp(&point.distance_squared(*b))
        })
        .unwrap_or_else(|| team.spawn())
}

fn simulate(time: Res<Time<Fixed>>, settings: Res<Settings>, mut battle: ResMut<Battle>) {
    let dt = time.delta_secs() * battle.pace.factor();
    if dt > 0.0 {
        step_battle(&mut battle, settings.difficulty, dt);
    }
}

fn step_battle(battle: &mut Battle, difficulty: Difficulty, dt: f32) {
    if battle.paused || battle.winner.is_some() || battle.draw {
        return;
    }
    let previous_elapsed = battle.elapsed;
    battle.elapsed += dt;
    battle.effects.retain_mut(|effect| {
        effect.ttl -= dt;
        effect.ttl > 0.0
    });
    let halts: Vec<bool> = battle
        .squads
        .iter()
        .map(|squad| {
            squad.cautious_move
                && !squad.hold_fire
                && squad.supplies >= 0.45
                && battle.squads.iter().any(|target| {
                    target.team != squad.team
                        && target.pos.distance(squad.pos) <= 220.0
                        && crate::contact::can_see(battle.map, squad, target, &battle.squads)
                })
        })
        .collect();
    for (squad, halt) in battle.squads.iter_mut().zip(halts) {
        squad.suppression = (squad.suppression - 0.13 * dt).max(0.0);
        squad.smoke = (squad.smoke - dt).max(0.0);
        for soldier in &mut squad.soldiers {
            soldier.cooldown = (soldier.cooldown - dt).max(0.0);
        }
        if !squad.alive() {
            squad.orders.clear();
            continue;
        }
        if !halt && let Some(&destination) = squad.orders.front() {
            let delta = destination - squad.pos;
            let speed = if matches!(squad.kit, Kit::Scout) {
                100.0
            } else {
                80.0
            };
            let speed = speed * (1.0 - 0.60 * squad.suppression);
            if delta.length() <= speed * dt {
                squad.pos = destination;
                squad.orders.pop_front();
            } else {
                squad.pos += delta.normalize_or_zero() * speed * dt;
            }
        }
        let destination = supply_destination(&battle.objectives, squad.team, squad.pos);
        if squad.pos.distance(destination) <= SUPPLY_RADIUS {
            let at_base = destination == squad.team.spawn();
            squad.supplies = (squad.supplies + (if at_base { 8.0 } else { 4.0 }) * dt).min(100.0);
            for soldier in &mut squad.soldiers {
                if at_base && soldier.hp > 0.0 {
                    soldier.hp = (soldier.hp + 3.0 * dt).min(100.0);
                }
            }
        }
    }
    let contacts: Vec<_> = battle
        .squads
        .iter()
        .map(|squad| {
            squad.team == Team::Blue
                || crate::contact::spotted(battle.map, &battle.squads, Team::Blue, squad)
        })
        .collect();
    for (squad, visible) in battle.squads.iter_mut().zip(contacts) {
        squad.visible_to_blue = visible;
        if squad.team == Team::Red {
            if visible {
                squad.last_contact = Some((squad.pos, battle.elapsed));
            } else if squad
                .last_contact
                .is_some_and(|(_, time)| battle.elapsed - time > 15.0)
            {
                squad.last_contact = None;
            }
        }
    }
    // Resolve fire from a snapshot so both sides get a turn in the same frame.
    let mut volleys = Vec::new();
    for i in 0..battle.squads.len() {
        let attacker = &battle.squads[i];
        if !attacker.alive() || attacker.hold_fire || attacker.supplies < 0.45 {
            continue;
        }
        let range = if matches!(attacker.kit, Kit::Scout) {
            255.0
        } else {
            220.0
        };
        let target = attacker
            .focus
            .filter(|&j| {
                battle.squads.get(j).is_some_and(|s| {
                    s.team != attacker.team
                        && s.pos.distance(attacker.pos) <= range
                        && crate::contact::can_see(battle.map, attacker, s, &battle.squads)
                })
            })
            .or_else(|| {
                battle
                    .squads
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| {
                        s.team != attacker.team
                            && s.pos.distance(attacker.pos) <= range
                            && crate::contact::can_see(battle.map, attacker, s, &battle.squads)
                    })
                    .min_by(|(_, a), (_, b)| {
                        a.pos
                            .distance_squared(attacker.pos)
                            .total_cmp(&b.pos.distance_squared(attacker.pos))
                    })
                    .map(|(j, _)| j)
            });
        if let Some(j) = target {
            let terrain = if in_cover(battle.map, battle.squads[j].pos) {
                0.75
            } else {
                1.0
            };
            let defense = terrain
                * (if battle.squads[j].entrenched {
                    0.65
                } else {
                    1.0
                })
                * (if battle.squads[j].smoke > 0.0 {
                    0.55
                } else {
                    1.0
                });
            let ready = attacker
                .soldiers
                .iter()
                .filter(|s| s.hp > 0.0 && s.cooldown <= 0.0)
                .count()
                .min((attacker.supplies / 0.45).floor() as usize);
            let difficulty = if attacker.team == Team::Red {
                match difficulty {
                    Difficulty::Recruit => 0.7,
                    Difficulty::Standard => 1.0,
                    Difficulty::Veteran => 1.3,
                }
            } else {
                1.0
            };
            if ready > 0 {
                volleys.push((
                    i,
                    j,
                    ready,
                    ready as f32 * 7.0 * defense * difficulty * (1.0 - 0.65 * attacker.suppression),
                ));
            }
        }
    }
    for &(attacker, target, ready, _) in &volleys {
        if battle.squads[attacker].visible_to_blue && battle.squads[target].visible_to_blue {
            battle.effects.push(VolleyEffect {
                from: battle.squads[attacker].pos,
                to: battle.squads[target].pos,
                team: battle.squads[attacker].team,
                ttl: 0.24,
            });
            battle.volley_count += 1;
        }
        let squad = &mut battle.squads[attacker];
        let cooldown = if squad.kit == Kit::Support { 0.85 } else { 1.2 };
        for soldier in squad
            .soldiers
            .iter_mut()
            .filter(|s| s.hp > 0.0 && s.cooldown <= 0.0)
            .take(ready)
        {
            soldier.cooldown = cooldown;
            soldier.veterancy = soldier.veterancy.saturating_add(1).min(100);
            squad.supplies = (squad.supplies - 0.45).max(0.0);
        }
    }
    for (_, j, ready, mut damage) in volleys {
        let squad = &mut battle.squads[j];
        squad.suppression = (squad.suppression + ready as f32 * 0.025).min(1.0);
        for soldier in &mut squad.soldiers {
            let dealt = damage.min(soldier.hp);
            soldier.hp -= dealt;
            damage -= dealt;
            if damage <= 0.0 {
                break;
            }
        }
    }
    // Medics restore health to living soldiers in nearby friendly squads.
    let healers: Vec<_> = battle
        .squads
        .iter()
        .filter(|s| {
            s.soldiers
                .iter()
                .any(|soldier| soldier.role == Role::Medic && soldier.hp > 0.0)
        })
        .map(|s| (s.team, s.pos))
        .collect();
    for squad in &mut battle.squads {
        if healers
            .iter()
            .any(|(team, pos)| *team == squad.team && pos.distance(squad.pos) <= 130.0)
        {
            for soldier in &mut squad.soldiers {
                if soldier.hp > 0.0 {
                    soldier.hp = (soldier.hp + 1.5 * dt).min(100.0);
                }
            }
        }
    }
    battle
        .selected
        .retain(|&i| battle.squads.get(i).is_some_and(Squad::alive));
    if battle.mode == Mode::Frontline {
        update_frontline(battle, dt);
    } else {
        let presence: Vec<_> = battle
            .objectives
            .iter()
            .map(|objective| {
                let blue = battle.squads.iter().any(|s| {
                    s.team == Team::Blue
                        && s.alive()
                        && s.pos.distance(objective.pos) <= CAPTURE_RADIUS
                });
                let red = battle.squads.iter().any(|s| {
                    s.team == Team::Red
                        && s.alive()
                        && s.pos.distance(objective.pos) <= CAPTURE_RADIUS
                });
                (blue, red)
            })
            .collect();
        for (objective, (blue, red)) in battle.objectives.iter_mut().zip(presence) {
            objective.contested = blue && red;
            if blue != red {
                objective.capture = (objective.capture + if blue { dt / 10.0 } else { -dt / 10.0 })
                    .clamp(-1.0, 1.0);
            }
            if objective.capture >= 1.0 {
                objective.owner = Some(Team::Blue);
            } else if objective.capture <= -1.0 {
                objective.owner = Some(Team::Red);
            } else if objective.owner == Some(Team::Blue) && objective.capture <= 0.0
                || objective.owner == Some(Team::Red) && objective.capture >= 0.0
            {
                objective.owner = None;
            }
        }
    }
    resolve_operation(battle, dt, previous_elapsed);
}

pub fn call_reserve(battle: &mut Battle, team: Team) -> bool {
    let side = usize::from(team == Team::Red);
    if battle.mode != Mode::Frontline
        || battle.elapsed < RESERVE_UNLOCK
        || battle.requisition[side] < RESERVE_COST
        || battle.reserve_used[side]
        || battle.winner.is_some()
        || battle.draw
        || battle.paused
    {
        return false;
    }
    battle.requisition[side] -= RESERVE_COST;
    battle.reserve_used[side] = true;
    let kit = if team == Team::Blue {
        battle.reserve_kit.unwrap_or(Kit::Support)
    } else {
        Kit::Support
    };
    battle.squads.push(make_squad(
        team,
        2,
        format!("{} Reserve", team.label()),
        kit,
        6,
    ));
    true
}

fn update_frontline(battle: &mut Battle, dt: f32) {
    let field: Vec<_> = influence(battle.squads.iter()).collect();
    for objective in &mut battle.objectives {
        let (owner, contested) = territory(objective.pos, &field);
        objective.owner = owner;
        objective.contested = contested;
        objective.capture = match owner {
            Some(Team::Blue) => 1.0,
            Some(Team::Red) => -1.0,
            None => 0.0,
        };
    }
    for points in &mut battle.requisition {
        *points = (*points + dt).min(100.0);
    }
    call_reserve(battle, Team::Red);
}

fn resolve_operation(battle: &mut Battle, dt: f32, previous_elapsed: f32) {
    let active = [Team::Blue, Team::Red].map(|team| {
        battle
            .squads
            .iter()
            .any(|squad| squad.team == team && squad.alive())
    });
    let owned = [Team::Blue, Team::Red].map(|team| {
        battle
            .objectives
            .iter()
            .filter(|point| point.owner == Some(team))
            .count()
    });
    let scored_dt = if battle.mode == Mode::Timed {
        dt.min((TIMED_LIMIT - previous_elapsed).max(0.0))
    } else {
        dt
    };
    if matches!(battle.mode, Mode::Control | Mode::Timed) {
        for point in battle.objectives.iter().filter(|point| !point.contested) {
            match point.owner {
                Some(Team::Blue) => battle.score[0] += scored_dt,
                Some(Team::Red) => battle.score[1] += scored_dt,
                None => {}
            }
        }
    }
    if battle.mode == Mode::Frontline {
        let majority = owned[0] as i32 - owned[1] as i32;
        if majority != 0 {
            let losing = usize::from(majority > 0);
            battle.resolve[losing] = (battle.resolve[losing] - dt * majority.abs() as f32).max(0.0);
        }
    }
    let outcome = match battle.mode {
        _ if active == [false, false] => Some((Finish::Elimination, None)),
        Mode::Conquest
            if !battle.objectives.is_empty() && owned.contains(&battle.objectives.len()) =>
        {
            Some((
                Finish::Conquest,
                Some(if owned[0] > owned[1] {
                    Team::Blue
                } else {
                    Team::Red
                }),
            ))
        }
        Mode::Conquest => None,
        _ if active[0] != active[1] => Some((
            Finish::Elimination,
            Some(if active[0] { Team::Blue } else { Team::Red }),
        )),
        Mode::Frontline if battle.resolve.contains(&0.0) => {
            Some((Finish::Resolve, leader(battle.resolve)))
        }
        Mode::Control
            if battle.score.iter().any(|&score| score >= CONTROL_TARGET)
                && leader(battle.score).is_some() =>
        {
            Some((Finish::Score, leader(battle.score)))
        }
        Mode::Timed if battle.elapsed >= TIMED_LIMIT => {
            battle.elapsed = TIMED_LIMIT;
            Some((Finish::Time, leader(battle.score)))
        }
        _ => None,
    };
    if let Some((reason, winner)) = outcome {
        battle.finish = Some(reason);
        battle.winner = winner;
        battle.draw = winner.is_none();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 64.0;

    #[test]
    fn scouting_reveals_distant_contacts_without_granting_fire_through_blockers() {
        let mut battle = new_battle(&Roster::default());
        battle.squads[0].pos = Vec2::new(0.0, -180.0);
        battle.squads[4].pos = Vec2::new(405.0, -180.0);
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert!(!battle.squads[4].visible_to_blue);
        battle.squads[0].kit = Kit::Scout;
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert!(battle.squads[4].visible_to_blue);
        let (blocker, _) = crate::contact::sight_blockers(battle.map)[0];
        battle.squads[0].pos = blocker - Vec2::X * 100.0;
        battle.squads[4].pos = blocker + Vec2::X * 100.0;
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert!(!battle.squads[4].visible_to_blue);
        assert_eq!(
            battle.squads[4].last_contact.map(|(pos, _)| pos),
            Some(Vec2::new(405., -180.))
        );
        assert!(battle.effects.is_empty());
    }

    #[test]
    fn cautious_advance_halts_at_contact_then_resumes_after_smoke_blocks_sight() {
        let mut battle = new_battle(&Roster::default());
        battle.squads[0].pos = Vec2::ZERO;
        battle.squads[4].pos = Vec2::X * 170.0;
        battle.squads[0].orders.push_back(Vec2::X * 300.0);
        battle.squads[0].cautious_move = true;
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert_eq!(battle.squads[0].pos, Vec2::ZERO);
        assert!(battle.squads[0].suppression > 0.0);
        battle.squads[4].smoke = 5.0;
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert!(battle.squads[0].pos.x > 0.0);
        assert!(!battle.squads[4].visible_to_blue);
    }

    #[test]
    fn suppression_reduces_movement_and_recovers_without_contact() {
        let mut battle = new_battle(&Roster::default());
        for (index, squad) in battle.squads[..2].iter_mut().enumerate() {
            squad.pos = Vec2::Y * (index as f32 * 100.0);
            squad.orders.push_back(squad.pos + Vec2::X * 200.0);
        }
        battle.squads[0].suppression = 0.9;
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert!(battle.squads[0].pos.x < battle.squads[1].pos.x);
        assert!(battle.squads[0].suppression < 0.9);
    }

    #[test]
    fn frontline_flags_follow_influence_beyond_the_capture_ring() {
        let mut battle = new_battle(&Roster {
            mode: Mode::Frontline,
            ..default()
        });
        let flag = battle.objectives[1].pos;
        battle.squads[0].pos = flag - Vec2::X * 135.0;
        battle.squads[4].pos = flag + Vec2::X * 135.0;
        for soldier in &mut battle.squads[0].soldiers[6..] {
            soldier.hp = 0.0;
        }
        update_frontline(&mut battle, DT);
        assert!(battle.objectives[1].contested);
        battle.squads[4].pos = Team::Red.spawn();
        update_frontline(&mut battle, DT);
        assert_eq!(battle.objectives[1].owner, Some(Team::Blue));
        battle.squads[0].pos = Team::Blue.spawn();
        update_frontline(&mut battle, DT);
        assert_eq!(battle.objectives[1].owner, None);
    }

    #[test]
    fn majority_wins_by_draining_the_other_teams_resolve() {
        let mut battle = new_battle(&Roster {
            mode: Mode::Frontline,
            ..default()
        });
        battle.objectives[0].owner = Some(Team::Blue);
        battle.resolve[1] = 0.5;
        resolve_operation(&mut battle, 0.5, 0.0);
        assert_eq!(battle.finish, Some(Finish::Resolve));
        assert_eq!(battle.winner, Some(Team::Blue));
    }

    #[test]
    fn reserve_unlock_requires_points_and_is_single_use() {
        let mut battle = new_battle(&Roster {
            mode: Mode::Frontline,
            reserve_kit: Kit::Medic,
            ..default()
        });
        assert!(!call_reserve(&mut battle, Team::Blue));
        battle.elapsed = RESERVE_UNLOCK;
        assert!(!call_reserve(&mut battle, Team::Blue));
        battle.requisition[0] = 100.0;
        assert!(call_reserve(&mut battle, Team::Blue));
        assert_eq!(battle.squads[8].kit, Kit::Medic);
        assert_eq!(battle.squads[8].pos.distance(Team::Blue.spawn()), 27.5);
        assert_eq!(battle.requisition[0], 100.0 - RESERVE_COST);
        battle.requisition[0] = 100.0;
        assert!(!call_reserve(&mut battle, Team::Blue));
    }

    #[test]
    fn control_scores_only_uncontested_owned_objectives() {
        let mut battle = new_battle(&Roster {
            mode: Mode::Control,
            ..default()
        });
        battle.objectives[0].owner = Some(Team::Blue);
        battle.objectives[1].owner = Some(Team::Blue);
        battle.objectives[1].contested = true;
        battle.score[0] = CONTROL_TARGET - 1.0;
        resolve_operation(&mut battle, 1.0, 0.0);
        assert_eq!(battle.score, [CONTROL_TARGET, 0.0]);
        assert_eq!(battle.finish, Some(Finish::Score));
        assert_eq!(battle.winner, Some(Team::Blue));
    }

    #[test]
    fn timed_mode_scores_only_until_the_deadline_and_can_draw() {
        let mut battle = new_battle(&Roster {
            mode: Mode::Timed,
            ..default()
        });
        battle.elapsed = TIMED_LIMIT + 0.5;
        battle.objectives[0].owner = Some(Team::Blue);
        battle.score = [1.0, 1.5];
        resolve_operation(&mut battle, 1.0, TIMED_LIMIT - 0.5);
        assert_eq!(battle.score, [1.5, 1.5]);
        assert_eq!(battle.elapsed, TIMED_LIMIT);
        assert_eq!(battle.finish, Some(Finish::Time));
        assert!(battle.draw);
    }

    #[test]
    fn alternate_layout_and_personnel_cap_apply_to_deployment() {
        let mut roster = Roster {
            map: crate::operation::Map::Highmoor,
            ..default()
        };
        roster.cycle_manpower(0);
        roster.cycle_manpower(1);
        assert_eq!(roster.squads[0].manpower, 4);
        assert_eq!(roster.squads[1].manpower, 10);
        assert!(roster.personnel() <= crate::operation::PERSONNEL_LIMIT);
        let battle = new_battle(&roster);
        assert_eq!(battle.objectives[0].pos, roster.map.points()[0]);
        assert_eq!(
            battle.report(Team::Blue).soldiers_deployed,
            usize::from(roster.personnel())
        );
    }

    #[test]
    fn depots_require_friendly_uncontested_ownership() {
        let mut battle = new_battle(&Roster::default());
        let depot = battle.objectives[DEPOTS[0]].pos;
        battle.squads[0].pos = depot;
        battle.squads[0].supplies = 20.0;
        battle.squads[0].soldiers[0].hp = 50.0;
        battle.objectives[DEPOTS[0]].owner = Some(Team::Blue);
        battle.objectives[DEPOTS[0]].capture = 1.0;
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert!(battle.squads[0].supplies > 20.0);
        assert_eq!(battle.squads[0].soldiers[0].hp, 50.0);
        battle.objectives[DEPOTS[0]].contested = true;
        assert_eq!(
            supply_destination(&battle.objectives, Team::Blue, depot),
            Team::Blue.spawn()
        );
    }

    #[test]
    fn hold_fire_preserves_supply_and_toggling_it_off_resumes_fire() {
        let mut battle = new_battle(&Roster::default());
        battle.squads[0].pos = Vec2::ZERO;
        battle.squads[4].pos = Vec2::X * 120.0;
        battle.selected.insert(0);
        toggle_fire_control(&mut battle);
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert_eq!(battle.squads[0].supplies, 100.0);
        toggle_fire_control(&mut battle);
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert!(battle.squads[0].supplies < 100.0);
    }

    #[test]
    fn match_report_counts_ownership_and_living_soldiers_by_team() {
        let mut battle = new_battle(&Roster::default());
        battle.objectives[0].owner = Some(Team::Blue);
        battle.objectives[1].owner = Some(Team::Red);
        for soldier in &mut battle.squads[0].soldiers {
            soldier.hp = 0.0;
        }
        battle.squads[4].soldiers[0].hp = 25.0;

        let blue = battle.report(Team::Blue);
        assert_eq!(blue.objectives, 1);
        assert_eq!(blue.squads_remaining, 3);
        assert_eq!(blue.soldiers_remaining, 24);
        assert_eq!(blue.soldiers_deployed, 32);

        let red = battle.report(Team::Red);
        assert_eq!(red.objectives, 1);
        assert_eq!(red.squads_remaining, 4);
        assert_eq!(red.soldiers_remaining, red.soldiers_deployed);
    }

    #[test]
    fn roster_deploys_four_squads_and_six_neutral_objectives() {
        let mut roster = Roster::default();
        roster.squads[0].callsign = "Test".to_owned();
        roster.squads[0].manpower = 5;
        let battle = new_battle(&roster);
        assert_eq!(battle.squads.len(), 8);
        assert_eq!(battle.squads[0].callsign, "Test");
        assert_eq!(battle.squads[0].manpower(), 5);
        assert_eq!(battle.objectives.len(), 6);
        assert!(battle.objectives.iter().all(|point| point.owner.is_none()));
    }

    #[test]
    fn contested_capture_resumes_when_one_squad_leaves() {
        let mut battle = new_battle(&Roster::default());
        let point = battle.objectives[0].pos;
        battle.squads[0].pos = point;
        battle.squads[4].pos = point;
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert!(battle.objectives[0].contested);
        assert_eq!(battle.objectives[0].capture, 0.0);

        battle.squads[4].pos = Team::Red.spawn();
        for _ in 0..700 {
            step_battle(&mut battle, Difficulty::Standard, DT);
        }
        assert!(!battle.objectives[0].contested);
        assert_eq!(battle.objectives[0].owner, Some(Team::Blue));
        assert_eq!(battle.winner, None);
    }

    #[test]
    fn red_ai_completes_all_six_objectives_after_blue_is_destroyed() {
        let mut battle = new_battle(&Roster::default());
        for soldier in battle.squads[..4]
            .iter_mut()
            .flat_map(|squad| &mut squad.soldiers)
        {
            soldier.hp = 0.0;
        }
        step_battle(&mut battle, Difficulty::Standard, DT);
        assert_eq!(battle.winner, None);
        for _ in 0..(180 * 64) {
            crate::ai::direct_squads(&mut battle);
            step_battle(&mut battle, Difficulty::Standard, DT);
            if battle.winner.is_some() {
                break;
            }
        }
        assert_eq!(battle.winner, Some(Team::Red));
        assert!(
            battle
                .objectives
                .iter()
                .all(|point| point.owner == Some(Team::Red))
        );
    }

    #[test]
    fn orders_queue_and_resupply_returns_a_squad_to_base() {
        let mut battle = new_battle(&Roster::default());
        battle.squads[0].pos = Vec2::ZERO;
        battle.squads[0].supplies = 0.0;
        battle.selected.insert(0);
        issue_move(&mut battle, Vec2::new(20.0, 0.0), false);
        issue_move(&mut battle, Vec2::new(20.0, 40.0), true);
        assert_eq!(battle.squads[0].orders.len(), 2);
        resupply_selected(&mut battle);
        assert_eq!(
            battle.squads[0].orders,
            VecDeque::from([Team::Blue.spawn()])
        );
        for _ in 0..(30 * 64) {
            step_battle(&mut battle, Difficulty::Standard, DT);
        }
        assert_eq!(battle.squads[0].supplies, 100.0);
    }
}
