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
| `game.rs` | **In-progress** port of `game.c`/`game.h` — currently mostly `unimplemented!()` stubs and shared constants/enums. Don't assume any function here works yet; check the body. |

### Still in C (compiled via `build.rs`)

`game.c`, `miner.c`, `robots.c` — these still implement `Game_Action`, miner physics, and robot movement. `build.rs` compiles only those three; everything else linked in is from Rust.

Note: `src/game_main.c`, `src/title.c`, `src/levels.c`, `src/rope.c`, and `src/codes.c` exist on disk (originals before porting) but are **not compiled** and should not be edited — they're reference material for the in-flight ports.

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
