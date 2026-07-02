# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

A Rust port of the 1984 ZX Spectrum game "Jet Set Willy" (originally by Matthew Smith), incrementally porting an existing C/SDL2 implementation. Gameplay is 100% identical to the original. The project adds per-pixel coloring (no color clash), polyphonic music, and stereo SFX.

## Build & Run

```bash
cargo build           # debug build
cargo build --release # optimized
cargo run             # run the game
```

**Required system libraries:** `libsdl2-dev libsdl2-mixer-dev build-essential`

`build.rs` uses the `cc` crate to compile remaining C files and links them via pkg-config (SDL2, SDL2_mixer). It also sets `env!("BUILD")` to a datestamped version string used in the loader screen.

## Architecture

The game uses a **hybrid Rust/C approach** — modules are ported one at a time. Both the entry point (`main.rs`) and the main loop (`game_main.rs::run`) live in Rust; `game_main.rs` owns SDL2 window/audio setup and drives the loop via four global function pointers:

```
Action    — current game state handler (runs once per frame, then sets itself to DoNothing)
Responder — input processor
Ticker    — logic/physics update
Drawer    — rendering
```

These are `#[unsafe(no_mangle)] pub static mut` globals defined in `game_main.rs` (and re-exported by `common.rs`); remaining C code references them via `extern`. Each state transition works by re-assigning the pointer (e.g., `Action = Some(Title_Action)`). The `Event` type is `Option<unsafe extern "C" fn()>`, so an unset slot is `None` and behaves as a no-op.

### Ported to Rust (`src/*.rs`)

| Module | Role |
|--------|------|
| `main.rs` | Entry point — calls `game_main::run()` |
| `game_main.rs` | SDL2 init, main loop, audio callback, keyboard polling, the four function-pointer globals |
| `common.rs` | `WIDTH=256`, `HEIGHT=192`, `Key` enum (`#[repr(i32)]`), `Event` type alias, re-exports of `game_main` globals, extern decls for any unported C `*_Action` |
| `video.rs` | Full rendering engine — `videoPixel` buffer, sprite drawing, two character sets (8px/128-char and 16px/96-char) |
| `misc.rs` | 16-color `videoColour` palette (`#[no_mangle]`), `Timer` struct, `Video_Viewport` |
| `cheat.rs` | "writetyper" cheat detection, level selection (1–60) |
| `die.rs` | Death animation and life-loss sequence |
| `audio.rs` | Square-wave synth — 8-channel mixer (3 SFX + 5 music), polyphonic sequencer, stereo panning |
| `title.rs` | Title screen — JSW logo, scrolling ticker, starts music |
| `gameover.rs` | Game over animation — boot kicks Willy, then returns to title |
| `loader.rs` | Loading screen — flashing text, loading bar, copy protection flow |
| `levels.rs` | Level data and room layout definitions |
| `rope.rs` | Rope swing physics and rendering |
| `codes.rs` | Copy-protection code entry and validation |
| `game.rs` | Port of `game.c`/`game.h` — now nearly complete (no `unimplemented!()` stubs remain). Owns `Game_Action`, `DoGameTicker`, `do_game_drawer`, `game_init_room`, `clock_ticker`, `Game_ChangeLevel`, `Game_GameReset`, `game_pause`, and the `GAME_STATE` shadow-state model (see below). Still calls into C for robot physics (`Robots_*`). |
| `miner.rs` | Port of `miner.c` — Willy physics (`Miner_*`): input, jump/fall/walk, ramps/conveyors, collision, item pickup, sprite rendering. Defines the `minerWilly` / `minerWillyRope` / `minerAttrSplit` C-ABI globals (still read by C `robots.c`). |

### Still in C (compiled via `build.rs`)

`build.rs` compiles exactly two C files: `game.c`, `robots.c`.

- `robots.c` — robot movement: the `Robots_*` functions. Reads `minerWilly.{y,air}`.
- `game.c` — **data only now.** Its functions are all ported to Rust; what remains is the definitions of the shared C globals (`gameLevel`, `gameLives`, `levelBorder`, `gameScoreClock`, `gameTimer`, …). These globals are still the source of truth that the C physics code reads and writes. (`minerWilly` & friends are now defined in `miner.rs` instead, via `#[no_mangle]`.)

`Level_Ticker`/`level_init` (`levels.rs`), `Rope_Ticker`/`Rope_Init` (`rope.rs`), and the `Miner_*` functions (`miner.rs`) are Rust, but `game.rs`/`title.rs`/`die.rs` still call them through `unsafe extern "C"` decls because of their exported ABI.

