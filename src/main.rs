mod ai;
mod battle;
mod interface;
mod model;
mod terrain;
mod world;

use bevy::{
    prelude::*,
    ui::UiScale,
    window::{MonitorSelection, PrimaryWindow, WindowMode, WindowResizeConstraints},
};
use model::{Battle, Roster, Screen, Settings};

fn main() {
    let mut app = App::new();
    configure(&mut app);
    app.run();
}

fn configure(app: &mut App) {
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "BEV Front".into(),
            resolution: (1280, 800).into(),
            resize_constraints: WindowResizeConstraints {
                min_width: 900.0,
                min_height: 620.0,
                ..default()
            },
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.055, 0.080, 0.078)))
    .init_state::<Screen>()
    .init_resource::<Settings>()
    .init_resource::<Roster>()
    .init_resource::<Battle>()
    .add_plugins((
        battle::BattlePlugin,
        world::WorldPlugin,
        interface::UiPlugin,
    ))
    .add_systems(Update, apply_display_settings);
}

fn apply_display_settings(
    settings: Res<Settings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
) {
    if !settings.is_changed() {
        return;
    }

    if let Ok(mut window) = windows.single_mut() {
        window.mode = if settings.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
    }
    ui_scale.0 = settings.ui_scale;
}
