use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
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

/// One-line-per-window trace of the framebuffer, the pointer feed and the hit
/// result, to separate "the UI blinks" from "the hover target flaps".
#[derive(Default)]
struct TraceAcc {
    frames: u32,
    fb_changes: u32,
    scene_changes: u32,
    hover_changes: u32,
    digest: Option<u64>,
    last_fb: (u32, u32),
    last_hover: Option<Option<u64>>,
    fb_min: (u32, u32),
    fb_max: (u32, u32),
    dt_min: f32,
    dt_max: f32,
    hover: BTreeSet<Option<u64>>,
    pos_min: (f32, f32),
    pos_max: (f32, f32),
    cursor: BTreeSet<(i32, i32)>,
}

const TRACE_WINDOW: u32 = 120;

fn trace_repose_scene(
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    state: Option<NonSend<ReposeState>>,
    mut acc: Local<TraceAcc>,
) {
    let Some(state) = state else {
        return;
    };
    let scale = windows
        .single()
        .map(|w| w.resolution.scale_factor())
        .unwrap_or(1.0);
    let cursor = windows
        .single()
        .ok()
        .and_then(|w| w.cursor_position())
        .map(|c| (px100(c.x), px100(c.y)));

    let dt_ms = time.delta_secs() * 1000.0;
    acc.dt_min = if acc.frames == 0 {
        dt_ms
    } else {
        acc.dt_min.min(dt_ms)
    };
    acc.dt_max = acc.dt_max.max(dt_ms);

    let fb = (state.fb_width, state.fb_height);
    let hover = state.runtime.hover_id;
    let pos = state.runtime.mouse_pos_px;
    let node_count = state.scene.nodes.len();

    if acc.frames == 0 {
        acc.fb_min = fb;
        acc.fb_max = fb;
        acc.last_fb = fb;
        acc.last_hover = Some(hover);
        acc.pos_min = pos;
        acc.pos_max = pos;
    } else {
        acc.fb_min = (acc.fb_min.0.min(fb.0), acc.fb_min.1.min(fb.1));
        acc.fb_max = (acc.fb_max.0.max(fb.0), acc.fb_max.1.max(fb.1));
        if fb != acc.last_fb {
            acc.fb_changes += 1;
            acc.last_fb = fb;
        }
        if acc.last_hover != Some(hover) {
            acc.hover_changes += 1;
            acc.last_hover = Some(hover);
        }
        acc.pos_min = (acc.pos_min.0.min(pos.0), acc.pos_min.1.min(pos.1));
        acc.pos_max = (acc.pos_max.0.max(pos.0), acc.pos_max.1.max(pos.1));
    }
    acc.hover.insert(hover);
    if let Some(c) = cursor {
        acc.cursor.insert(c);
    }

    let mut hasher = DefaultHasher::new();
    for node in &state.scene.nodes {
        std::mem::discriminant(node).hash(&mut hasher);
        if let SceneNode::Text { text, .. } = node {
            text.hash(&mut hasher);
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
        "repose diag[v3]: fb={}x{}..{}x{} fb_churn={} scale={} dt={:.0}..{:.0}ms nodes={} hover={:?} hover_churn={} pos={:.0},{:.0}..{:.0},{:.0} cursor_px100={:?} scene_changed={}/{}",
        acc.fb_min.0,
        acc.fb_min.1,
        acc.fb_max.0,
        acc.fb_max.1,
        acc.fb_changes,
        scale,
        acc.dt_min,
        acc.dt_max,
        node_count,
        acc.hover,
        acc.hover_changes,
        acc.pos_min.0,
        acc.pos_min.1,
        acc.pos_max.0,
        acc.pos_max.1,
        acc.cursor,
        acc.scene_changes,
        acc.frames,
    );

    *acc = TraceAcc::default();
}

fn px100(value: f32) -> i32 {
    (value * 100.0).round() as i32
}
