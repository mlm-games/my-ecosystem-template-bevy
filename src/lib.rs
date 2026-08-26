mod app;
mod asset_tracking;
mod demo;
mod dev_tools;
mod menus;
mod save;
mod screens;
mod theme;

use app::AppPlugin;
use bevy::prelude::*;
use bevy::window::WindowResolution;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let primary_window = Window {
        title: "My Ecosystem Bevy".into(),
        resolution: WindowResolution::new(1280, 720),
        #[cfg(target_arch = "wasm32")]
        fit_canvas_to_parent: true,
        #[cfg(target_arch = "wasm32")]
        prevent_default_event_handling: true,
        ..default()
    };

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(primary_window),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    );

    // PHYSICS
    {
        use bevy_rapier2d::prelude::*;
        let mut init = RapierContextInitialization::default_with_length_unit(20.0);
        if let RapierContextInitialization::InitializeDefaultRapierContext {
            rapier_configuration,
            ..
        } = &mut init
        {
            rapier_configuration.gravity = Vec2::new(0.0, -980.0);
        }
        app.insert_resource(init);

        app.insert_resource(TimestepMode::Interpolated {
            dt: 1.0 / 60.0,
            time_scale: 1.0,
            substeps: 1,
        });
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(20.0));
    }

    app.add_plugins(AppPlugin).run();
}
