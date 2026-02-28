# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

A Rust port of the 1984 ZX Spectrum game "Jet Set Willy" (originally by Matthew Smith), porting an existing C/SDL2 implementation incrementally. Gameplay is 100% identical to the original. The project adds per-pixel coloring (no color clash), polyphonic music, and stereo SFX.

## Build & Run

```bash
cargo build           # debug build
cargo build --release # optimized
cargo run             # run the game
```

**Required system libraries:** `libsdl2-dev libsdl2-mixer-dev build-essential`

The `build.rs` script uses the `cc` crate to compile remaining C files and links them via pkg-config (SDL2, SDL2_mixer).

## Architecture

The game uses a **hybrid Rust/C approach** — modules are ported one at a time from C to Rust. The Rust entry point (`main.rs`) calls `game_main()`, a C function that drives the main loop via four function pointers:

```
Action    — current game state handler
Responder — input processor
Ticker    — logic/physics update
Drawer    — rendering
```

These are declared as `pub static mut` in `common.rs` and written to by both Rust and C code.

### Already ported to Rust (`src/*.rs`)

| Module | Role |
|--------|------|
| `common.rs` | Shared constants (`WIDTH=256`, `HEIGHT=192`), `Key` enum (`#[repr(C)]`), `Event` type alias, extern C declarations |
| `video.rs` | Full rendering engine — pixel buffer, sprite drawing, text with 2 character sets |
| `misc.rs` | 16-color palette, `Timer` struct, viewport/scaling utils |
| `cheat.rs` | "writetyper" cheat code detection, level selection (1–60) |
| `die.rs` | Death animation and life-loss sequence |

### Still in C (`src/*.c`)

`game_main.c`, `game.c`, `miner.c`, `robots.c`, `audio.c`, `levels.c`, `loader.c`, `title.c`, `gameover.c`, `rope.c`, `codes.c`

### FFI conventions

- Rust functions callable from C use `#[unsafe(no_mangle)]` and C-style names (e.g., `Video_DrawSprite`)
- C functions used from Rust are declared as `unsafe extern "C"` in `common.rs`
- All inter-language globals use `static mut` with `unsafe` blocks
- The `Key` enum and `Event` type are `#[repr(C)]` for ABI compatibility

### Graphics model

- Internal 256×192 pixel buffer (`videoPixel` array in `video.rs`)
- Per-pixel ink/paper tracking via bit flags: `B_LEVEL=1`, `B_ROBOT=2`, `B_WILLY=4`
- Two character sets: small (8px, 128 chars) and large (16px, 96 chars)
