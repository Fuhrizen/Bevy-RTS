//! Shared game state for simulation, rendering, and the player interface.
use crate::operation::{Finish, Map, Mode, PERSONNEL_LIMIT};
use std::collections::{BTreeSet, VecDeque};

use bevy::prelude::*;

pub const MAP_HALF: Vec2 = Vec2::new(1120.0, 720.0);
pub const CAPTURE_RADIUS: f32 = 85.0;
pub const SUPPLY_RADIUS: f32 = 100.0;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum Screen {
    #[default]
    Menu,
    Settings,
    Roster,
    Battle,
}

#[derive(Component)]
pub struct HudBlock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Team {
    Blue,
    Red,
}

impl Team {
    pub fn label(self) -> &'static str {
        match self {
            Self::Blue => "Blue",
            Self::Red => "Red",
        }
    }

    pub fn spawn(self) -> Vec2 {
        match self {
            Self::Blue => Vec2::new(-875.0, -70.0),
            Self::Red => Vec2::new(875.0, 70.0),
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Blue => Color::srgb(0.38, 0.73, 1.0),
            Self::Red => Color::srgb(1.0, 0.38, 0.38),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kit {
    Rifle,
    Support,
    Scout,
    Medic,
}

impl Kit {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rifle => "Rifle",
            Self::Support => "Support",
            Self::Scout => "Scout",
            Self::Medic => "Medic",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Leader,
    Rifle,
    Support,
    Medic,
}

impl Role {
    pub fn label(self) -> &'static str {
        match self {
            Self::Leader => "Leader",
            Self::Rifle => "Rifle",
            Self::Support => "Support",
            Self::Medic => "Medic",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Difficulty {
    Recruit,
    Standard,
    Veteran,
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Self::Recruit => "Recruit",
            Self::Standard => "Standard",
            Self::Veteran => "Veteran",
        }
    }
}

#[derive(Resource)]
pub struct Settings {
    pub difficulty: Difficulty,
    pub fullscreen: bool,
    pub edge_pan: bool,
    pub ui_scale: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Standard,
            fullscreen: false,
            edge_pan: true,
            ui_scale: 1.0,
        }
    }
}

pub struct SquadConfig {
    pub callsign: String,
    pub kit: Kit,
    pub manpower: u8,
}

#[derive(Resource)]
pub struct Roster {
    pub squads: [SquadConfig; 4],
    pub map: Map,
    pub mode: Mode,
}

impl Default for Roster {
    fn default() -> Self {
        Self {
            map: Map::default(),
            mode: Mode::default(),
            squads: [
                ("Alpha", Kit::Rifle),
                ("Bravo", Kit::Support),
                ("Charlie", Kit::Scout),
                ("Delta", Kit::Medic),
            ]
            .map(|(callsign, kit)| SquadConfig {
                callsign: callsign.into(),
                kit,
                manpower: 8,
            }),
        }
    }
}

impl Roster {
    pub fn personnel(&self) -> u8 {
        self.squads.iter().map(|squad| squad.manpower).sum()
    }

