//! Shared game state for simulation, rendering, and the player interface.
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
}

impl Default for Roster {
    fn default() -> Self {
        Self {
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

pub struct VolleyEffect {
    pub from: Vec2,
    pub to: Vec2,
    pub team: Team,
    pub ttl: f32,
}

#[derive(Default, Resource)]
pub struct Battle {
    pub squads: Vec<Squad>,
    pub objectives: Vec<Objective>,
    pub winner: Option<Team>,
    pub draw: bool,
    pub elapsed: f32,
    pub effects: Vec<VolleyEffect>,
    pub volley_count: u64,
    pub order_count: u64,
    pub paused: bool,
    pub selected: BTreeSet<usize>,
}
