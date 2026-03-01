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

The game uses a **hybrid Rust/C approach** — modules are ported one at a time. The Rust entry point (`main.rs`) calls `game_main()`, a C function in `game_main.c` that owns the SDL2 window/audio setup and drives the main loop via four global function pointers:

```
Action    — current game state handler (runs once per frame, then sets itself to DoNothing)
Responder — input processor
Ticker    — logic/physics update
Drawer    — rendering
```

These are declared as `pub static mut` globals in C (`game_main.c`) and referenced by Rust via `unsafe extern "C"` blocks in `common.rs`. Each state transition works by re-assigning these pointers (e.g., `Action = Some(Title_Action)`).

### Ported to Rust (`src/*.rs`)

| Module | Role |
|--------|------|
| `common.rs` | `WIDTH=256`, `HEIGHT=192`, `Key` enum (`#[repr(C)]`), `Event` type alias, extern C declarations for C-side globals |
| `video.rs` | Full rendering engine — `videoPixel` buffer, sprite drawing, two character sets (8px/128-char and 16px/96-char) |
| `misc.rs` | 16-color `videoColour` palette (`#[no_mangle]`), `Timer` struct, `Video_Viewport` |
| `cheat.rs` | "writetyper" cheat detection, level selection (1–60) |
| `die.rs` | Death animation and life-loss sequence |
| `audio.rs` | Square-wave synth — 8-channel mixer (3 SFX + 5 music), polyphonic sequencer, stereo panning |
| `title.rs` | Title screen — JSW logo, scrolling ticker, starts music |
| `gameover.rs` | Game over animation — boot kicks Willy, then returns to title |
| `loader.rs` | Loading screen — flashing text, loading bar, copy protection flow |

### Still in C (compiled via `build.rs`)

`game_main.c`, `game.c`, `miner.c`, `robots.c`, `levels.c`, `rope.c`, `codes.c`

Note: `src/title.c` exists on disk (the original before porting) but is **not compiled**.

### FFI Conventions

- Rust functions exported to C: `#[unsafe(no_mangle)]` with C-style names (e.g., `Video_DrawSprite`, `Audio_Init`)
- C functions called from Rust: declared as `unsafe extern "C"` in each Rust file (or `common.rs`)
- Shared globals: `static mut` with `#[unsafe(no_mangle)]`; Rust 2024 edition requires `&raw mut` / `&raw const` to take pointers to them (avoids `static_mut_refs` lint)
- `Key` enum and `Colour` struct are `#[repr(C)]` for ABI compatibility; `Colour` has a `_padding: u8` field to match the C layout

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
