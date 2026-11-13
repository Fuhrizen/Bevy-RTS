//! Tactical map, unit symbology and direct battlefield controls.
use crate::model::{
    Battle, CAPTURE_RADIUS, HudBlock, MAP_HALF, SUPPLY_RADIUS, Screen, Settings, Team,
};
use bevy::input::mouse::MouseWheel;
use bevy::window::PrimaryWindow;
use bevy::{picking::hover::Hovered, prelude::*, text::FontSize};

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PointerState>()
            .add_systems(Startup, setup_world)
            .add_systems(
                OnEnter(Screen::Battle),
                (enter_map, create_labels.after(crate::battle::setup_battle)),
            )
            .add_systems(OnExit(Screen::Battle), leave_map)
            .add_systems(
                Update,
                (camera_controls, pointer_controls, update_labels, draw_map)
                    .run_if(in_state(Screen::Battle)),
            );
    }
}

#[derive(Component)]
struct MapEntity;
#[derive(Resource)]
struct Terrain(Handle<Image>);
#[derive(Component)]
struct SquadLabel(usize);
#[derive(Component)]
struct ObjectiveLabel;
#[derive(Resource, Default)]
struct PointerState {
    start: Option<Vec2>,
    current: Option<Vec2>,
    dragged: bool,
    groups: [Vec<usize>; 4],
}

fn setup_world(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);
    commands.insert_resource(Terrain(images.add(crate::terrain::relief_image())));
}

