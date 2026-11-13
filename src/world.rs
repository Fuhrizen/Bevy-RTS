//! Minimal playable-world scaffold. Gameplay systems can grow into separate plugins.

use bevy::prelude::*;
use brevy_core::grid;

const CAMERA_SPEED: f32 = 420.0;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, pan_camera);
    }
}

#[derive(Component)]
struct Unit;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let tile_size = Vec2::splat(grid::CELL_SIZE - 2.0);
    for row in 0..grid::ROWS {
        for column in 0..grid::COLUMNS {
            let [x, y] = grid::tile_position(column, row);
            let shade = if (row + column) % 2 == 0 { 0.20 } else { 0.23 };
            commands.spawn((
                Sprite::from_color(Color::srgb(0.12, shade, 0.16), tile_size),
                Transform::from_xyz(x, y, 0.0),
            ));
        }
    }

    for (column, row, color) in [
        (3, 5, Color::srgb(0.24, 0.60, 0.92)),
        (4, 5, Color::srgb(0.24, 0.60, 0.92)),
        (13, 6, Color::srgb(0.93, 0.55, 0.24)),
        (14, 6, Color::srgb(0.93, 0.55, 0.24)),
    ] {
        let [x, y] = grid::tile_position(column, row);
        commands.spawn((
            Unit,
            Sprite::from_color(color, Vec2::splat(grid::CELL_SIZE * 0.60)),
            Transform::from_xyz(x, y, 1.0),
        ));
    }
}

fn pan_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };

    let mut movement = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        movement.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        movement.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        movement.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        movement.y += 1.0;
    }

    camera.translation +=
        (movement.normalize_or_zero() * CAMERA_SPEED * time.delta_secs()).extend(0.0);
}
