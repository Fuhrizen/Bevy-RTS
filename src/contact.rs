//! Spotting and occlusion for BEV Front's abstract skirmish maps.
use bevy::prelude::*;

use crate::{
    model::{Kit, Squad, Team},
    operation::{Map, in_cover},
};

pub const PIN_THRESHOLD: f32 = 0.65;

pub fn sight_blockers(map: Map) -> [(Vec2, f32); 3] {
    let offset = match map {
        Map::Riverbend => Vec2::new(-260., 40.),
        Map::Highmoor => Vec2::new(110., 50.),
        Map::Crossroads => Vec2::new(-190., -20.),
    };
    [
        (offset, 64.),
        (Vec2::new(315., 255.) - offset * 0.3, 54.),
        (Vec2::new(-370., -220.) + offset * 0.2, 66.),
    ]
}

fn crosses(a: Vec2, b: Vec2, center: Vec2, radius: f32) -> bool {
    let segment = b - a;
    let squared = segment.length_squared();
    if squared == 0.0 {
        return false;
    }
    let t = (center - a).dot(segment) / squared;
    (0.0..=1.0).contains(&t) && (a + t * segment).distance_squared(center) < radius * radius
}

pub fn can_see(map: Map, observer: &Squad, target: &Squad, squads: &[Squad]) -> bool {
    let range = if observer.kit == Kit::Scout {
        470.
    } else {
        310.
    };
    let concealment = if in_cover(map, target.pos) && !target.engaged() {
        0.67
    } else {
        1.0
    };
    observer.alive()
        && target.alive()
        && observer.pos.distance_squared(target.pos)
            <= (range * concealment) * (range * concealment)
        && !sight_blockers(map)
            .iter()
            .any(|&(center, radius)| crosses(observer.pos, target.pos, center, radius))
        && !squads
            .iter()
            .any(|screen| screen.smoke > 0.0 && crosses(observer.pos, target.pos, screen.pos, 53.))
}

pub fn spotted(map: Map, squads: &[Squad], team: Team, target: &Squad) -> bool {
    squads
        .iter()
        .any(|observer| observer.team == team && can_see(map, observer, target, squads))
}
