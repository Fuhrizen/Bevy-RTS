use crate::model::{Battle, Difficulty, HudBlock, Kit, Pace, Roster, Screen, Settings, Team};
use crate::operation::{CONTROL_TARGET, Finish, Mode, PERSONNEL_LIMIT, TIMED_LIMIT};
use bevy::{
    ecs::system::SystemParam,
    picking::hover::Hovered,
    prelude::*,
    text::FontSize,
    ui::Pressed,
    ui_widgets::{Activate, Button},
};

pub struct UiPlugin;

#[derive(Component)]
struct UiRoot;
#[derive(Component, Clone, Copy)]
enum Action {
    Play,
    Settings,
    BackMenu,
    Deploy,
    Resume,
    Restart,
    Exit,
    Difficulty,
    Fullscreen,
    EdgePan,
    Scale,
    Callsign(usize),
    Map,
    Mode,
    Size(usize),
    Kit(usize),
    Pause,
    Hold,
    FireControl,
    Pace(Pace),
    Entrench,
    Smoke,
    Resupply,
    Select(usize),
}
#[derive(Message)]
struct QueuedAction(Action);

#[derive(Component, Clone, Copy)]
enum HudText {
    Tally,
    Objective(usize),
    Squad(usize),
    Selected,
    Detail,
    Soldier(usize),
    Equipment,
    Pace,
    CommandHint,
    Pause,
}
fn dynamic(p: &mut ChildSpawnerCommands, marker: HudText, size: f32, color: Color) {
    p.spawn((
        marker,
        Text::new(""),
        Pickable::IGNORE,
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    ));
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<QueuedAction>()
            .add_systems(OnEnter(Screen::Menu), build_menu)
            .add_systems(OnEnter(Screen::Settings), build_settings)
            .add_systems(OnEnter(Screen::Roster), build_roster)
            .add_systems(
                OnEnter(Screen::Battle),
                build_battle.after(crate::battle::setup_battle),
            )
            .add_systems(OnExit(Screen::Menu), clear_ui)
            .add_systems(OnExit(Screen::Settings), clear_ui)
            .add_systems(OnExit(Screen::Roster), clear_ui)
            .add_systems(OnExit(Screen::Battle), clear_ui)
            .add_systems(
                Update,
                (
                    button_feedback,
                    actions,
                    refresh_battle,
                    update_hud,
                    update_rings,
                )
                    .chain(),
            );
    }
}

const INK: Color = Color::srgb(0.87, 0.90, 0.84);
const MUTED: Color = Color::srgb(0.56, 0.64, 0.60);
const PANEL: Color = Color::srgba(0.035, 0.055, 0.055, 0.94);
const BLUE: Color = Color::srgb(0.34, 0.77, 0.86);
const GOLD: Color = Color::srgb(0.90, 0.72, 0.39);

