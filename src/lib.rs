pub mod app;
mod asset_tracking;
mod demo;
mod dev_tools;
pub mod ecosystem;
pub mod menus;
pub mod screens;
mod theme;

pub use app::AppPlugin;

use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, WindowMode};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(create_primary_window()),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(AppPlugin)
        .add_systems(Update, (make_window_visible, toggle_fullscreen))
        .run();
}

fn create_primary_window() -> Window {
    Window {
        title: "My Ecosystem Bevy".into(),
        name: Some("my.ecosystem.bevy".into()),
        resolution: (1280, 720).into(),
        present_mode: PresentMode::AutoVsync,
        resizable: true,

        fit_canvas_to_parent: true,
        prevent_default_event_handling: false,

        #[cfg(not(any(
            target_arch = "wasm32",
            target_os = "android",
            target_os = "ios"
        )))]
        visible: false,

        #[cfg(any(target_os = "android", target_os = "ios"))]
        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        #[cfg(any(target_os = "android", target_os = "ios"))]
        resizable: false,

        #[cfg(target_os = "ios")]
        recognize_rotation_gesture: true,
        #[cfg(target_os = "ios")]
        prefers_home_indicator_hidden: true,
        #[cfg(target_os = "ios")]
        prefers_status_bar_hidden: true,

        ..default()
    }
}

fn make_window_visible(mut window: Single<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == 3 {
        window.visible = true;
    }
}

fn toggle_fullscreen(
    keys: Res<ButtonInput<KeyCode>>,
    mut window: Single<&mut Window>,
) {
    if !keys.just_pressed(KeyCode::F11) {
        return;
    }
    window.mode = match window.mode {
        WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        _ => WindowMode::Windowed,
    };
}