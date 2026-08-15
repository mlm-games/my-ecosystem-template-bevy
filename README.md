# My Ecosystem Bevy

A WIP Bevy 2D game template with ecosystem plugins ported from [my-ecosystem-template](https://github.com/mlm-games/my-ecosystem-template) (Godot).

## Features

- **Game Feel** - recoil, knockback, slow-motion, rumble (gamepad)
- **Screen Effects** - trauma shake, freeze frame, flash white, chromatic aberration pulse + decay
- **Transitions** - fade to black, circle wipe scene transitions with input edge blocking
- **Audio** - channel-based SFX/Music/UI buses with independent volume control (`BaseVolume` × bus), pitch variation, pooled SFX (uses Bevy built-in audio, no external dep)
- **Localization** - Fluent-based i18n with 7 bundled locales (en, es, fr, de, ja, zh, pt), language switcher in settings, `LocaleResources` resource
- **Save System** - persistent RON save with atomic writes + version migration via `directories`
- **Object Pooling** - generic entity pool with acquire/release
- **Juice** - pop-in, squash & stretch, bounce scale, shake, particles with gravity/fade
- **VFX** - damage numbers, particle bursts, trail emitters
- **UI Effects** - hover scale, typewriter text, number counter
- **Math Utils** - smooth_damp, approach, wave (f32, Vec2, Vec3)
- **Center Pivot** - sprite origin centering component
- **UI** - animated buttons, popup system, pause/settings/credits with localized text (Repose)
- **States** - Splash -> Loading -> Title -> InGame with pause overlay
- **Theme** - centralized color constants
- **Dev Tools** - FPS overlay, state logging (dev feature)
- **Demo Scene** - player with shooting, enemies, trauma, recoil, burst effects, damage numbers, gamepad rumble

## Quick Start

```bash
cargo run
```

With physics (Avian2d, will be switched to rapier soon):
```bash
cargo run --features physics
```

Dev build with hot-reload:
```bash
cargo run --features dev
```

## Structure

The game-feel ecosystem (audio, transitions, juice, VFX, save, i18n, pooling, game feel)
lives in the **[game-utils](https://github.com/mlm-games/game-utils)** workspace, split into:

- `crates/game-utils` - Bevy-agnostic core (save manager, i18n, math, stats, achievements)
- `crates/game-utils-bevy/src` - Bevy plugins:
  - `audio.rs` - channel-based audio buses (SfxChannel/MusicChannel/UiChannel)
  - `center_pivot.rs` - sprite origin centering
  - `game_feel.rs` - recoil, knockback, slow-motion, gamepad rumble
  - `i18n.rs` - Fluent-based localization (7 locales, language switcher)
  - `juice.rs` - pop-in, squash/stretch, bounce, shake, particles
  - `pooling.rs` - generic entity pooling
  - `save.rs` - RON save/load with atomic writes + version migration
  - `screen_effects.rs` - trauma, freeze frame, flash white, chromatic aberration
  - `time_scale.rs` - single owner of virtual-time speed/pause
  - `transitions.rs` - fade/circle wipe with input edge blocking
  - `ui_effects.rs` - hover scale, typewriter, number counter
  - `vfx.rs` - damage numbers, particle bursts, trail emitters

This template repo holds only the app layer:

```
src/
├── main.rs              # Entry point
├── app.rs               # AppPlugin, states, system sets
├── save.rs              # SaveData type (persisted via game-utils)
├── screens/             # Splash, loading, title
├── menus/               # Main, pause, settings, credits (localized)
├── theme/               # Theme resource
├── demo/                # Sample gameplay with all juice
├── dev_tools.rs         # FPS overlay, state logging
└── asset_tracking.rs    # Preload tracking
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `bevy` (git rev) | Engine |
| `repose-bevy` / `repose-*` | UI framework |
| `fluent-bundle` + `unic-langid` | Localization (Fluent) |
| `serde` + `ron` + `directories` | Save system |
| `rand` | Random variation (audio pitch, VFX) |
| `avian2d` (optional) | Physics |

## License

GPL-3.0
