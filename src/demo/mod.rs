use bevy::input::gamepad::{Gamepad, GamepadRumbleRequest};
use bevy::prelude::*;
use rand::RngExt;

use crate::app::{AppState, Paused};
use crate::save::SaveData;
use game_utils_bevy::game_feel::GameFeel;
use game_utils_bevy::juice::Juice;
use game_utils_bevy::save::SaveManager;
use game_utils_bevy::screen_effects::{ChromaticAberration, ScreenEffects, Trauma};
use game_utils_bevy::transitions::Transition;
use game_utils_bevy::vfx::VfxSpawner;
#[derive(Clone, Copy, PartialEq, Eq)]
enum PowerupKind {
    Chromatic,
}

#[derive(Component)]
struct PowerupDrop {
    kind: PowerupKind,
    speed: f32,
}

#[derive(Resource, Default)]
pub struct Score(pub u32);

/// Set when in-memory save data diverges from disk; a throttled system flushes it so
/// the high score isn't written on every kill.
#[derive(Resource, Default)]
pub struct SaveDirty(pub bool);

#[derive(Component)]
struct Player {
    speed: f32,
    cooldown: Timer,
}

#[derive(Component)]
struct Bullet {
    vel: Vec2,
    life: Timer,
}

#[derive(Component)]
struct Enemy {
    speed: f32,
}

#[derive(Component)]
struct DemoCleanup;

#[derive(Resource)]
struct PowerupActive(Timer);

pub struct DemoPlugin;
impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .init_resource::<SaveDirty>()
            .insert_resource(PowerupActive(Timer::new(
                core::time::Duration::ZERO,
                TimerMode::Once,
            )))
            .add_systems(OnEnter(AppState::InGame), (setup_demo, init_powerup))
            .add_systems(OnExit(AppState::InGame), cleanup_demo)
            .add_systems(
                Update,
                (
                    player_move,
                    player_shoot,
                    move_bullets,
                    spawn_enemies,
                    move_enemies,
                    bullet_enemy_collision,
                    move_powerups,
                    collect_powerups,
                    tick_powerup,
                    flush_dirty_save,
                )
                    .run_if(in_state(AppState::InGame))
                    .run_if(|p: Res<Paused>| !p.0)
                    .run_if(|t: Res<Transition<AppState>>| !t.block_input),
            )
            .add_systems(OnExit(AppState::InGame), flush_dirty_save_once);
    }
}

fn setup_demo(mut commands: Commands, mut score: ResMut<Score>) {
    score.0 = 0;
    let player = commands
        .spawn((
            DemoCleanup,
            Player {
                speed: 320.0,
                cooldown: Timer::from_seconds(0.18, TimerMode::Repeating),
            },
            Sprite {
                color: Color::srgb(0.3, 0.75, 1.0),
                custom_size: Some(Vec2::splat(28.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 10.0),
        ))
        .id();
    Juice::pop_in(&mut commands, player, 0.35);
}

fn init_powerup(mut powerup: ResMut<PowerupActive>) {
    powerup.0.finish();
}

fn cleanup_demo(
    mut commands: Commands,
    q: Query<Entity, With<DemoCleanup>>,
    numbers: Query<Entity, With<game_utils_bevy::vfx::DamageNumber>>,
    particles: Query<Entity, With<game_utils_bevy::juice::Particle>>,
) {
    for e in q.iter().chain(numbers.iter()).chain(particles.iter()) {
        commands.entity(e).despawn();
    }
}

fn player_move(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&Player, &mut Transform)>,
) {
    let Ok((p, mut tf)) = q.single_mut() else {
        return;
    };
    let mut d = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        d.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        d.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        d.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        d.x += 1.0;
    }
    if d != Vec2::ZERO {
        tf.translation += (d.normalize() * p.speed * time.delta_secs()).extend(0.0);
        tf.translation.x = tf.translation.x.clamp(-600.0, 600.0);
        tf.translation.y = tf.translation.y.clamp(-320.0, 320.0);
    }
}

fn spawn_bullet(commands: &mut Commands, origin: Vec3, dir: Vec2) {
    let angle = dir.y.atan2(dir.x) - f32::to_radians(90.0);
    commands.spawn((
        DemoCleanup,
        Bullet {
            vel: dir * 520.0,
            life: Timer::from_seconds(1.2, TimerMode::Once),
        },
        Sprite {
            color: Color::srgb(1.0, 0.9, 0.3),
            custom_size: Some(Vec2::new(6.0, 14.0)),
            ..default()
        },
        Transform::from_translation(origin).with_rotation(Quat::from_rotation_z(angle)),
    ));
}

fn player_shoot(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Player, &Transform)>,
    powerup: Option<Res<PowerupActive>>,
) {
    let Ok((e, mut p, tf)) = q.single_mut() else {
        return;
    };
    p.cooldown.tick(time.delta());
    let fire = mouse.pressed(MouseButton::Left) || keys.pressed(KeyCode::Space);
    if fire && p.cooldown.just_finished() {
        GameFeel::add_recoil(&mut commands, e, Vec2::NEG_Y, 6.0, 0.2);
        let origin = tf.translation + Vec3::Y * 20.0;
        let triple = powerup.as_ref().is_some_and(|p| !p.0.is_finished());
        spawn_bullet(&mut commands, origin, Vec2::Y);
        if triple {
            spawn_bullet(&mut commands, origin, Vec2::new(-1.0, 1.0).normalize());
            spawn_bullet(&mut commands, origin, Vec2::new(1.0, 1.0).normalize());
        }
    }
}

