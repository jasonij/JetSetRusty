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
use crate::video::{video_draw_rope_seg, video_get_pixel, VIDEO_PIXEL};
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

#[unsafe(no_mangle)]
pub static mut Rope_Ticker: Option<unsafe extern "C" fn()> = None;

#[unsafe(no_mangle)]
pub static mut Rope_Drawer: Option<unsafe extern "C" fn()> = None;

// ----------------------------------------------------------------------------
// Extern declarations — things still living in C
// ----------------------------------------------------------------------------

unsafe extern "C" {
    static mut minerWillyRope: i32;
    static mut minerWilly: MinerWilly;

    static gameLevel: i32;
    fn Level_Dir(dir: i32) -> i32;
    fn DoNothing();
}

// Level constants — verified against game.h, these are the rope rooms
//
// See ../ROOMS.md for the complete list
//
// 1 is the off license (level 1)
// 0 is on a branch (level 10)
// a is the front door (level 11)
// t is the nightmare room (level 30)
// ! is the banyan tree (level 31)
// ) is the emergency generator (level 40)
// A is doctor Jones (level 41)
// T is the bow (level 60)

const QUIRKAFLEEG: i32 = 16; // g <return>
const ONTHEROOF: i32 = 18; // i <return>
const COLDSTORE: i32 = 25; // p <return>
const SWIMMINGPOOL: i32 = 31; // @ <return>
const THEBEACH: i32 = 57; // R <return>

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

        unsafe {
            if minerWillyRope == 0 && (video_get_pixel(&mut pixels, pos) & B_WILLY) != 0 {
                minerWillyRope = seg;
                rope_set!(hold, 1);
            }

            if minerWillyRope == seg && rope_get!(hold) != 0 {
                minerWilly.x = x & 248;
                minerWilly.y = y - 8;

                if (x & 6) == 6 {
                    minerWilly.frame = 1;
                } else if (x & 4) != 0 {
                    minerWilly.frame = 0;
                } else {
                    minerWilly.x -= 8;
                    if (x & 2) != 0 {
                        minerWilly.frame = 3;
                    } else {
                        minerWilly.frame = 2;
                    }
                }

                minerWilly.tile = minerWilly.y / 8 * 32 + minerWilly.x / 8;
                minerWilly.align = yalign(y); // y before deduction
            }

            video_draw_rope_seg(pos, ink);
        }
    }

    // negative minerWillyRope lets Willy jump/fall off the rope
    unsafe {
        if minerWillyRope < 0 {
            minerWillyRope += 1;
            rope_set!(hold, 0);
            return;
        }
    }

    unsafe {
        if rope_get!(hold) != 0 && minerWilly.r#move != 0 {
            let dir = rope_get!(dir);
            let mut seg = minerWillyRope + ROPE_MOVE[(dir ^ minerWilly.dir) as usize];

            if Level_Dir(R_ABOVE) == 0 && seg < 15 {
                seg = 15;
            }

            if seg < ROPE_SEGS {
                minerWillyRope = seg;
                return;
            }

            minerWillyRope = -16;
            minerWilly.y &= 124;
            minerWilly.air = 0;
        }
    }
}

unsafe fn do_rope_ticker() {
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

extern "C" fn rope_ticker_trampoline() {
    unsafe {
        do_rope_ticker();
    }
}

extern "C" fn rope_drawer_trampoline() {
    do_rope_drawer();
}

// ----------------------------------------------------------------------------
// Public API
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn Rope_Init() {
    let (x, ink) = match unsafe { gameLevel } {
        QUIRKAFLEEG => (16, 6u8),
        ONTHEROOF => (16, 4u8),
        COLDSTORE => (16, 6u8),
        SWIMMINGPOOL => (16, 7u8),
        THEBEACH => (14, 5u8),
        _ => {
            unsafe {
                Rope_Ticker = Some(DoNothing as unsafe extern "C" fn());
                Rope_Drawer = Some(DoNothing as unsafe extern "C" fn());
            }
            return;
        }
    };

    rope_set!(x, x);
    rope_set!(ink, ink);
    rope_set!(dir, 0);
    rope_set!(pos, 34);
    rope_set!(side, 0);
    rope_set!(hold, 0);

    unsafe {
        Rope_Ticker = Some(rope_ticker_trampoline);
        Rope_Drawer = Some(rope_drawer_trampoline);
    }
}
