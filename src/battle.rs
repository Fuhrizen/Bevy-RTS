use crate::model::*;
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
    let points = [
        ("Northwest", Vec2::new(-620.0, 380.0)),
        ("North", Vec2::new(0.0, 440.0)),
        ("Northeast", Vec2::new(620.0, 380.0)),
        ("Southwest", Vec2::new(-620.0, -380.0)),
        ("South", Vec2::new(0.0, -440.0)),
        ("Southeast", Vec2::new(620.0, -380.0)),
    ];
    let objectives = points
        .into_iter()
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
        paused: false,
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

/// Resupply is only available at the team's home position.
pub fn resupply_selected(battle: &mut Battle) {
    for &i in &battle.selected {
        if let Some(squad) = battle.squads.get_mut(i) {
            if squad.team == Team::Blue && squad.pos.distance(squad.team.spawn()) <= SUPPLY_RADIUS {
                squad.supplies = 100.0;
                for soldier in &mut squad.soldiers {
                    if soldier.hp > 0.0 {
                        soldier.hp = (soldier.hp + 35.0).min(100.0);
                    }
                }
            } else if squad.team == Team::Blue && squad.alive() {
                squad.orders.clear();
                squad.orders.push_back(squad.team.spawn());
                squad.entrenched = false;
                squad.focus = None;
            }
        }
    }
}

fn simulate(time: Res<Time<Fixed>>, settings: Res<Settings>, mut battle: ResMut<Battle>) {
    step_battle(&mut battle, settings.difficulty, time.delta_secs());
}

fn step_battle(battle: &mut Battle, difficulty: Difficulty, dt: f32) {
    if battle.paused || battle.winner.is_some() || battle.draw {
        return;
    }
    battle.elapsed += dt;
    for squad in &mut battle.squads {
        squad.smoke = (squad.smoke - dt).max(0.0);
        for soldier in &mut squad.soldiers {
            soldier.cooldown = (soldier.cooldown - dt).max(0.0);
        }
        if !squad.alive() {
            squad.orders.clear();
            continue;
        }
        if let Some(&destination) = squad.orders.front() {
            let delta = destination - squad.pos;
            let speed = if matches!(squad.kit, Kit::Scout) {
                100.0
            } else {
                80.0
            };
            if delta.length() <= speed * dt {
                squad.pos = destination;
                squad.orders.pop_front();
            } else {
                squad.pos += delta.normalize_or_zero() * speed * dt;
            }
        }
        if squad.pos.distance(squad.team.spawn()) <= SUPPLY_RADIUS {
            squad.supplies = (squad.supplies + 8.0 * dt).min(100.0);
            for soldier in &mut squad.soldiers {
                if soldier.hp > 0.0 {
                    soldier.hp = (soldier.hp + 3.0 * dt).min(100.0);
                }
            }
        }
    }
    // Resolve fire from a snapshot so both sides get a turn in the same frame.
    let mut volleys = Vec::new();
    for i in 0..battle.squads.len() {
        let attacker = &battle.squads[i];
        if !attacker.alive() || attacker.supplies < 1.0 {
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
                    s.team != attacker.team && s.alive() && s.pos.distance(attacker.pos) <= range
                })
            })
            .or_else(|| {
                battle
                    .squads
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| {
                        s.team != attacker.team
                            && s.alive()
                            && s.pos.distance(attacker.pos) <= range
                    })
                    .min_by(|(_, a), (_, b)| {
                        a.pos
                            .distance_squared(attacker.pos)
                            .total_cmp(&b.pos.distance_squared(attacker.pos))
                    })
                    .map(|(j, _)| j)
            });
        if let Some(j) = target {
            let defense = (if battle.squads[j].entrenched {
                0.65
            } else {
                1.0
            }) * (if battle.squads[j].smoke > 0.0 {
                0.55
            } else {
                1.0
            });
            let ready = attacker
                .soldiers
                .iter()
                .filter(|s| s.hp > 0.0 && s.cooldown <= 0.0)
                .count();
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
                volleys.push((i, j, ready as f32 * 7.0 * defense * difficulty));
            }
        }
    }
    for &(attacker, _, _) in &volleys {
        let squad = &mut battle.squads[attacker];
        let cooldown = if squad.kit == Kit::Support { 0.85 } else { 1.2 };
        for soldier in &mut squad.soldiers {
            if soldier.hp > 0.0 && soldier.cooldown <= 0.0 {
                soldier.cooldown = cooldown;
                soldier.veterancy = soldier.veterancy.saturating_add(1).min(100);
                squad.supplies = (squad.supplies - 0.45).max(0.0);
            }
        }
    }
    for (_, j, mut damage) in volleys {
        let squad = &mut battle.squads[j];
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
    let presence: Vec<_> = battle
        .objectives
        .iter()
        .map(|objective| {
            let blue = battle.squads.iter().any(|s| {
                s.team == Team::Blue && s.alive() && s.pos.distance(objective.pos) <= CAPTURE_RADIUS
            });
            let red = battle.squads.iter().any(|s| {
                s.team == Team::Red && s.alive() && s.pos.distance(objective.pos) <= CAPTURE_RADIUS
            });
            (blue, red)
        })
        .collect();
    for (objective, (blue, red)) in battle.objectives.iter_mut().zip(presence) {
        objective.contested = blue && red;
        if blue != red {
            objective.capture =
                (objective.capture + if blue { dt / 10.0 } else { -dt / 10.0 }).clamp(-1.0, 1.0);
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
    let blue_alive = battle
        .squads
        .iter()
        .any(|s| s.team == Team::Blue && s.alive());
    let red_alive = battle
        .squads
        .iter()
        .any(|s| s.team == Team::Red && s.alive());
    if !blue_alive && !red_alive {
        battle.draw = true;
    } else if !battle.objectives.is_empty()
        && battle
            .objectives
            .iter()
            .all(|o| o.owner == Some(Team::Blue))
    {
        battle.winner = Some(Team::Blue);
    } else if !battle.objectives.is_empty()
        && battle.objectives.iter().all(|o| o.owner == Some(Team::Red))
    {
        battle.winner = Some(Team::Red);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 64.0;

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