fn enter_map(
    mut commands: Commands,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    mut pointer: ResMut<PointerState>,
    terrain: Res<Terrain>,
) {
    *pointer = PointerState::default();
    if let Ok((mut transform, mut projection)) = camera.single_mut() {
        transform.translation.x = 0.;
        transform.translation.y = 0.;
        if let Projection::Orthographic(ref mut p) = *projection {
            p.scale = 1.85;
        }
    }
    commands.spawn((
        MapEntity,
        Sprite {
            custom_size: Some(MAP_HALF * 2.),
            ..Sprite::from_image(terrain.0.clone())
        },
        Transform::from_xyz(0., 0., -10.),
    ));
}
fn leave_map(mut commands: Commands, entities: Query<Entity, With<MapEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
fn create_labels(mut commands: Commands, battle: Res<Battle>) {
    for (i, squad) in battle.squads.iter().enumerate() {
        commands.spawn((
            MapEntity,
            SquadLabel(i),
            Text2d::new(squad.callsign.clone()),
            TextFont {
                font_size: FontSize::Px(16.),
                ..default()
            },
            TextColor(Color::srgb(0.87, 0.92, 0.82)),
            Transform::from_xyz(squad.pos.x, squad.pos.y - 40., 5.),
        ));
    }
    for team in [Team::Blue, Team::Red] {
        let center = team.spawn();
        commands.spawn((
            MapEntity,
            Text2d::new(format!("{} Supply", team.label())),
            TextFont {
                font_size: FontSize::Px(20.),
                ..default()
            },
            TextColor(team.color()),
            Transform::from_xyz(center.x, center.y + 115., 5.),
        ));
    }
    for objective in &battle.objectives {
        commands.spawn((
            MapEntity,
            ObjectiveLabel,
            Text2d::new(objective.name),
            TextFont {
                font_size: FontSize::Px(19.),
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.88, 0.74)),
            Transform::from_xyz(objective.pos.x, objective.pos.y + 102., 5.),
        ));
    }
}
fn update_labels(
    battle: Res<Battle>,
    mut squads: Query<
        (&SquadLabel, &mut Transform, &mut Visibility, &mut Text2d),
        Without<ObjectiveLabel>,
    >,
) {
    for (label, mut transform, mut visibility, mut text) in &mut squads {
        if let Some(squad) = battle.squads.get(label.0) {
            let activity = if squad.supplies < 1.0 {
                "OUT OF SUPPLY"
            } else if squad
                .soldiers
                .iter()
                .any(|soldier| soldier.cooldown > 0.0 && soldier.hp > 0.0)
            {
                "ENGAGED"
            } else if !squad.orders.is_empty() {
                "MOVING"
            } else if squad.entrenched {
                "DUG IN"
            } else {
                "HOLDING"
            };
            let label_text = format!("{}  /  {}", squad.callsign, activity);
            if text.0 != label_text {
                text.0 = label_text;
            }
            transform.translation.x = squad.pos.x;
            transform.translation.y = squad.pos.y - 40.;
            *visibility = if squad.alive() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
fn camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    settings: Res<Settings>,
    hud: Query<&Hovered, With<HudBlock>>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let Ok((mut transform, mut projection)) = cameras.single_mut() else {
        return;
    };
    let mut direction = Vec2::ZERO;
    if (keys.pressed(KeyCode::KeyA)
        && !keys.pressed(KeyCode::ControlLeft)
        && !keys.pressed(KeyCode::ControlRight))
        || keys.pressed(KeyCode::ArrowLeft)
    {
        direction.x -= 1.;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.;
    }
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.;
    }
    let blocked = hud.iter().any(Hovered::get);
    if settings.edge_pan
        && !blocked
        && let Ok(window) = windows.single()
        && let Some(p) = window.cursor_position()
    {
        if p.x < 12. {
            direction.x -= 1.;
        }
        if p.x > window.width() - 12. {
            direction.x += 1.;
        }
        if p.y < 12. {
            direction.y += 1.;
        }
        if p.y > window.height() - 12. {
            direction.y -= 1.;
        }
    }
    let mut scale = 1.;
    if let Projection::Orthographic(ref mut p) = *projection {
        if !blocked {
            for event in wheel.read() {
                p.scale = (p.scale * (1. - event.y * 0.1)).clamp(0.65, 2.4);
            }
        } else {
            wheel.clear();
        }
        scale = p.scale;
    }
    transform.translation +=
        (direction.normalize_or_zero() * 520. * scale * time.delta_secs()).extend(0.);
    transform.translation.x = transform.translation.x.clamp(-MAP_HALF.x, MAP_HALF.x);
    transform.translation.y = transform.translation.y.clamp(-MAP_HALF.y, MAP_HALF.y);
}
fn pointer_controls(
    mut battle: ResMut<Battle>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    hud: Query<&Hovered, With<HudBlock>>,
    mut pointer: ResMut<PointerState>,
) {
    if battle.paused || battle.winner.is_some() || battle.draw {
        pointer.start = None;
        pointer.current = None;
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let blocked = hud.iter().any(Hovered::get);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl && keys.just_pressed(KeyCode::KeyA) {
        let all: Vec<_> = battle
            .squads
            .iter()
            .enumerate()
            .filter(|(_, s)| s.team == Team::Blue && s.alive())
            .map(|(i, _)| i)
            .collect();
        battle.selected.clear();
        battle.selected.extend(all);
    }
    for (slot, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ]
    .into_iter()
    .enumerate()
    {
        if keys.just_pressed(key) {
            if ctrl {
                pointer.groups[slot] = battle.selected.iter().copied().collect();
            } else {
                let group: Vec<_> = pointer.groups[slot]
                    .iter()
                    .copied()
                    .filter(|&i| battle.squads.get(i).is_some_and(|s| s.alive()))
                    .collect();
                if !shift {
                    battle.selected.clear();
                }
                battle.selected.extend(group);
            }
        }
    }
    let Some(world) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    else {
        if !buttons.pressed(MouseButton::Left) {
            pointer.start = None;
            pointer.current = None;
        }
        return;
    };
    if buttons.just_pressed(MouseButton::Left) && !blocked {
        pointer.start = Some(world);
        pointer.current = Some(world);
        pointer.dragged = false;
    }
    if let Some(start) = pointer.start
        && buttons.pressed(MouseButton::Left)
    {
        pointer.current = Some(world);
        pointer.dragged |= start.distance(world) > 12.;
    }
    if buttons.just_released(MouseButton::Left) {
        if let Some(start) = pointer.start.take() {
            if !shift {
                battle.selected.clear();
            }
            if pointer.dragged {
                let min = start.min(world);
                let max = start.max(world);
                let selected: Vec<usize> = battle
                    .squads
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| {
                        s.team == Team::Blue
                            && s.alive()
                            && s.pos.cmpge(min).all()
                            && s.pos.cmple(max).all()
                    })
                    .map(|(i, _)| i)
                    .collect();
                battle.selected.extend(selected);
            } else if let Some((i, _)) = battle
                .squads
                .iter()
                .enumerate()
                .filter(|(_, s)| s.team == Team::Blue && s.alive() && s.pos.distance(world) < 35.)
                .min_by(|(_, a), (_, b)| {
                    a.pos
                        .distance_squared(world)
                        .total_cmp(&b.pos.distance_squared(world))
                })
            {
                battle.selected.insert(i);
            }
        }
        pointer.current = None;
    }
    if buttons.just_pressed(MouseButton::Right) && !blocked && !battle.selected.is_empty() {
        battle.order_count += 1;
        let target = battle
            .squads
            .iter()
            .enumerate()
            .filter(|(_, s)| s.team == Team::Red && s.alive() && s.pos.distance(world) < 42.)
            .min_by(|(_, a), (_, b)| {
                a.pos
                    .distance_squared(world)
                    .total_cmp(&b.pos.distance_squared(world))
            })
            .map(|(i, _)| i);
        if let Some(target) = target {
            let enemy = battle.squads[target].pos;
            let Battle {
                selected, squads, ..
            } = &mut *battle;
            for &i in selected.iter() {
                if let Some(squad) = squads.get_mut(i)
                    && squad.team == Team::Blue
                    && squad.alive()
                {
                    if !shift {
                        squad.orders.clear();
                    }
                    squad.focus = Some(target);
                    squad.entrenched = false;
                    let direction = (squad.pos - enemy).normalize_or_zero();
                    if squad.pos.distance(enemy) > 200.0 {
                        squad
                            .orders
                            .push_back((enemy + direction * 150.).clamp(-MAP_HALF, MAP_HALF));
                    }
                }
            }
        } else {
            crate::battle::issue_move(&mut battle, world, shift);
        }
    }
}
fn draw_map(mut gizmos: Gizmos, battle: Res<Battle>, pointer: Res<PointerState>) {
    let grid = Color::srgba(0.60, 0.68, 0.52, 0.10);
    for x in -11..=11 {
        let x = x as f32 * 100.;
        gizmos.line_2d(Vec2::new(x, -MAP_HALF.y), Vec2::new(x, MAP_HALF.y), grid);
    }
    for y in -7..=7 {
        let y = y as f32 * 100.;
        gizmos.line_2d(Vec2::new(-MAP_HALF.x, y), Vec2::new(MAP_HALF.x, y), grid);
    }
    // A restrained route across the map gives the landscape a readable axis.
    let road = [
        Vec2::new(-1120., -210.),
        Vec2::new(-760., -160.),
        Vec2::new(-420., -60.),
        Vec2::new(-120., -100.),
        Vec2::new(230., 25.),
        Vec2::new(550., 50.),
        Vec2::new(830., 170.),
        Vec2::new(1120., 190.),
    ];
    for pair in road.windows(2) {
        gizmos.line_2d(pair[0], pair[1], Color::srgba(0.66, 0.62, 0.46, 0.30));
    }
    for team in [Team::Blue, Team::Red] {
        let p = team.spawn();
        gizmos.circle_2d(p, SUPPLY_RADIUS, team.color().with_alpha(0.25));
        gizmos.circle_2d(p, SUPPLY_RADIUS + 5., team.color().with_alpha(0.11));
        let cross = p + Vec2::new(0., 32.);
        gizmos.line_2d(cross - Vec2::X * 11., cross + Vec2::X * 11., team.color());
        gizmos.line_2d(cross - Vec2::Y * 11., cross + Vec2::Y * 11., team.color());
    }
    for objective in &battle.objectives {
        let color = objective
            .owner
            .map(|t| t.color())
            .unwrap_or(Color::srgb(0.70, 0.72, 0.60));
        gizmos.circle_2d(objective.pos, CAPTURE_RADIUS, color.with_alpha(0.32));
        if objective.contested {
            gizmos.circle_2d(
                objective.pos,
                CAPTURE_RADIUS + 5.,
                Color::srgba(1., 0.72, 0.31, 0.8),
            );
        }
        let bar = objective.pos + Vec2::new(0., -96.);
        gizmos.line_2d(
            bar + Vec2::new(-35., 0.),
            bar + Vec2::new(35., 0.),
            Color::srgba(0.7, 0.73, 0.63, 0.35),
        );
        let capture = objective.capture.clamp(-1., 1.);
        gizmos.line_2d(
            bar,
            bar + Vec2::new(capture * 35., 0.),
            if capture >= 0. {
                Team::Blue.color()
            } else {
                Team::Red.color()
            },
        );
        gizmos.circle_2d(objective.pos, 14., color.with_alpha(0.85));
        gizmos.line_2d(
            objective.pos + Vec2::new(-6., 0.),
            objective.pos + Vec2::new(6., 0.),
            color,
        );
        gizmos.line_2d(
            objective.pos + Vec2::new(0., -6.),
            objective.pos + Vec2::new(0., 6.),
            color,
        );
    }
    for effect in &battle.effects {
        let alpha = (effect.ttl / 0.24).clamp(0.0, 1.0);
        let direction = (effect.to - effect.from).normalize_or_zero();
        let start = effect.from + direction * 28.0;
        let end = effect.to - direction * 28.0;
        gizmos.line_2d(start, end, effect.team.color().with_alpha(alpha * 0.9));
        gizmos.circle_2d(
            effect.from,
            12.0 + (1.0 - alpha) * 10.0,
            Color::srgba(1.0, 0.8, 0.4, alpha),
        );
        gizmos.circle_2d(
            effect.to,
            8.0 + (1.0 - alpha) * 12.0,
            Color::srgba(1.0, 0.6, 0.3, alpha),
        );
    }
    for (i, squad) in battle.squads.iter().enumerate() {
        if !squad.alive() {
            continue;
        }
        let p = squad.pos;
        let selected = battle.selected.contains(&i);
        let color = squad.team.color();
        if squad.smoke > 0.0 {
            gizmos.circle_2d(p, 48., Color::srgba(0.84, 0.88, 0.88, 0.48));
            gizmos.circle_2d(p, 54., Color::srgba(0.84, 0.88, 0.88, 0.25));
        }
        if selected {
            gizmos.circle_2d(p, 36., Color::srgb(0.95, 0.88, 0.48));
        }
        let background = Color::srgba(0.025, 0.045, 0.045, 0.92);
        for y in -17..=17 {
            gizmos.line_2d(
                p + Vec2::new(-25., y as f32),
                p + Vec2::new(25., y as f32),
                background,
            );
        }
        let a = p + Vec2::new(-23., -15.);
        let b = p + Vec2::new(23., 15.);
        let border = if selected {
            Color::srgb(1., 0.91, 0.54)
        } else {
            color
        };
        rect(&mut gizmos, a, b, border);
        gizmos.line_2d(a, b, color.with_alpha(0.9));
        gizmos.line_2d(
            Vec2::new(a.x, b.y),
            Vec2::new(b.x, a.y),
            color.with_alpha(0.9),
        );
        for y in 17..=24 {
            gizmos.line_2d(
                p + Vec2::new(-11., y as f32),
                p + Vec2::new(11., y as f32),
                color,
            );
        }
        rect(
            &mut gizmos,
            p + Vec2::new(-26., -18.),
            p + Vec2::new(26., 18.),
            border.with_alpha(0.65),
        );
        if squad.entrenched {
            gizmos.line_2d(
                p + Vec2::new(-29., -20.),
                p + Vec2::new(29., -20.),
                Color::srgb(0.85, 0.78, 0.50),
            );
        }
        gizmos.line_2d(
            p + Vec2::new(-23., -22.),
            p + Vec2::new(23., -22.),
            Color::srgba(0.2, 0.25, 0.2, 0.9),
        );
        gizmos.line_2d(
            p + Vec2::new(-23., -22.),
            p + Vec2::new(
                -23. + 46. * (squad.manpower() as f32 / squad.soldiers.len() as f32).clamp(0., 1.),
                -22.,
            ),
            Color::srgb(0.56, 0.78, 0.50),
        );
        gizmos.line_2d(
            p + Vec2::new(-23., -27.),
            p + Vec2::new(-23. + 46. * (squad.supplies / 100.).clamp(0., 1.), -27.),
            Color::srgb(0.84, 0.72, 0.42),
        );
        if selected {
            let mut previous = p;
            for point in &squad.orders {
                gizmos.line_2d(previous, *point, Color::srgba(0.97, 0.86, 0.46, 0.7));
                gizmos.circle_2d(*point, 6., Color::srgb(0.97, 0.86, 0.46));
                previous = *point;
            }
        }
    }
    if let (Some(a), Some(b)) = (pointer.start, pointer.current)
        && pointer.dragged
    {
        rect(
            &mut gizmos,
            a.min(b),
            a.max(b),
            Color::srgb(0.94, 0.88, 0.55),
        );
    }
}
fn rect(g: &mut Gizmos, a: Vec2, b: Vec2, color: Color) {
    g.line_2d(a, Vec2::new(b.x, a.y), color);
    g.line_2d(Vec2::new(b.x, a.y), b, color);
    g.line_2d(b, Vec2::new(a.x, b.y), color);
    g.line_2d(Vec2::new(a.x, b.y), a, color);
}
