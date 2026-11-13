//! Scenario layouts and victory rules for the skirmish setup.
use bevy::prelude::*;

use crate::model::Team;

pub const CONTROL_TARGET: f32 = 480.0;
pub const TIMED_LIMIT: f32 = 180.0;
pub const PERSONNEL_LIMIT: u8 = 32;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Map {
    #[default]
    Riverbend,
    Highmoor,
    Crossroads,
}

impl Map {
    pub fn next(self) -> Self {
        match self {
            Self::Riverbend => Self::Highmoor,
            Self::Highmoor => Self::Crossroads,
            Self::Crossroads => Self::Riverbend,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Riverbend => "Riverbend",
            Self::Highmoor => "Highmoor",
            Self::Crossroads => "Crossroads",
        }
    }

    pub fn points(self) -> [Vec2; 6] {
        let coordinates = match self {
            Self::Riverbend => [
                (-620., 380.),
                (0., 440.),
                (620., 380.),
                (-620., -380.),
                (0., -440.),
                (620., -380.),
            ],
            Self::Highmoor => [
                (-680., 280.),
                (-190., 510.),
                (480., 300.),
                (-480., -300.),
                (190., -510.),
                (680., -280.),
            ],
            Self::Crossroads => [
                (-390., 360.),
                (0., 250.),
                (390., 360.),
                (-390., -360.),
                (0., -250.),
                (390., -360.),
            ],
        };
        coordinates.map(|(x, y)| Vec2::new(x, y))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Mode {
    #[default]
    Conquest,
    Control,
    Timed,
    Frontline,
}

impl Mode {
    pub fn next(self) -> Self {
        match self {
            Self::Conquest => Self::Control,
            Self::Control => Self::Timed,
            Self::Timed => Self::Frontline,
            Self::Frontline => Self::Conquest,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Conquest => "Conquest",
            Self::Control => "Control",
            Self::Timed => "Timed",
            Self::Frontline => "Frontline",
        }
    }

    pub fn rules(self) -> &'static str {
        match self {
            Self::Conquest => "Hold all six sectors to win.",
            Self::Control => "Uncontested sectors earn points. First to 480 wins.",
            Self::Timed => "Lead the control score when the 3-minute clock ends.",
            Self::Frontline => {
                "Project influence over flags. The sector majority drains enemy resolve."
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Finish {
    Conquest,
    Score,
    Time,
    Elimination,
    Resolve,
}

pub fn leader(values: [f32; 2]) -> Option<Team> {
    match values[0].total_cmp(&values[1]) {
        std::cmp::Ordering::Greater => Some(Team::Blue),
        std::cmp::Ordering::Less => Some(Team::Red),
        std::cmp::Ordering::Equal => None,
    }
}

/// The same circular areas drive cover in the simulation and on the map.
pub fn cover_areas(map: Map) -> [(Vec2, f32); 4] {
    let points = map.points();
    [
        (points[0] + Vec2::new(65., -60.), 100.),
        (points[2] + Vec2::new(-60., -60.), 100.),
        (points[3] + Vec2::new(65., 60.), 100.),
        (points[5] + Vec2::new(-60., 60.), 100.),
    ]
}

pub fn in_cover(map: Map, point: Vec2) -> bool {
    cover_areas(map)
        .iter()
        .any(|(center, radius)| point.distance_squared(*center) <= radius * radius)
}

pub const DEPOTS: [usize; 2] = [1, 4];

pub const RESOLVE: f32 = 150.0;
pub const RESERVE_COST: f32 = 60.0;
pub const RESERVE_UNLOCK: f32 = 45.0;

#[derive(Clone, Copy)]
pub struct Influence {
    pub pos: Vec2,
    pub team: Team,
    pub weight: f32,
}
pub fn influence(squads: &[crate::model::Squad]) -> impl Iterator<Item = Influence> + '_ {
    squads.iter().filter_map(|squad| {
        let strength = squad.manpower();
        (strength > 0).then_some(Influence {
            pos: squad.pos,
            team: squad.team,
            weight: strength as f32,
        })
    })
}
pub fn territory(point: Vec2, field: &[Influence]) -> (Option<Team>, bool) {
    let mut sides = [0.0; 2];
    for emitter in field {
        let weight = (1.0 - point.distance(emitter.pos) / 360.0).max(0.0) * emitter.weight;
        sides[usize::from(emitter.team == Team::Red)] += weight;
    }
    let contested = sides[0].min(sides[1]) >= 0.5 && (sides[0] - sides[1]).abs() < 1.5;
    let owner = if !contested && sides[0].max(sides[1]) >= 0.5 {
        leader(sides)
    } else {
        None
    };
    (owner, contested)
}
