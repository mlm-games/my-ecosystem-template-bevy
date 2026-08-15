use bevy::prelude::*;
use game_utils_bevy::transitions::Transition;

use crate::app::AppState;
use crate::asset_tracking::AssetsLoading;

pub struct ScreensPlugin;
impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), |mut c: Commands| {
            c.insert_resource(SplashTimer(Timer::from_seconds(1.5, TimerMode::Once)));
        })
        .add_systems(
            OnEnter(AppState::Loading),
            (|mut c: Commands, asset_server: Res<AssetServer>| {
                c.insert_resource(LoadingTimer(Timer::from_seconds(0.5, TimerMode::Once)));
                let handles = vec![asset_server.load::<Font>("fonts/default.ttf").untyped()];
                c.insert_resource(AssetsLoading(handles));
            },)
                .chain(),
        )
        .add_systems(Update, (tick_splash, tick_loading))
        .add_systems(
            OnExit(AppState::Splash),
            |mut c: Commands| c.remove_resource::<SplashTimer>(),
        )
        .add_systems(
            OnExit(AppState::Loading),
            |mut c: Commands| {
                c.remove_resource::<LoadingTimer>();
                c.remove_resource::<AssetsLoading>();
            },
        );
    }
}

#[derive(Resource)]
struct SplashTimer(Timer);
#[derive(Resource)]
struct LoadingTimer(Timer);

fn tick_splash(
    time: Res<Time<Real>>,
    mut tr: ResMut<Transition<AppState>>,
    timer: Option<ResMut<SplashTimer>>,
) {
    let Some(mut timer) = timer else { return };
    if timer.0.tick(time.delta()).just_finished() {
        tr.begin_to_state(AppState::Title);
    }
}

fn tick_loading(
    time: Res<Time<Real>>,
    mut tr: ResMut<Transition<AppState>>,
    asset_server: Res<AssetServer>,
    timer: Option<ResMut<LoadingTimer>>,
    assets: Option<Res<AssetsLoading>>,
) {
    let Some(mut timer) = timer else { return };
    let loaded = assets
        .map(|a| {
            a.0.iter()
                .all(|h| asset_server.is_loaded_with_dependencies(h))
        })
        .unwrap_or(true);
    if loaded && timer.0.tick(time.delta()).just_finished() {
        tr.begin_to_state(AppState::InGame);
    }
}
