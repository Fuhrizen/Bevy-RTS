//! Opponent state transitions for the sector-control game mode.
use bevy::prelude::*;

use crate::model::*;

pub fn direct_red(mut battle: ResMut<Battle>) {
    direct_squads(&mut battle);
}

pub fn direct_squads(battle: &mut Battle) {
    if battle.paused || battle.winner.is_some() || battle.draw || battle.objectives.is_empty() {
        return;
    }

    let enemies: Vec<_> = battle
        .squads
        .iter()
        .enumerate()
        .filter(|(_, squad)| squad.team == Team::Blue && squad.alive())
        .map(|(i, squad)| (i, squad.pos))
        .collect();
    let mut claims = vec![0; battle.objectives.len()];
    for squad in &battle.squads {
        if squad.team == Team::Red
            && squad.alive()
            && let AiState::Advance(point) | AiState::Hold(point) = squad.ai
        {
            claims[point] += 1;
        }
    }

    for squad in &mut battle.squads {
        if squad.team != Team::Red || !squad.alive() {
            continue;
        }
        let assigned = match squad.ai {
            AiState::Advance(point) | AiState::Hold(point) => Some(point),
            _ => None,
        };
        if let Some(point) = assigned {
            claims[point] -= 1;
        }
        squad.ai = choose_state(squad, &battle.objectives, &claims, &enemies);
        squad.focus = None;
        squad.entrenched = false;
        match squad.ai {
            AiState::Resupply | AiState::Recover => {
                route_to(squad, Team::Red.spawn());
            }
            AiState::Engage(enemy) => {
                squad.focus = Some(enemy);
                squad.orders.clear();
                squad.entrenched = true;
            }
            AiState::Advance(point) => {
                claims[point] += 1;
                route_to(squad, battle.objectives[point].pos);
            }
            AiState::Hold(point) => {
                claims[point] += 1;
                squad.orders.clear();
                squad.entrenched = true;
            }
            AiState::Idle => {
                squad.orders.clear();
            }
        }
    }
}

fn choose_state(
    squad: &Squad,
    objectives: &[Objective],
    claims: &[u32],
    enemies: &[(usize, Vec2)],
) -> AiState {
    if squad.supplies < 14.0 || squad.ai == AiState::Resupply && squad.supplies < 90.0 {
        return AiState::Resupply;
    }
    let living = squad.soldiers.iter().filter(|soldier| soldier.hp > 0.0);
    let average_hp = living.map(|soldier| soldier.hp).sum::<f32>() / squad.manpower() as f32;
    if average_hp < 38.0 || squad.ai == AiState::Recover && average_hp < 84.0 {
        return AiState::Recover;
    }

    let engage_distance = if matches!(squad.ai, AiState::Engage(_)) {
        218.0
    } else {
        204.0
    };
    if let Some((enemy, _)) = enemies
        .iter()
        .filter(|(_, position)| position.distance(squad.pos) <= engage_distance)
        .min_by(|(_, a), (_, b)| {
            a.distance_squared(squad.pos)
                .total_cmp(&b.distance_squared(squad.pos))
        })
    {
        return AiState::Engage(*enemy);
    }

    let target = match squad.ai {
        AiState::Advance(point) | AiState::Hold(point)
            if objectives[point].owner != Some(Team::Red) || objectives[point].contested =>
        {
            Some(point)
        }
        _ => objectives
            .iter()
            .enumerate()
            .filter(|(_, point)| point.owner != Some(Team::Red) || point.contested)
            .map(|(i, point)| {
                let score = squad.pos.distance(point.pos) + 520.0 * claims[i] as f32
                    - if point.contested { 190.0 } else { 0.0 };
                (i, score)
            })
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(i, _)| i),
    };

    match target {
        Some(point) if squad.pos.distance(objectives[point].pos) < CAPTURE_RADIUS * 0.6 => {
            AiState::Hold(point)
        }
        Some(point) => AiState::Advance(point),
        None => AiState::Idle,
    }
}

fn route_to(squad: &mut Squad, point: Vec2) {
    if squad.pos.distance_squared(point) < 1.0 {
        squad.orders.clear();
    } else if squad.orders.front().copied() != Some(point) {
        squad.orders.clear();
        squad.orders.push_back(point);
    }
}
