#![allow(non_snake_case)]

/// rope.rs was ported over by talking with Claude Sonnet 4.6 Extended via the
/// web browser chat interface. I've been experimenting using LLMs for porting
/// these files, for example the Claude Code CLI tool, Aider, Emacs integrations
/// like gptel, ellama, and Aidermacs (this is being replaced with Emigo), along
/// with many different models through Ollama.
///
/// I think so far I've gotten the best results from the web-based Claude chat
/// client, actually. That said, obviously there's had to be some manual
/// verification and fixing (it doesn't compile out of the gate and throws a
/// bazillion warnings) but it's been faster and less expensive than using
/// Claude Code directly (for me). I like the thread-local state bundle struct,
/// although we'll have to see how this shakes out once everything has been
/// ported over and we get to remove all the C FFI code.
///
use crate::common::{MinerWilly, WIDTH};
use crate::game::{COLDSTORE, ONTHEROOF, QUIRKAFLEEG, SWIMMINGPOOL, THEBEACH};
use crate::video::{video_draw_rope_seg, video_get_pixel, VIDEO_PIXEL};
use std::sync::{LazyLock, Mutex};
use std::cell::Cell;

const ROPE_SEGS: i32 = 33;

// ----------------------------------------------------------------------------
// Rope animation data (immutable, no unsafe needed)
// ----------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct RopeData {
    x: i32,
    y: i32,
}