fn clear_ui(mut commands: Commands, roots: Query<Entity, With<UiRoot>>) {
    for e in &roots {
        commands.entity(e).despawn();
    }
}
fn text(parent: &mut ChildSpawnerCommands, value: impl Into<String>, size: f32, color: Color) {
    parent.spawn((
        Text::new(value),
        Pickable::IGNORE,
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    ));
}
fn button(parent: &mut ChildSpawnerCommands, label: impl Into<String>, action: Action, width: f32) {
    parent
        .spawn((
            Button,
            action,
            HudBlock,
            Hovered::default(),
            Node {
                min_width: px(width),
                min_height: px(35),
                padding: UiRect::axes(px(12), px(7)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.25, 0.36, 0.35)),
            BackgroundColor(Color::srgb(0.10, 0.15, 0.14)),
        ))
        .observe(
            move |_: On<Activate>, mut queue: MessageWriter<QueuedAction>| {
                queue.write(QueuedAction(action));
            },
        )
        .with_children(|p| {
            if let Action::Select(i) = action {
                dynamic(p, HudText::Squad(i), 11., INK);
            } else if let Action::Pause = action {
                dynamic(p, HudText::Pause, 12., INK);
            } else {
                text(p, label, 12., INK);
            }
        });
}
fn root(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            UiRoot,
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .id()
}
fn panel() -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(18)),
        row_gap: px(10),
        border: UiRect::all(px(1)),
        ..default()
    }
}
fn block(
    commands: &mut Commands,
    root: Entity,
    node: Node,
    f: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let child = commands
        .spawn((
            node,
            HudBlock,
            Hovered::default(),
            BackgroundColor(PANEL),
            BorderColor::all(Color::srgb(0.23, 0.33, 0.32)),
        ))
        .with_children(f)
        .id();
    commands.entity(root).add_child(child);
}
fn button_feedback(
    mut buttons: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<Button>>,
) {
    for (hovered, pressed, mut background) in &mut buttons {
        let color = if pressed {
            Color::srgb(0.25, 0.39, 0.36)
        } else if hovered.get() {
            Color::srgb(0.16, 0.25, 0.23)
        } else {
            Color::srgb(0.10, 0.15, 0.14)
        };
        background.set_if_neq(BackgroundColor(color));
    }
}
fn build_menu(mut commands: Commands) {
    let r = root(&mut commands);
    block(
        &mut commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            left: percent(7),
            top: percent(16),
            width: px(405),
            ..panel()
        },
        |p| {
            text(p, "BEV FRONT", 54., INK);
            text(p, "SECTOR CONTROL", 15., GOLD);
            text(
                p,
                "Lead four squads and capture the objectives.",
                16.,
                MUTED,
            );
            p.spawn((
                Pickable::IGNORE,
                Node {
                    height: px(15),
                    ..default()
                },
            ));
            button(p, "PLAY", Action::Play, 210.);
            button(p, "SETTINGS", Action::Settings, 210.);
        },
    );
}
fn build_settings(mut commands: Commands, settings: Res<Settings>) {
    let r = root(&mut commands);
    settings_panel(&mut commands, r, &settings);
}
fn settings_panel(commands: &mut Commands, r: Entity, settings: &Settings) {
    block(
        commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            left: percent(8),
            top: percent(12),
            width: px(460),
            ..panel()
        },
        |p| {
            text(p, "CONFIGURATION", 12., BLUE);
            text(p, "SETTINGS", 34., INK);
            text(p, "Adjust the operation to your preferences.", 14., MUTED);
            button(
                p,
                format!("DIFFICULTY    {}", settings.difficulty.label()),
                Action::Difficulty,
                300.,
            );
            button(
                p,
                format!(
                    "FULLSCREEN    {}",
                    if settings.fullscreen { "ON" } else { "OFF" }
                ),
                Action::Fullscreen,
                300.,
            );
            button(
                p,
                format!(
                    "EDGE PAN    {}",
                    if settings.edge_pan { "ON" } else { "OFF" }
                ),
                Action::EdgePan,
                300.,
            );
            button(
                p,
                format!("UI SCALE    {:.0}%", settings.ui_scale * 100.),
                Action::Scale,
                300.,
            );
            button(p, "BACK", Action::BackMenu, 150.);
        },
    );
}
fn build_roster(mut commands: Commands, roster: Res<Roster>) {
    let r = root(&mut commands);
    roster_panel(&mut commands, r, &roster);
}
fn roster_panel(commands: &mut Commands, r: Entity, roster: &Roster) {
    block(
        commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            left: percent(5),
            top: percent(6),
            width: px(620),
            ..panel()
        },
        |p| {
            text(p, "BLUE FORCE  /  MISSION PREPARATION", 12., BLUE);
            text(p, "ASSEMBLE YOUR FORCE", 30., INK);
            p.spawn((
                Pickable::IGNORE,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8),
                    ..default()
                },
            ))
            .with_children(|row| {
                button(
                    row,
                    format!("MAP  {}", roster.map.label()),
                    Action::Map,
                    180.,
                );
                button(
                    row,
                    format!("MODE  {}", roster.mode.label()),
                    Action::Mode,
                    180.,
                );
            });
            text(p, roster.mode.rules(), 13., MUTED);
            text(
                p,
                format!("PERSONNEL  {}/{}", roster.personnel(), PERSONNEL_LIMIT),
                13.,
                BLUE,
            );
            for (i, squad) in roster.squads.iter().enumerate() {
                p.spawn((
                    Pickable::IGNORE,
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: px(6),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|row| {
                    text(row, format!("{:02}", i + 1), 14., BLUE);
                    button(row, squad.callsign.as_str(), Action::Callsign(i), 145.);
                    button(
                        row,
                        format!("{} TROOPS", squad.manpower),
                        Action::Size(i),
                        130.,
                    );
                    button(row, squad.kit.label(), Action::Kit(i), 130.);
                });
            }
            p.spawn((
                Pickable::IGNORE,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8),
                    margin: UiRect::top(px(8)),
                    ..default()
                },
            ))
            .with_children(|row| {
                button(row, "BACK", Action::BackMenu, 135.);
                button(row, "DEPLOY FORCE", Action::Deploy, 190.);
            });
        },
    );
}
fn build_battle(mut commands: Commands, battle: Res<Battle>) {
    let r = root(&mut commands);
    battle_panels(&mut commands, r, &battle);
}
fn battle_panels(commands: &mut Commands, r: Entity, battle: &Battle) {
    block(
        commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            margin: UiRect::left(px(-190)),
            top: px(12),
            width: px(380),
            padding: UiRect::all(px(10)),
            ..panel()
        },
        |p| {
            dynamic(p, HudText::Tally, 16., INK);
            p.spawn((
                Pickable::IGNORE,
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: px(5),
                    ..default()
                },
            ))
            .with_children(|row| {
                for i in 0..battle.objectives.len() {
                    dynamic(row, HudText::Objective(i), 11., INK);
                }
            });
        },
    );
    block(
        commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            right: px(14),
            top: px(12),
            padding: UiRect::all(px(8)),
            ..panel()
        },
        |p| {
            dynamic(p, HudText::Pace, 11., BLUE);
            p.spawn((
                Pickable::IGNORE,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4),
                    ..default()
                },
            ))
            .with_children(|row| {
                for pace in [Pace::Planning, Pace::Deliberate, Pace::Normal] {
                    button(row, pace.label(), Action::Pace(pace), 48.);
                }
            });
            button(p, "MENU", Action::Pause, 88.);
        },
    );
    block(
        commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            bottom: px(12),
            width: percent(23),
            padding: UiRect::all(px(10)),
            row_gap: px(5),
            ..panel()
        },
        |p| {
            text(p, "SQUADS", 11., BLUE);
            for (i, _) in battle
                .squads
                .iter()
                .enumerate()
                .filter(|(_, s)| s.team == Team::Blue)
            {
                button(p, "", Action::Select(i), 0.);
            }
        },
    );
    block(
        commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            left: percent(25),
            bottom: px(12),
            width: percent(48),
            padding: UiRect::all(px(10)),
            row_gap: px(5),
            ..panel()
        },
        |p| {
            dynamic(p, HudText::Selected, 14., GOLD);
            dynamic(p, HudText::Detail, 12., INK);
            dynamic(p, HudText::Equipment, 11., BLUE);
            p.spawn((
                Pickable::IGNORE,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8),
                    ..default()
                },
            ))
            .with_children(|row| {
                for col in 0..2 {
                    row.spawn((
                        Pickable::IGNORE,
                        Node {
                            width: percent(49),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(3),
                            ..default()
                        },
                    ))
                    .with_children(|column| {
                        for i in 0..5 {
                            soldier_row(column, col * 5 + i);
                        }
                    });
                }
            });
        },
    );
    block(
        commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            right: px(12),
            bottom: px(12),
            width: percent(24),
            padding: UiRect::all(px(10)),
            row_gap: px(5),
            ..panel()
        },
        |p| {
            text(p, "ORDERS", 11., BLUE);
            p.spawn((
                Pickable::IGNORE,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(5),
                    ..default()
                },
            ))
            .with_children(|row| {
                button(row, "HOLD", Action::Hold, 68.);
                button(row, "DIG IN", Action::Entrench, 68.);
            });
            p.spawn((
                Pickable::IGNORE,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(5),
                    ..default()
                },
            ))
            .with_children(|row| {
                button(row, "SMOKE", Action::Smoke, 68.);
                button(row, "RESUPPLY", Action::Resupply, 68.);
            });
            button(p, "TOGGLE HOLD FIRE", Action::FireControl, 90.);
            dynamic(p, HudText::CommandHint, 10., MUTED);
        },
    );
    if battle.paused || battle.winner.is_some() || battle.draw {
        block(
            commands,
            r,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                margin: UiRect::left(px(-185)),
                top: percent(15),
                width: px(370),
                ..panel()
            },
            |p| {
                text(p, "OPERATION STATUS", 11., BLUE);
                text(
                    p,
                    if battle.draw {
                        "DRAW"
                    } else if let Some(winner) = battle.winner {
                        if winner == Team::Blue {
                            "VICTORY"
                        } else {
                            "DEFEAT"
                        }
                    } else {
                        "PAUSED"
                    },
                    36.,
                    INK,
                );
                if battle.winner.is_some() || battle.draw {
                    let seconds = battle.elapsed as u32;
                    let winner = battle.winner.map_or("", Team::label);
                    let reason = match battle.finish {
                        Some(Finish::Score) => format!("{winner} reached the control target."),
                        Some(Finish::Time) if battle.draw => {
                            "Time expired with scores level.".into()
                        }
                        Some(Finish::Time) => format!("{winner} led when time expired."),
                        Some(Finish::Elimination) if battle.draw => {
                            "Both forces were wiped out.".into()
                        }
                        Some(Finish::Elimination) => {
                            format!("{winner} eliminated the opposing force.")
                        }
                        _ => format!(
                            "{winner} secured all {} objectives.",
                            battle.objectives.len()
                        ),
                    };
                    text(
                        p,
                        format!("{reason}  {:02}:{:02}", seconds / 60, seconds % 60),
                        13.,
                        MUTED,
                    );
                    if battle.mode != Mode::Conquest {
                        text(
                            p,
                            format!(
                                "CONTROL SCORE  {:.0} BLUE / {:.0} RED",
                                battle.score[0].floor(),
                                battle.score[1].floor()
                            ),
                            12.,
                            GOLD,
                        );
                    }
                    for team in [Team::Blue, Team::Red] {
                        let report = battle.report(team);
                        text(
                            p,
                            format!(
                                "{}  |  {} held  |  {} squads  |  {}/{} troops",
                                team.label(),
                                report.objectives,
                                report.squads_remaining,
                                report.soldiers_remaining,
                                report.soldiers_deployed
                            ),
                            12.,
                            team.color(),
                        );
                    }
                } else {
                    button(p, "RESUME", Action::Resume, 210.);
                    text(
                        p,
                        "Drag select  /  Ctrl+A select all\nShift + right-click queues waypoints\nCtrl+1-4 set group  /  1-4 recall\nWASD or arrows pan  /  wheel zoom\nSpace toggles planning pause",
                        12.,
                        MUTED,
                    );
                }
                button(p, "BACK TO LOADOUT", Action::Restart, 210.);
                button(p, "EXIT TO MENU", Action::Exit, 210.);
            },
        );
    }
}
#[derive(SystemParam)]
struct Menus<'w, 's> {
    commands: Commands<'w, 's>,
    roots: Query<'w, 's, Entity, With<UiRoot>>,
    screen: Res<'w, State<Screen>>,
    next: ResMut<'w, NextState<Screen>>,
}

