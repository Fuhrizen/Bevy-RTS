use crate::model::{Battle, Difficulty, HudBlock, Kit, Roster, Screen, Settings, Team};
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
    Size(usize),
    Kit(usize),
    Pause,
    Hold,
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
    Roles,
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
                (button_feedback, actions, refresh_battle, update_hud).chain(),
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
            text(
                p,
                "Cycle each field to configure your four squads.",
                14.,
                MUTED,
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
            left: px(18),
            top: px(18),
            width: px(236),
            ..panel()
        },
        |p| {
            text(p, "OPERATION  /  SECTOR CONTROL", 11., BLUE);
            dynamic(p, HudText::Tally, 19., INK);
            for i in 0..battle.objectives.len() {
                dynamic(p, HudText::Objective(i), 11., INK);
            }
        },
    );
    block(
        commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            right: px(18),
            top: px(18),
            width: px(210),
            ..panel()
        },
        |p| {
            text(p, "FIELD GUIDE", 11., GOLD);
            text(
                p,
                "LMB  Select / drag\nRMB  Move / attack\nSHIFT  Add / queue\nCTRL + 1-4  Set group\n1-4  Recall group\nWASD / EDGE  Pan\nWHEEL  Zoom",
                12.,
                INK,
            );
            button(
                p,
                if battle.paused { "RESUME" } else { "PAUSE" },
                Action::Pause,
                150.,
            );
        },
    );
    block(
        commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            left: px(18),
            bottom: px(18),
            width: px(405),
            ..panel()
        },
        |p| {
            text(p, "BLUE FORCE  /  SQUAD STATUS", 11., BLUE);
            for (i, s) in battle
                .squads
                .iter()
                .enumerate()
                .filter(|(_, s)| s.team == Team::Blue)
            {
                let selected = if battle.selected.contains(&i) {
                    ">"
                } else {
                    " "
                };
                button(
                    p,
                    format!(
                        "{} {}   {}   {}/{}   {:.0}% SUPPLY",
                        selected,
                        s.callsign.to_uppercase(),
                        s.kit.label().to_uppercase(),
                        s.manpower(),
                        s.soldiers.len(),
                        s.supplies
                    ),
                    Action::Select(i),
                    350.,
                );
            }
            dynamic(p, HudText::Selected, 13., GOLD);
            dynamic(p, HudText::Detail, 12., INK);
            dynamic(p, HudText::Roles, 10., MUTED);
        },
    );
    block(
        commands,
        r,
        Node {
            position_type: PositionType::Absolute,
            right: px(18),
            bottom: px(18),
            width: px(310),
            ..panel()
        },
        |p| {
            text(p, "COMMANDS  /  SELECTED FORCE", 11., BLUE);
            p.spawn((
                Pickable::IGNORE,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(6),
                    ..default()
                },
            ))
            .with_children(|row| {
                button(row, "HOLD", Action::Hold, 80.);
                button(row, "DIG IN", Action::Entrench, 80.);
            });
            p.spawn((
                Pickable::IGNORE,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(6),
                    ..default()
                },
            ))
            .with_children(|row| {
                button(row, "SMOKE", Action::Smoke, 80.);
                button(row, "RESUPPLY", Action::Resupply, 95.);
            });
            text(p, "RIGHT CLICK GROUND TO MOVE", 10., MUTED);
        },
    );
    if battle.paused || battle.winner.is_some() || battle.draw {
        block(
            commands,
            r,
            Node {
                position_type: PositionType::Absolute,
                left: percent(39),
                top: percent(30),
                width: px(300),
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
                    text(p, "The operation has concluded.", 13., MUTED);
                } else {
                    button(p, "RESUME", Action::Resume, 210.);
                }
                button(p, "RESTART OPERATION", Action::Restart, 210.);
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
    let mut redraw = false;
    for QueuedAction(action) in actions.read() {
        if (battle.paused || battle.winner.is_some() || battle.draw)
            && matches!(
                *action,
                Action::Hold
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
            Action::Size(i) => {
                roster.squads[i].manpower = match roster.squads[i].manpower {
                    4 => 6,
                    6 => 8,
                    8 => 10,
                    _ => 4,
                };
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
fn update_hud(battle: Res<Battle>, mut labels: Query<(&HudText, &mut Text)>) {
    for (marker, mut text) in &mut labels {
        let value = match *marker {
            HudText::Tally => format!(
                "{:02}:{:02}  |  {}/{} HELD",
                battle.elapsed as u32 / 60,
                battle.elapsed as u32 % 60,
                battle
                    .objectives
                    .iter()
                    .filter(|o| o.owner == Some(Team::Blue))
                    .count(),
                battle.objectives.len()
            ),
            HudText::Objective(i) => battle
                .objectives
                .get(i)
                .map(|o| {
                    format!(
                        "{}    {}",
                        o.name.to_uppercase(),
                        if o.contested {
                            "CONTESTED"
                        } else if o.owner == Some(Team::Blue) {
                            "FRIENDLY"
                        } else if o.owner == Some(Team::Red) {
                            "HOSTILE"
                        } else {
                            "NEUTRAL"
                        }
                    )
                })
                .unwrap_or_default(),
            HudText::Squad(i) => battle
                .squads
                .get(i)
                .map(|s| {
                    format!(
                        "{} {}   {}   {}/{}   {:.0}% SUPPLY",
                        if battle.selected.contains(&i) {
                            ">"
                        } else {
                            " "
                        },
                        s.callsign.to_uppercase(),
                        s.kit.label().to_uppercase(),
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
                        if s.entrenched { "ENTRENCHED" } else { "MOBILE" }
                    )
                })
                .unwrap_or_default(),
            HudText::Roles => battle
                .selected
                .iter()
                .next()
                .and_then(|i| battle.squads.get(*i))
                .map(|s| {
                    s.soldiers
                        .iter()
                        .filter(|x| x.hp > 0. && x.hp < 100.)
                        .take(3)
                        .map(|x| {
                            format!(
                                "{} #{} {:.0} HP V{}",
                                x.role.label(),
                                x.id + 1,
                                x.hp,
                                x.veterancy
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" / ")
                })
                .unwrap_or_default(),
            HudText::Pause => {
                if battle.paused {
                    "RESUME".into()
                } else {
                    "PAUSE".into()
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
        press(&mut app, |action| matches!(action, Action::Deploy));

        let battle = app.world().resource::<Battle>();
        assert_eq!(battle.squads.len(), 8);
        assert_eq!(battle.squads[0].kit, Kit::Support);
        press(&mut app, |action| matches!(action, Action::Pause));
        assert!(app.world().resource::<Battle>().paused);
        press(&mut app, |action| matches!(action, Action::Resume));
        assert!(!app.world().resource::<Battle>().paused);
        press(&mut app, |action| matches!(action, Action::Select(0)));
        assert!(app.world().resource::<Battle>().selected.contains(&0));
    }
}