fn move_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Bullet, &mut Transform)>,
) {
    for (e, mut b, mut tf) in &mut q {
        b.life.tick(time.delta());
        tf.translation += (b.vel * time.delta_secs()).extend(0.0);
        if b.life.just_finished() {
            commands.entity(e).despawn();
        }
    }
}

fn spawn_enemies(mut commands: Commands, time: Res<Time>, mut timer: Local<f32>) {
    *timer -= time.delta_secs();
    if *timer > 0.0 {
        return;
    }
    *timer = 0.8;
    let mut rng = rand::rng();
    let x = rng.random_range(-550.0..550.0);
    let e = commands
        .spawn((
            DemoCleanup,
            Enemy {
                speed: rng.random_range(60.0..140.0),
            },
            Sprite {
                color: Color::srgb(1.0, 0.35, 0.35),
                custom_size: Some(Vec2::splat(24.0)),
                ..default()
            },
            Transform::from_xyz(x, 360.0, 5.0),
        ))
        .id();
    Juice::pop_in(&mut commands, e, 0.25);
}

fn move_enemies(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &Enemy, &mut Transform)>,
) {
    for (e, en, mut tf) in &mut q {
        tf.translation.y -= en.speed * time.delta_secs();
        if tf.translation.y < -400.0 {
            commands.entity(e).despawn();
        }
    }
}

fn spawn_powerup(commands: &mut Commands, pos: Vec2) {
    let kind = PowerupKind::Chromatic;
    let color = match kind {
        PowerupKind::Chromatic => Color::srgb(0.2, 0.6, 1.0),
    };
    let e = commands
        .spawn((
            DemoCleanup,
            PowerupDrop {
                kind,
                speed: rand::rng().random_range(30.0..60.0),
            },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(18.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 3.0),
        ))
        .id();
    Juice::pop_in(commands, e, 0.2);
}

fn bullet_enemy_collision(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut trauma: ResMut<Trauma>,
    mut save: ResMut<SaveData>,
    mut dirty: ResMut<SaveDirty>,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut rumble: MessageWriter<GamepadRumbleRequest>,
) {
    for (be, bt) in &bullets {
        for (ee, et) in &enemies {
            if bt
                .translation
                .truncate()
                .distance(et.translation.truncate())
                < 18.0
            {
                let pos = et.translation.truncate();
                commands.entity(be).despawn();
                commands.entity(ee).despawn();
                score.0 += 10;
                ScreenEffects::add_trauma(&mut trauma, 0.35);
                GameFeel::rumble_controller(&mut rumble, &gamepads, 0.3, 0.7, 0.15);
                VfxSpawner::spawn_damage_number(&mut commands, 10, pos, Color::srgb(1.0, 0.9, 0.2));
                VfxSpawner::spawn_burst(
                    &mut commands,
                    pos,
                    8,
                    Color::srgb(1.0, 0.4, 0.3),
                    (40.0, 100.0),
                );
                if rand::rng().random_range(0.0..1.0) < 0.05 {
                    spawn_powerup(&mut commands, pos);
                }
                if score.0 > save.high_score {
                    save.high_score = score.0;
                    dirty.0 = true;
                }
            }
        }
    }
}

fn tick_powerup(time: Res<Time>, mut powerup_active: ResMut<PowerupActive>) {
    powerup_active.0.tick(time.delta());
}

fn move_powerups(time: Res<Time>, mut q: Query<(&PowerupDrop, &mut Transform)>) {
    for (p, mut tf) in &mut q {
        tf.translation.y -= p.speed * time.delta_secs();
        if tf.translation.y < -380.0 {
            tf.translation.y = 380.0;
            tf.translation.x = rand::rng().random_range(-550.0..550.0);
        }
    }
}

fn collect_powerups(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    powerups: Query<(Entity, &Transform, &PowerupDrop)>,
    mut chroma: ResMut<ChromaticAberration>,
    mut powerup_active: ResMut<PowerupActive>,
) {
    let Ok(pt) = player.single() else {
        return;
    };
    let ppos = pt.translation.truncate();
    for (e, t, drop) in &powerups {
        if ppos.distance(t.translation.truncate()) > 28.0 {
            continue;
        }
        commands.entity(e).despawn();
        match drop.kind {
            PowerupKind::Chromatic => {
                ScreenEffects::chromatic_pulse(&mut chroma, 0.8);
                powerup_active.0 = Timer::from_seconds(5.0, TimerMode::Once);
            }
        }
    }
}

/// Throttled flush of the dirty high score so disk writes happen at most once per ~5s
/// of active play instead of on every kill.
fn flush_dirty_save(
    mut accumulator: Local<f32>,
    time: Res<Time>,
    dirty: Res<SaveDirty>,
    save: Res<SaveData>,
    manager: Res<SaveManager>,
) {
    if !dirty.0 {
        return;
    }
    *accumulator += time.delta_secs();
    if *accumulator >= 5.0 {
        *accumulator = 0.0;
        let _ = manager.save(&*save);
    }
}

/// Ensure a pending dirty save lands when leaving the demo, even if the throttle timer
/// hadn't fired yet.
fn flush_dirty_save_once(dirty: Res<SaveDirty>, save: Res<SaveData>, manager: Res<SaveManager>) {
    if dirty.0 {
        let _ = manager.save(&*save);
    }
}
