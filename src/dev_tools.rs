use bevy::prelude::*;

pub struct DevToolsPlugin;
impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "dev")]
        {
            app.add_systems(Update, log_state_change);
        }
    }
}

#[cfg(feature = "dev")]
fn log_state_change(state: Res<State<crate::app::AppState>>) {
    if state.is_changed() {
        bevy::log::info!("AppState  {:?}", state.get());
    }
}