Note: `src/game_main.c`, `src/title.c`, `src/levels.c`, `src/rope.c`, and `src/codes.c` exist on disk (originals before porting) but are **not compiled** and should not be edited — they're reference material for the in-flight ports.

### GAME_STATE shadow-state & C↔Rust sync (read before touching `game.rs`)

The port is mid-migration: still-C code (`robots.c`) reads the raw C globals (`gameLevel`, `minerWilly`, …), while ported Rust code works against `GAME_STATE`, a `LazyLock<GameState>` Rust-side shadow of those globals. `GameState` mixes cheap `Atomic*` fields (level, lives, frame, clock_ticks, item_count, …) with `Mutex<>` fields for the compound ones (`miner`, `timer`, `level_border`, `score_clock`, `score_items`). The C globals are aliased into Rust via `#[link_name]` (imported as `c_game_level`, `c_miner_willy`, … plus `cheatEnabled`).

Two private functions in `game.rs` bridge the two worlds:

- `sync_c_to_rust()` — copy every C global into `GAME_STATE`.
- `sync_rust_to_c()` — copy `GAME_STATE` back out to the C globals.

**The pattern** for a Rust function that shares state with still-C code: call `sync_c_to_rust()` on entry, operate on `GAME_STATE`, then `sync_rust_to_c()` on exit.

**Deadlock rule (the one that keeps biting — cause of the last several commits):** `sync_c_to_rust`/`sync_rust_to_c` lock *every* `Mutex` field. So **never hold a `GAME_STATE` mutex guard across a call to either sync function** — a `let`-bound guard (`timer`, `miner`, `level_border`, `score_clock`) that's still alive when `sync_*` runs self-deadlocks and freezes the game. Wrap the guard in an explicit `{ }` scope so it drops before the sync call. See the `game-state-guard-across-sync-deadlock` memory.

The sync functions currently carry `eprintln!` tracing (and there's an "LLM Slop" `Display` impl on `GameState`) added to hunt these deadlocks — scaffolding to remove once the sync path is stable.

### FFI Conventions

- Rust functions exported to C: `#[unsafe(no_mangle)]` with C-style names (e.g., `Video_DrawSprite`, `Audio_Init`)
- C functions called from Rust: declared as `unsafe extern "C"` blocks in each Rust file (or `common.rs`)
- Shared globals: `static mut` with `#[unsafe(no_mangle)]`; Rust 2024 edition requires `&raw mut` / `&raw const` to take pointers to them (avoids `static_mut_refs` lint)
- `Key` enum and `Colour` struct are `#[repr(C)]`/`#[repr(i32)]` for ABI compatibility; `Colour` has a `_padding: u8` field to match the C layout

**Module hierarchy for FFI declarations** (from `notes.md`): if an unported C function is used by multiple modules, declare it in `common.rs`. If only one module uses it, declare it locally. As a C file is ported, replace its extern declarations with the real `pub` Rust impl in the natural module — do **not** put FFI decls in a sibling module and import them, which inverts the dependency graph.

### Graphics Model

- Internal 256×192 `videoPixel` array of `Pixel { ink: u8, point: u8 }` structs
- Per-pixel bit flags in `ink`: `B_LEVEL=1`, `B_ROBOT=2`, `B_WILLY=4`
- Text uses embedded control bytes: `\x01` = set paper (next byte = index), `\x02` = set ink
- `TILE2PIXEL(t)` macro converts tile coordinates to pixel buffer offsets

### Audio Model

- `Audio_Output` is called from C's SDL audio callback (22050 Hz, stereo i16)
- 8 channels: indices 0–2 are SFX slots, 3–7 are music channels
- Square wave oscillator: MSB of `phase` (u32) selects between two amplitude values
- Music scores are `&[i16]` slices with events encoded as `(channel | type, note, duration)` triplets; `EV_END` terminates and either loops (`MUS_PLAY`) or stops (`MUS_STOP`)
- `audioPanX` (exported to C) controls stereo position for SFX

## Debugging

Valgrind catches FFI memory mistakes that show up when porting C globals to Rust:

```sh
valgrind --error-exitcode=1 ./target/debug/jetsetrusty 2>&1 | head -40
```

## Room reference

`ROOMS.md` is the authoritative numbered list of all 60 rooms with their teleport-code key bindings (used by the "writetyper" cheat). Refer to it when working on level transitions, cheat code handling, or anything that names a specific room.
