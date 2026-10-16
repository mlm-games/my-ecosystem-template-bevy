use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;
use repose_bevy::state::ReposeState;
use repose_core::SceneNode;

pub struct DevToolsPlugin;
impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "dev")]
        {
            app.add_systems(Update, log_state_change);
        }
        app.add_systems(
            PostUpdate,
            trace_repose_scene.after(repose_bevy::compose::compose_repose_system),
        );
    }
}

#[cfg(feature = "dev")]
fn log_state_change(state: Res<State<crate::app::AppState>>) {
    if state.is_changed() {
        bevy::log::info!("AppState  {:?}", state.get());
    }
}

#[derive(Default)]
struct TraceAcc {
    frames: u32,
    changed: u32,
    digest: Option<u64>,
    layers: BTreeSet<(u32, i32, i32)>,
    elevation: BTreeSet<i32>,
}

const TRACE_WINDOW: u32 = 60;

fn trace_repose_scene(state: Option<NonSend<ReposeState>>, mut acc: Local<TraceAcc>) {
    let Some(state) = state else {
        return;
    };

    let mut hasher = DefaultHasher::new();
    for node in &state.scene.nodes {
        std::mem::discriminant(node).hash(&mut hasher);
        match node {
            SceneNode::BeginLayer {
                layer_id,
                blur_radius_x,
                blur_radius_y,
                ..
            } => {
                layer_id.hash(&mut hasher);
                acc.layers
                    .insert((*layer_id, px100(blur_radius_x.0), px100(blur_radius_y.0)));
            }
            SceneNode::EndLayer { layer_id } => layer_id.hash(&mut hasher),
            SceneNode::CompositeShadow {
                layer_id, blur_px, ..
            } => {
                layer_id.hash(&mut hasher);
                acc.layers.insert((*layer_id, px100(blur_px.0), 0));
            }
            SceneNode::Shadow { elevation, .. } => {
                elevation.0.to_bits().hash(&mut hasher);
                acc.elevation.insert(px100(elevation.0));
            }
            SceneNode::Text { text, .. } => text.hash(&mut hasher),
            _ => {}
        }
    }

    let digest = hasher.finish();
    acc.frames += 1;
    if acc.digest != Some(digest) {
        acc.digest = Some(digest);
        acc.changed += 1;
    }

    if acc.frames < TRACE_WINDOW {
        return;
    }

    if acc.changed > 0 {
        info!(
            "repose diag: fb={}x{} nodes={} scene_changed={}/{} layers={:?} elevation={:?}",
            state.fb_width,
            state.fb_height,
            state.scene.nodes.len(),
            acc.changed,
            acc.frames,
            acc.layers,
            acc.elevation,
        );
    }

    *acc = TraceAcc::default();
}

fn px100(value: f32) -> i32 {
    (value * 100.0).round() as i32
}
