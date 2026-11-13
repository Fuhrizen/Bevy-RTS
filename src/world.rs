//! Minimal playable-world scaffold. Gameplay systems can grow into separate plugins.

use bevy::prelude::*;

const COLUMNS: u32 = 18;
const ROWS: u32 = 12;
const CELL_SIZE: f32 = 50.0;
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

    for row in 0..ROWS {
        for column in 0..COLUMNS {
            let position = tile_position(column, row);
            let shade = if (row + column) % 2 == 0 { 0.20 } else { 0.23 };
            commands.spawn((
                Sprite::from_color(
                    Color::srgb(0.12, shade, 0.16),
                    Vec2::splat(CELL_SIZE - 2.0),
                ),
                Transform::from_xyz(position.x, position.y, 0.0),
            ));
        }
    }

    for (column, row, color) in [
        (3, 5, Color::srgb(0.24, 0.60, 0.92)),
        (4, 5, Color::srgb(0.24, 0.60, 0.92)),
        (13, 6, Color::srgb(0.93, 0.55, 0.24)),
        (14, 6, Color::srgb(0.93, 0.55, 0.24)),
    ] {
        let position = tile_position(column, row);
        commands.spawn((
            Unit,
            Sprite::from_color(color, Vec2::splat(CELL_SIZE * 0.60)),
            Transform::from_xyz(position.x, position.y, 1.0),
        ));
    }
}

fn tile_position(column: u32, row: u32) -> Vec2 {
    Vec2::new(
        (column as f32 - (COLUMNS - 1) as f32 / 2.0) * CELL_SIZE,
        (row as f32 - (ROWS - 1) as f32 / 2.0) * CELL_SIZE,
    )
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

    camera.translation += (movement.normalize_or_zero() * CAMERA_SPEED * time.delta_secs())
        .extend(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_is_centred_around_origin() {
        assert_eq!(tile_position(0, 0), Vec2::new(-425.0, -275.0));
        assert_eq!(tile_position(COLUMNS - 1, ROWS - 1), Vec2::new(425.0, 275.0));
    }

    #[test]
    fn neighbouring_tiles_are_spaced_evenly() {
        let origin = tile_position(3, 5);
        let next = tile_position(4, 5);
        assert_eq!(next - origin, Vec2::new(CELL_SIZE, 0.0));
    }
}