impl RopeData {
    const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

static ROPE_DATA: [RopeData; 86] = [
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(0, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(1, 3),
    RopeData::new(2, 3),
    RopeData::new(2, 3),
    RopeData::new(2, 3),
    RopeData::new(2, 3),
    RopeData::new(2, 2),
    RopeData::new(2, 3),
    RopeData::new(2, 3),
    RopeData::new(2, 2),
    RopeData::new(2, 3),
    RopeData::new(2, 2),
    RopeData::new(2, 3),
    RopeData::new(2, 2),
    RopeData::new(2, 3),
    RopeData::new(2, 2),
    RopeData::new(2, 2),
    RopeData::new(2, 2),
    RopeData::new(2, 3),
    RopeData::new(2, 2),
    RopeData::new(2, 2),
    RopeData::new(2, 2),
    RopeData::new(2, 2),
    RopeData::new(2, 2),
    RopeData::new(1, 2),
    RopeData::new(2, 2),
    RopeData::new(2, 2),
    RopeData::new(1, 2),
    RopeData::new(1, 2),
    RopeData::new(2, 2),
    RopeData::new(1, 2),
    RopeData::new(1, 2),
    RopeData::new(2, 2),
    RopeData::new(2, 2),
    RopeData::new(3, 2),
    RopeData::new(2, 2),
    RopeData::new(3, 2),
    RopeData::new(2, 2),
    RopeData::new(3, 2),
    RopeData::new(3, 2),
    RopeData::new(3, 2),
    RopeData::new(3, 2),
    RopeData::new(3, 2),
    RopeData::new(3, 2),
];

static ROPE_MOVE: [i32; 2] = [-1, 1];

// ----------------------------------------------------------------------------
// Rope state — thread_local + Cell means no unsafe for reads/writes
// ----------------------------------------------------------------------------

#[derive(Default)]
struct RopeState {
    dir: Cell<i32>,
    pos: Cell<i32>,
    hold: Cell<i32>,
    x: Cell<i32>,
    side: Cell<i32>,
    ink: Cell<u8>,
}

thread_local! {
    static ROPE: RopeState = RopeState::default();
}

macro_rules! rope_get {
    ($field:ident) => {
        ROPE.with(|r| r.$field.get())
    };
}

macro_rules! rope_set {
    ($field:ident, $val:expr) => {
        ROPE.with(|r| r.$field.set($val))
    };
}

// ----------------------------------------------------------------------------
// EVENT function pointers — exposed to C
// ----------------------------------------------------------------------------

use std::sync::LazyLock;

static ROPE_TICKER: LazyLock<Mutex<Option<extern "C" fn()>>> = LazyLock::new(|| Mutex::new(Some(DoNothing)));
static ROPE_DRAWER: LazyLock<Mutex<Option<extern "C" fn()>>> = LazyLock::new(|| Mutex::new(Some(DoNothing)));

// ----------------------------------------------------------------------------
// Extern declarations — things still living in C
// ----------------------------------------------------------------------------


static MINER_WILLY_ROPE: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(0));
static MINER_WILLY: LazyLock<Mutex<MinerWilly>> = LazyLock::new(|| Mutex::new(MinerWilly {
    x: 0,
    y: 0,
    tile: 0,
    align: 0,
    frame: 0,
    dir: 0,
    r#move: 0,
    air: 0,
    jump: 0,
}));


// Level constants — verified against game.h, these are the rope rooms
const B_WILLY: i32 = 4; // video.h
const R_ABOVE: i32 = 0; // game.h

#[inline]
fn yalign(y: i32) -> i32 {
    y & !7
}

// ----------------------------------------------------------------------------
// Internal implementation
// ----------------------------------------------------------------------------

fn do_rope_drawer() {
    let mut data_idx = rope_get!(pos) as usize;
    let ink = rope_get!(ink);

    let mut x = rope_get!(x) * 8;
    let mut y: i32 = 0;

    video_draw_rope_seg(x, ink);

    if rope_get!(pos) == 0 {
        rope_set!(side, rope_get!(side) ^ 1);
    }

    let mut pixels = VIDEO_PIXEL.lock().unwrap();
    for seg in 1..ROPE_SEGS {
        let data = &ROPE_DATA[data_idx];
        y += data.y;
        x -= data.x * ROPE_MOVE[rope_get!(side) as usize];
        data_idx += 1;

        let pos = y * WIDTH + x;

        // Check for Willy collision
        let pixel_val = video_get_pixel(&mut pixels, pos);
        let willy_rope_zero = *MINER_WILLY_ROPE.lock().unwrap() == 0;
        if willy_rope_zero && (pixel_val & B_WILLY) != 0 {
            *MINER_WILLY_ROPE.lock().unwrap() = seg;
            rope_set!(hold, 1);
        }

        // Handle Willy position if holding rope
        let willy_on_rope = *MINER_WILLY_ROPE.lock().unwrap() == seg;
        if willy_on_rope && rope_get!(hold) != 0 {
            let willy_x = x & 248;
            let willy_y = y - 8;

            let frame = if (x & 6) == 6 {
                1
            } else if (x & 4) != 0 {
                0
            } else {
                if (x & 2) != 0 {
                    3
                } else {
                    2
                }
            };

            MINER_WILLY.lock().unwrap().x = if frame < 2 { willy_x } else { willy_x - 8 };
            MINER_WILLY.lock().unwrap().y = willy_y;
            MINER_WILLY.lock().unwrap().frame = frame;
            MINER_WILLY.lock().unwrap().tile = MINER_WILLY.lock().unwrap().y / 8 * 32 + MINER_WILLY.lock().unwrap().x / 8;
            MINER_WILLY.lock().unwrap().align = yalign(y);
        }

        video_draw_rope_seg(pos, ink);
    }

    // Handle negative minerWillyRope
    if *MINER_WILLY_ROPE.lock().unwrap() < 0 {
        *MINER_WILLY_ROPE.lock().unwrap() += 1;
        rope_set!(hold, 0);
        return;
    }

    // Handle rope movement when holding
    if rope_get!(hold) != 0 {
        let willy_moving = MINER_WILLY.lock().unwrap().r#move != 0;
        if willy_moving {
            let dir = rope_get!(dir);
            let willy_dir = MINER_WILLY.lock().unwrap().dir;
            let seg = *MINER_WILLY_ROPE.lock().unwrap() + ROPE_MOVE[(dir ^ willy_dir) as usize];

            let level_dir = unsafe { Level_Dir(R_ABOVE) };
            let adjusted_seg = if level_dir == 0 && seg < 15 { 15 } else { seg };

            if adjusted_seg < ROPE_SEGS {
                *MINER_WILLY_ROPE.lock().unwrap() = adjusted_seg;
                return;
            }

            *MINER_WILLY_ROPE.lock().unwrap() = -16;
            MINER_WILLY.lock().unwrap().y &= 124;
            MINER_WILLY.lock().unwrap().air = 0;
        }
    }
}

fn do_rope_ticker() {
    let dir = rope_get!(dir);
    let side = rope_get!(side);
    let step = ROPE_MOVE[(dir ^ side) as usize] * 2;

    rope_set!(pos, rope_get!(pos) + step);

    if rope_get!(pos) < 16 {
        rope_set!(pos, rope_get!(pos) + step);
    } else if rope_get!(pos) == 54 {
        rope_set!(dir, dir ^ 1);
    }
}

#[no_mangle]
pub extern "C" fn rope_ticker_trampoline() {
    do_rope_ticker();
}

#[no_mangle]
pub extern "C" fn rope_drawer_trampoline() {
    do_rope_drawer();
}

// ----------------------------------------------------------------------------
// Public API
// ----------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn Rope_Init() {
    let level = unsafe { gameLevel };
    let (x, ink) = match level {
        QUIRKAFLEEG => (16, 6u8),
        ONTHEROOF => (16, 4u8),
        COLDSTORE => (16, 6u8),
        SWIMMINGPOOL => (16, 7u8),
        THEBEACH => (14, 5u8),
        _ => {
            *ROPE_TICKER.lock().unwrap() = Some(DoNothing);
            *ROPE_DRAWER.lock().unwrap() = Some(DoNothing);
            return;
        }
    };

    rope_set!(x, x);
    rope_set!(ink, ink);
    rope_set!(dir, 0);
    rope_set!(pos, 34);
    rope_set!(side, 0);
    rope_set!(hold, 0);

}
#[no_mangle]
pub extern "C" fn DoNothing() {}
unsafe extern "C" {
    static gameLevel: i32;
    fn Level_Dir(dir: i32) -> i32;
}