fn actions(
    menus: Menus,
    mut actions: MessageReader<QueuedAction>,
    mut settings: ResMut<Settings>,
    mut roster: ResMut<Roster>,
    mut battle: ResMut<Battle>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Menus {
        mut commands,
        roots,
        screen,
        mut next,
    } = menus;
    if keys.just_pressed(KeyCode::Escape)
        && *screen.get() == Screen::Battle
        && battle.winner.is_none()
        && !battle.draw
    {
        battle.paused = !battle.paused;
    }
    if *screen.get() == Screen::Battle
        && !battle.paused
        && battle.winner.is_none()
        && !battle.draw
        && keys.just_pressed(KeyCode::Space)
    {
        let pace = if battle.pace == Pace::Planning {
            battle.resume_pace
        } else {
            Pace::Planning
        };
        battle.set_pace(pace);
    }
    let mut redraw = false;
    for QueuedAction(action) in actions.read() {
        if (battle.paused || battle.winner.is_some() || battle.draw)
            && matches!(
                *action,
                Action::Hold
                    | Action::Pace(_)
                    | Action::FireControl
                    | Action::Entrench
                    | Action::Smoke
                    | Action::Resupply
                    | Action::Select(_)
            )
        {
            continue;
        }
        match *action {
            Action::Play => next.set(Screen::Roster),
            Action::Settings => next.set(Screen::Settings),
            Action::BackMenu | Action::Exit => next.set(Screen::Menu),
            Action::Restart => next.set(Screen::Roster),
            Action::Deploy => next.set(Screen::Battle),
            Action::Resume => {
                battle.paused = false;
            }
            Action::Pause => {
                if battle.winner.is_none() && !battle.draw {
                    battle.paused = !battle.paused;
                }
            }
            Action::Difficulty => {
                settings.difficulty = match settings.difficulty {
                    Difficulty::Recruit => Difficulty::Standard,
                    Difficulty::Standard => Difficulty::Veteran,
                    Difficulty::Veteran => Difficulty::Recruit,
                };
                redraw = true;
            }
            Action::Fullscreen => {
                settings.fullscreen = !settings.fullscreen;
                redraw = true;
            }
            Action::EdgePan => {
                settings.edge_pan = !settings.edge_pan;
                redraw = true;
            }
            Action::Scale => {
                settings.ui_scale = if settings.ui_scale < 0.9 {
                    1.0
                } else if settings.ui_scale < 1.1 {
                    1.15
                } else {
                    0.85
                };
                redraw = true;
            }
            Action::Callsign(i) => {
                let names = [
                    "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Fox", "Ghost", "Viper", "Raven",
                    "Spear",
                ];
                let current = names
                    .iter()
                    .position(|name| *name == roster.squads[i].callsign)
                    .unwrap_or(0);
                roster.squads[i].callsign = names[(current + 1) % names.len()].into();
                redraw = true;
            }
            Action::Map => {
                roster.map = roster.map.next();
                redraw = true;
            }
            Action::Mode => {
                roster.mode = roster.mode.next();
                redraw = true;
            }
            Action::Size(i) => {
                roster.cycle_manpower(i);
                redraw = true;
            }
            Action::Kit(i) => {
                roster.squads[i].kit = match roster.squads[i].kit {
                    Kit::Rifle => Kit::Support,
                    Kit::Support => Kit::Scout,
                    Kit::Scout => Kit::Medic,
                    Kit::Medic => Kit::Rifle,
                };
                redraw = true;
            }
            Action::Hold => {
                let selected: Vec<_> = battle.selected.iter().copied().collect();
                for i in selected {
                    if let Some(s) = battle.squads.get_mut(i) {
                        s.orders.clear();
                        s.focus = None;
                    }
                }
            }
            Action::Pace(pace) => battle.set_pace(pace),
            Action::FireControl => crate::battle::toggle_fire_control(&mut battle),
            Action::Entrench => crate::battle::toggle_entrench(&mut battle),
            Action::Smoke => crate::battle::deploy_smoke(&mut battle),
            Action::Resupply => crate::battle::resupply_selected(&mut battle),
            Action::Select(i) => {
                if !battle.squads.get(i).is_some_and(|s| s.alive()) {
                    continue;
                }
                let additive =
                    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
                if !additive {
                    battle.selected.clear();
                }
                if additive && battle.selected.contains(&i) {
                    battle.selected.remove(&i);
                } else {
                    battle.selected.insert(i);
                }
            }
        }
    }
    if redraw {
        for e in &roots {
            commands.entity(e).despawn();
        }
        let r = root(&mut commands);
        match screen.get() {
            Screen::Settings => settings_panel(&mut commands, r, &settings),
            Screen::Roster => roster_panel(&mut commands, r, &roster),
            Screen::Battle => battle_panels(&mut commands, r, &battle),
            _ => {}
        }
    }
}
fn refresh_battle(
    screen: Res<State<Screen>>,
    battle: Res<Battle>,
    mut commands: Commands,
    roots: Query<Entity, With<UiRoot>>,
    mut previous: Local<(bool, bool, bool)>,
) {
    if *screen.get() != Screen::Battle {
        *previous = (false, false, false);
        return;
    }
    let status = (battle.paused, battle.winner.is_some(), battle.draw);
    if *previous != status {
        *previous = status;
        for e in &roots {
            commands.entity(e).despawn();
        }
        let r = root(&mut commands);
        battle_panels(&mut commands, r, &battle);
    }
}
fn update_hud(battle: Res<Battle>, mut labels: Query<(&HudText, &mut Text, &mut TextColor)>) {
    for (marker, mut text, mut color) in &mut labels {
        let value = match *marker {
            HudText::CommandHint => {
                let count = battle.selected.len();
                if count == 0 {
                    "SELECT A SQUAD TO GIVE ORDERS".into()
                } else {
                    let queued: usize = battle
                        .selected
                        .iter()
                        .filter_map(|&i| battle.squads.get(i))
                        .map(|squad| squad.orders.len())
                        .sum();
                    format!("{count} SELECTED  /  {queued} WAYPOINTS")
                }
            }
            HudText::Pace => {
                if battle.pace == Pace::Planning {
                    "PLANNING  /  ORDERS ENABLED".into()
                } else {
                    format!("SPEED  {}", battle.pace.label())
                }
            }
            HudText::Tally => {
                let seconds = if battle.mode == Mode::Timed {
                    (TIMED_LIMIT - battle.elapsed).max(0.0).ceil() as u32
                } else {
                    battle.elapsed as u32
                };
                let clock = format!("{:02}:{:02}", seconds / 60, seconds % 60);
                match battle.mode {
                    Mode::Conquest => format!(
                        "{clock}  |  {}/{} HELD",
                        battle.report(Team::Blue).objectives,
                        battle.objectives.len()
                    ),
                    Mode::Control => format!(
                        "{clock}  |  {:.0} - {:.0} / {CONTROL_TARGET:.0}",
                        battle.score[0].floor(),
                        battle.score[1].floor()
                    ),
                    Mode::Timed => format!(
                        "{clock}  |  BLUE {:.0} - {:.0} RED",
                        battle.score[0].floor(),
                        battle.score[1].floor()
                    ),
                }
            }
            HudText::Objective(i) => battle
                .objectives
                .get(i)
                .map(|o| {
                    color.set_if_neq(TextColor(o.display_color()));
                    format!(
                        "{}\n{}",
                        ["NW", "N", "NE", "SW", "S", "SE"][i],
                        o.capture_label()
                    )
                })
                .unwrap_or_default(),
            HudText::Squad(i) => battle
                .squads
                .get(i)
                .map(|s| {
                    format!(
                        "{} {}  {}/{}  {:.0}%",
                        if battle.selected.contains(&i) {
                            ">"
                        } else {
                            " "
                        },
                        s.callsign.to_uppercase(),
                        s.manpower(),
                        s.soldiers.len(),
                        s.supplies
                    )
                })
                .unwrap_or_default(),
            HudText::Selected => battle
                .selected
                .iter()
                .next()
                .and_then(|i| battle.squads.get(*i))
                .map(|s| {
                    format!(
                        "SELECTED  /  {}   /   {}",
                        s.callsign.to_uppercase(),
                        s.kit.label().to_uppercase()
                    )
                })
                .unwrap_or_else(|| "SELECT A SQUAD TO VIEW DETAILS".into()),
            HudText::Detail => battle
                .selected
                .iter()
                .next()
                .and_then(|i| battle.squads.get(*i))
                .map(|s| {
                    format!(
                        "{}/{} troops  |  {:.0}% supplies  |  {}",
                        s.manpower(),
                        s.soldiers.len(),
                        s.supplies,
                        s.marker_status().0
                    )
                })
                .unwrap_or_default(),
            HudText::Equipment => battle
                .selected
                .iter()
                .next()
                .and_then(|i| battle.squads.get(*i))
                .map(|s| {
                    match s.kit {
                        Kit::Rifle => "RIFLE KIT  /  SMOKE",
                        Kit::Support => "SUPPORT KIT  /  SMOKE",
                        Kit::Scout => "SCOUT KIT  /  SMOKE",
                        Kit::Medic => "MEDIC KIT  /  SMOKE",
                    }
                    .to_owned()
                })
                .unwrap_or_default(),
            HudText::Soldier(index) => battle
                .selected
                .iter()
                .next()
                .and_then(|i| battle.squads.get(*i))
                .and_then(|s| s.soldiers.get(index).map(|soldier| (s, soldier)))
                .map(|(s, soldier)| {
                    let state = if soldier.hp <= 0.0 {
                        "KIA"
                    } else if soldier.cooldown > 0.0 {
                        "COOLDOWN"
                    } else if !s.orders.is_empty() {
                        "MOVING"
                    } else if s.hold_fire {
                        "HOLD FIRE"
                    } else if s.entrenched {
                        "DUG IN"
                    } else {
                        "READY"
                    };
                    format!(
                        "{:02} {}  {}  {:.0}HP V{}",
                        soldier.id + 1,
                        soldier.role.label(),
                        state,
                        soldier.hp,
                        soldier.veterancy
                    )
                })
                .unwrap_or_default(),
            HudText::Pause => {
                if battle.paused {
                    "RESUME".into()
                } else {
                    "MENU".into()
                }
            }
        };
        if **text != value {
            **text = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(app: &mut App, choose: impl Fn(&Action) -> bool) {
        let world = app.world_mut();
        let mut buttons = world.query::<(Entity, &Action)>();
        let button = buttons
            .iter(world)
            .find(|(_, action)| choose(action))
            .unwrap()
            .0;
        world.trigger(Activate { entity: button });
        app.update();
        app.update();
    }

    #[test]
    fn menu_settings_roster_and_pause_flow() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .init_state::<Screen>()
            .init_resource::<Settings>()
            .init_resource::<Roster>()
            .init_resource::<Battle>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_plugins((crate::battle::BattlePlugin, UiPlugin));
        app.update();

        press(&mut app, |action| matches!(action, Action::Settings));
        assert_eq!(
            *app.world().resource::<State<Screen>>().get(),
            Screen::Settings
        );
        press(&mut app, |action| matches!(action, Action::Difficulty));
        assert_eq!(
            app.world().resource::<Settings>().difficulty,
            Difficulty::Veteran
        );
        press(&mut app, |action| matches!(action, Action::BackMenu));
        press(&mut app, |action| matches!(action, Action::Play));
        press(&mut app, |action| matches!(action, Action::Kit(0)));
        press(&mut app, |action| matches!(action, Action::Map));
        press(&mut app, |action| matches!(action, Action::Deploy));

        let battle = app.world().resource::<Battle>();
        assert_eq!(battle.squads.len(), 8);
        assert_eq!(battle.map, crate::operation::Map::Highmoor);
        assert_eq!(battle.squads[0].kit, Kit::Support);
        press(&mut app, |action| matches!(action, Action::Pause));
        assert!(app.world().resource::<Battle>().paused);
        press(&mut app, |action| matches!(action, Action::Resume));
        assert!(!app.world().resource::<Battle>().paused);
        press(&mut app, |action| matches!(action, Action::Select(0)));
        assert!(app.world().resource::<Battle>().selected.contains(&0));
        press(&mut app, |action| matches!(action, Action::FireControl));
        assert!(app.world().resource::<Battle>().squads[0].hold_fire);
        press(&mut app, |action| {
            matches!(action, Action::Pace(Pace::Planning))
        });
        assert_eq!(app.world().resource::<Battle>().pace, Pace::Planning);
        app.world_mut().resource_mut::<Battle>().squads[0]
            .orders
            .push_back(Vec2::ZERO);
        press(&mut app, |action| matches!(action, Action::Hold));
        assert!(app.world().resource::<Battle>().squads[0].orders.is_empty());
        press(&mut app, |action| {
            matches!(action, Action::Pace(Pace::Normal))
        });
        assert_eq!(app.world().resource::<Battle>().pace, Pace::Normal);

        {
            let mut battle = app.world_mut().resource_mut::<Battle>();
            for point in &mut battle.objectives {
                point.owner = Some(Team::Blue);
                point.capture = 1.0;
            }
            battle.winner = Some(Team::Blue);
        }
        app.update();
        let world = app.world_mut();
        let mut labels = world.query::<&Text>();
        assert!(
            labels
                .iter(world)
                .any(|text| text.0.contains("Blue secured all 6 objectives"))
        );
        press(&mut app, |action| matches!(action, Action::Restart));
        press(&mut app, |action| matches!(action, Action::Deploy));
        let battle = app.world().resource::<Battle>();
        assert_eq!(battle.winner, None);
        assert_eq!(battle.squads[0].kit, Kit::Support);
    }
}

#[derive(Component)]
struct RingDot {
    soldier: usize,
    step: usize,
}

fn soldier_row(parent: &mut ChildSpawnerCommands, soldier: usize) {
    parent
        .spawn((
            Pickable::IGNORE,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(5),
                min_height: px(17),
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Pickable::IGNORE,
                Node {
                    width: px(16),
                    height: px(16),
                    flex_shrink: 0.0,
                    ..default()
                },
            ))
            .with_children(|ring| {
                for step in 0..12 {
                    let angle = step as f32 * std::f32::consts::TAU / 12.0;
                    ring.spawn((
                        RingDot { soldier, step },
                        Pickable::IGNORE,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(6.0 + angle.sin() * 6.0),
                            top: px(6.0 - angle.cos() * 6.0),
                            width: px(3),
                            height: px(3),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ));
                }
            });
            dynamic(row, HudText::Soldier(soldier), 11., INK);
        });
}

fn update_rings(battle: Res<Battle>, mut dots: Query<(&RingDot, &mut BackgroundColor)>) {
    let squad = battle
        .selected
        .iter()
        .next()
        .and_then(|i| battle.squads.get(*i));
    for (dot, mut color) in &mut dots {
        let next = squad
            .and_then(|s| s.soldiers.get(dot.soldier).map(|soldier| (s.kit, soldier)))
            .map_or(Color::NONE, |(kit, soldier)| {
                let duration = if kit == Kit::Support { 0.85 } else { 1.2 };
                let progress = 1.0 - soldier.cooldown / duration;
                if soldier.hp <= 0.0 {
                    Color::srgb(0.75, 0.25, 0.25)
                } else if soldier.cooldown == 0.0 {
                    BLUE
                } else if dot.step as f32 / 12.0 < progress {
                    GOLD
                } else {
                    Color::srgb(0.25, 0.30, 0.30)
                }
            });
        color.set_if_neq(BackgroundColor(next));
    }
}
