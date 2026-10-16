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

/// One-line-per-second trace of everything that could make the whole UI blink:
/// framebuffer churn (the overlay image is rebuilt whenever the size changes,
/// and it is rebuilt transparent), node-count churn, elevation, and frame time.
#[derive(Default)]
struct TraceAcc {
    frames: u32,
    fb_changes: u32,
    node_changes: u32,
    scene_changes: u32,
    digest: Option<u64>,
    last_fb: (u32, u32),
    last_nodes: usize,
    fb_min: (u32, u32),
    fb_max: (u32, u32),
    nodes_min: usize,
    nodes_max: usize,
    elev_min: i32,
    elev_max: i32,
    dt_min: f32,
    dt_max: f32,
    elevation: BTreeSet<i32>,
}

const TRACE_WINDOW: u32 = 120;

fn trace_repose_scene(
    time: Res<Time>,
    state: Option<NonSend<ReposeState>>,
    mut acc: Local<TraceAcc>,
) {
    let Some(state) = state else {
        return;
    };

    let dt_ms = time.delta_secs() * 1000.0;
    acc.dt_min = if acc.frames == 0 {
        dt_ms
    } else {
        acc.dt_min.min(dt_ms)
    };
    acc.dt_max = acc.dt_max.max(dt_ms);

    let fb = (state.fb_width, state.fb_height);
    if acc.frames == 0 {
        acc.fb_min = fb;
        acc.fb_max = fb;
        acc.last_fb = fb;
        acc.nodes_min = state.scene.nodes.len();
        acc.nodes_max = acc.nodes_min;
        acc.last_nodes = acc.nodes_min;
    } else {
        acc.fb_min = (acc.fb_min.0.min(fb.0), acc.fb_min.1.min(fb.1));
        acc.fb_max = (acc.fb_max.0.max(fb.0), acc.fb_max.1.max(fb.1));
        if fb != acc.last_fb {
            acc.fb_changes += 1;
            acc.last_fb = fb;
        }
        let nodes = state.scene.nodes.len();
        acc.nodes_min = acc.nodes_min.min(nodes);
        acc.nodes_max = acc.nodes_max.max(nodes);
        if nodes != acc.last_nodes {
            acc.node_changes += 1;
            acc.last_nodes = nodes;
        }
    }

    let mut hasher = DefaultHasher::new();
    for node in &state.scene.nodes {
        std::mem::discriminant(node).hash(&mut hasher);
        match node {
            SceneNode::Shadow { elevation, .. } => {
                let e = px100(elevation.0);
                acc.elevation.insert(e);
                acc.elev_min = if acc.elevation.len() == 1 {
                    e
                } else {
                    acc.elev_min.min(e)
                };
                acc.elev_max = acc.elev_max.max(e);
            }
            SceneNode::Text { text, .. } => text.hash(&mut hasher),
            SceneNode::BeginLayer { layer_id, .. } => layer_id.hash(&mut hasher),
            _ => {}
        }
    }

    let digest = hasher.finish();
    acc.frames += 1;
    if acc.digest != Some(digest) {
        acc.digest = Some(digest);
        acc.scene_changes += 1;
    }

    if acc.frames < TRACE_WINDOW {
        return;
    }

    info!(
        "repose diag: fb={}x{}..{}x{} fb_churn={} dt={:.1}..{:.1}ms nodes={}..{} node_churn={} scene_changed={}/{} elev={:?}",
        acc.fb_min.0,
        acc.fb_min.1,
        acc.fb_max.0,
        acc.fb_max.1,
        acc.fb_changes,
        acc.dt_min,
        acc.dt_max,
        acc.nodes_min,
        acc.nodes_max,
        acc.node_changes,
        acc.scene_changes,
        acc.frames,
        acc.elevation,
    );

    *acc = TraceAcc::default();
}

fn px100(value: f32) -> i32 {
    (value * 100.0).round() as i32
}
