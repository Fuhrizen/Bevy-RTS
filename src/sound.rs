//! Short audio cues sourced from match state.
use crate::model::{Battle, Screen, Team};
use bevy::{
    audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume},
    prelude::*,
};

pub struct SoundPlugin;

#[derive(Resource)]
struct Sounds {
    volley: Handle<AudioSource>,
    capture: Handle<AudioSource>,
    order: Handle<AudioSource>,
}

#[derive(Default, Resource)]
struct Cues {
    volleys: u64,
    orders: u64,
    owners: Vec<Option<Team>>,
    last_volley: f32,
}

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        bevy::asset::embedded_asset!(app, "audio/volley.wav");
        bevy::asset::embedded_asset!(app, "audio/capture.wav");
        bevy::asset::embedded_asset!(app, "audio/order.wav");
        app.init_resource::<Cues>()
            .add_systems(Startup, load)
            .add_systems(OnEnter(Screen::Battle), |mut cues: ResMut<Cues>| {
                *cues = Cues::default()
            })
            .add_systems(Update, play_cues.run_if(in_state(Screen::Battle)));
    }
}

fn load(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Sounds {
        volley: bevy::asset::load_embedded_asset!(&*assets, "audio/volley.wav"),
        capture: bevy::asset::load_embedded_asset!(&*assets, "audio/capture.wav"),
        order: bevy::asset::load_embedded_asset!(&*assets, "audio/order.wav"),
    });
}

fn play_cues(
    mut commands: Commands,
    battle: Res<Battle>,
    sounds: Res<Sounds>,
    mut cues: ResMut<Cues>,
) {
    let mut play = |source: &Handle<AudioSource>, volume| {
        commands.spawn((
            AudioPlayer::new(source.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
        ));
    };
    if cues.volleys != battle.volley_count && battle.elapsed - cues.last_volley > 0.14 {
        play(&sounds.volley, 0.25);
        cues.last_volley = battle.elapsed;
    }
    cues.volleys = battle.volley_count;
    let owners: Vec<_> = battle.objectives.iter().map(|point| point.owner).collect();
    if owners
        .iter()
        .zip(&cues.owners)
        .any(|(new, old)| *new == Some(Team::Blue) && new != old)
    {
        play(&sounds.capture, 0.4);
    }
    cues.owners = owners;
    if cues.orders != battle.order_count {
        play(&sounds.order, 0.25);
        cues.orders = battle.order_count;
    }
}