    pub fn cycle_manpower(&mut self, index: usize) {
        let current = self.squads[index].manpower;
        let next = if current >= 10 { 4 } else { current + 2 };
        self.squads[index].manpower = if self.personnel() - current + next <= PERSONNEL_LIMIT {
            next
        } else {
            4
        };
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiState {
    Advance(usize),
    Hold(usize),
    Engage(usize),
    Resupply,
    Recover,
    Idle,
}

pub struct Soldier {
    pub id: u8,
    pub hp: f32,
    pub role: Role,
    pub veterancy: u8,
    pub cooldown: f32,
}

pub struct Squad {
    pub team: Team,
    pub pos: Vec2,
    pub callsign: String,
    pub kit: Kit,
    pub soldiers: Vec<Soldier>,
    pub supplies: f32,
    pub orders: VecDeque<Vec2>,
    pub entrenched: bool,
    pub hold_fire: bool,
    pub smoke: f32,
    pub focus: Option<usize>,
    pub ai: AiState,
}

impl Squad {
    pub fn manpower(&self) -> usize {
        self.soldiers
            .iter()
            .filter(|soldier| soldier.hp > 0.0)
            .count()
    }

    pub fn engaged(&self) -> bool {
        self.soldiers
            .iter()
            .any(|soldier| soldier.hp > 0.0 && soldier.cooldown > 0.0)
    }

    pub fn marker_status(&self) -> (&'static str, Color) {
        const WARNING: Color = Color::srgb(1.0, 0.69, 0.30);
        if self.supplies < 1.0 {
            ("EMPTY", WARNING)
        } else if self.supplies <= 20.0 {
            ("LOW SUPPLY", WARNING)
        } else if self.manpower() * 2 <= self.soldiers.len() {
            ("LOW STRENGTH", WARNING)
        } else if self.smoke > 0.0 {
            ("SMOKE", Color::srgb(0.82, 0.88, 0.90))
        } else if self.hold_fire {
            ("HOLD FIRE", Color::srgb(0.75, 0.69, 1.0))
        } else if self.engaged() {
            ("ENGAGED", Color::srgb(1.0, 0.52, 0.30))
        } else if !self.orders.is_empty() {
            ("MOVING", Color::srgb(0.48, 0.86, 0.97))
        } else if self.entrenched {
            ("DUG IN", Color::srgb(0.63, 0.88, 0.57))
        } else {
            ("READY", self.team.color())
        }
    }

    pub fn alive(&self) -> bool {
        self.manpower() != 0
    }
}

pub struct Objective {
    pub name: &'static str,
    pub pos: Vec2,
    pub owner: Option<Team>,
    pub capture: f32,
    pub contested: bool,
}

impl Objective {
    pub fn display_color(&self) -> Color {
        if self.contested {
            return Color::srgb(1.0, 0.74, 0.34);
        }
        let team = if self.capture > 0.0 {
            Some(Team::Blue)
        } else if self.capture < 0.0 {
            Some(Team::Red)
        } else {
            self.owner
        };
        team.map_or(Color::srgb(0.75, 0.78, 0.73), Team::color)
    }

    pub fn capture_label(&self) -> String {
        if self.contested {
            "TIED".into()
        } else if let Some(team) = self.owner {
            if self.capture.abs() < 1.0 {
                "LOSING".into()
            } else {
                team.label().to_uppercase()
            }
        } else if self.capture == 0.0 {
            "OPEN".into()
        } else {
            format!("{:.0}%", (self.capture.abs() * 100.0).floor())
        }
    }
}

pub struct VolleyEffect {
    pub from: Vec2,
    pub to: Vec2,
    pub team: Team,
    pub ttl: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pace {
    Planning,
    #[default]
    Deliberate,
    Normal,
}

impl Pace {
    pub fn factor(self) -> f32 {
        match self {
            Self::Planning => 0.0,
            Self::Deliberate => 0.65,
            Self::Normal => 1.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Planning => "PLAN",
            Self::Deliberate => "0.65x",
            Self::Normal => "1x",
        }
    }
}

#[derive(Default, Resource)]
pub struct Battle {
    pub squads: Vec<Squad>,
    pub objectives: Vec<Objective>,
    pub winner: Option<Team>,
    pub draw: bool,
    pub elapsed: f32,
    pub effects: Vec<VolleyEffect>,
    pub map: Map,
    pub mode: Mode,
    pub score: [f32; 2],
    pub finish: Option<Finish>,
    pub volley_count: u64,
    pub order_count: u64,
    pub paused: bool,
    pub pace: Pace,
    pub resume_pace: Pace,
    pub selected: BTreeSet<usize>,
}

/// Team totals calculated from the final simulation snapshot.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TeamReport {
    pub objectives: usize,
    pub squads_remaining: usize,
    pub soldiers_remaining: usize,
    pub soldiers_deployed: usize,
}

impl Battle {
    pub fn set_pace(&mut self, pace: Pace) {
        if pace != Pace::Planning {
            self.resume_pace = pace;
        }
        self.pace = pace;
    }

    pub fn report(&self, team: Team) -> TeamReport {
        let mut report = TeamReport {
            objectives: self
                .objectives
                .iter()
                .filter(|point| point.owner == Some(team))
                .count(),
            ..default()
        };
        for squad in self.squads.iter().filter(|squad| squad.team == team) {
            let remaining = squad.manpower();
            report.squads_remaining += usize::from(remaining > 0);
            report.soldiers_remaining += remaining;
            report.soldiers_deployed += squad.soldiers.len();
        }
        report
    }
}
