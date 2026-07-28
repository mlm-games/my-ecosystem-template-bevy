use bevy::prelude::*;

use crate::app::AppState;
use crate::ecosystem::transitions::Transition;

pub struct ScreensPlugin;
impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), |mut c: Commands| {
            c.insert_resource(SplashTimer(Timer::from_seconds(1.5, TimerMode::Once)));
        })
        .add_systems(OnEnter(AppState::Loading), |mut c: Commands| {
            c.insert_resource(LoadingTimer(Timer::from_seconds(1.0, TimerMode::Once)));
        })
        .add_systems(Update, (tick_splash, tick_loading));
    }
}

#[derive(Resource)]
struct SplashTimer(Timer);
#[derive(Resource)]
struct LoadingTimer(Timer);

fn tick_splash(time: Res<Time<Real>>, mut tr: ResMut<Transition>, timer: Option<ResMut<SplashTimer>>) {
    let Some(mut timer) = timer else { return };
    if timer.0.tick(time.delta()).just_finished() {
        tr.begin_to_state(AppState::Loading);
    }
}

fn tick_loading(time: Res<Time<Real>>, mut tr: ResMut<Transition>, timer: Option<ResMut<LoadingTimer>>) {
    let Some(mut timer) = timer else { return };
    if timer.0.tick(time.delta()).just_finished() {
        tr.begin_to_state(AppState::Title);
    }
}
