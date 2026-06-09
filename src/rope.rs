#![allow(non_snake_case)]

/// rope.rs - Modernized version with proper FFI safety
use crate::common::{Event, MinerWilly, WIDTH};
use crate::game::{COLDSTORE, ONTHEROOF, QUIRKAFLEEG, SWIMMINGPOOL, THEBEACH};
use crate::video::{video_draw_rope_seg, video_draw_rope_seg_inner, video_get_pixel, VIDEO_PIXEL};
use std::sync::atomic::{AtomicI32, Ordering};

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
// Rope state - using Atomic types for thread-safe access
// ----------------------------------------------------------------------------

struct RopeState {
    dir: AtomicI32,
    pos: AtomicI32,
    hold: AtomicI32,
    x: AtomicI32,
    side: AtomicI32,
    ink: AtomicI32,
}

impl Default for RopeState {
    fn default() -> Self {
        Self {
            dir: AtomicI32::new(0),
            pos: AtomicI32::new(0),
            hold: AtomicI32::new(0),
            x: AtomicI32::new(0),
            side: AtomicI32::new(0),
            ink: AtomicI32::new(0),
        }
    }
}

static ROPE: RopeState = RopeState {
    dir: AtomicI32::new(0),
    pos: AtomicI32::new(0),
    hold: AtomicI32::new(0),
    x: AtomicI32::new(0),
    side: AtomicI32::new(0),
    ink: AtomicI32::new(0),
};

// ----------------------------------------------------------------------------
// C-visible function pointer variables (game.c calls these indirectly)
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut Rope_Ticker: Event = None;

#[unsafe(no_mangle)]
pub static mut Rope_Drawer: Event = None;

// ----------------------------------------------------------------------------
// Extern declarations - the real C globals shared with levels.rs and the C code
// ----------------------------------------------------------------------------

unsafe extern "C" {
    static gameLevel: i32;
    static mut minerWilly: MinerWilly;
    static mut minerWillyRope: i32;
}

// Level constants
const B_WILLY: i32 = 4;
const R_ABOVE: i32 = 0;

// ----------------------------------------------------------------------------
// Internal implementation
// ----------------------------------------------------------------------------

fn do_rope_drawer() {
    let mut data_idx = ROPE.pos.load(Ordering::Relaxed) as usize;
    let ink = ROPE.ink.load(Ordering::Relaxed) as u8;

    let mut x = ROPE.x.load(Ordering::Relaxed) * 8;
    let mut y: i32 = 0;

    video_draw_rope_seg(x, ink);

    if ROPE.pos.load(Ordering::Relaxed) == 0 {
        ROPE.side
            .store(ROPE.side.load(Ordering::Relaxed) ^ 1, Ordering::Relaxed);
    }

    let mut pixels = VIDEO_PIXEL.lock().unwrap();
    for seg in 1..ROPE_SEGS {
        let data = &ROPE_DATA[data_idx];
        y += data.y;
        x -= data.x * ROPE_MOVE[ROPE.side.load(Ordering::Relaxed) as usize];
        data_idx += 1;

        let pos = y * WIDTH + x;

        // Check for Willy collision
        let pixel_val = video_get_pixel(&mut pixels, pos);
        let willy_rope_zero = unsafe { minerWillyRope } == 0;
        if willy_rope_zero && (pixel_val & B_WILLY) != 0 {
            unsafe {
                minerWillyRope = seg;
            }
            ROPE.hold.store(1, Ordering::Relaxed);
        }

        // Handle Willy position if holding rope
        let willy_on_rope = unsafe { minerWillyRope } == seg;
        if willy_on_rope && ROPE.hold.load(Ordering::Relaxed) != 0 {
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

            unsafe {
                minerWilly.x = if frame < 2 { willy_x } else { willy_x - 8 };
                minerWilly.y = willy_y;
                minerWilly.frame = frame;
                minerWilly.tile = minerWilly.y / 8 * 32 + minerWilly.x / 8;
                minerWilly.align = 4;
            }
        }

        video_draw_rope_seg_inner(&mut pixels, pos, ink);
    }

    // Handle negative minerWillyRope
    if unsafe { minerWillyRope } < 0 {
        unsafe {
            minerWillyRope += 1;
        }
        ROPE.hold.store(0, Ordering::Relaxed);
        return;
    }

    // Handle rope movement when holding
    if ROPE.hold.load(Ordering::Relaxed) != 0 {
        let willy_moving = unsafe { minerWilly.r#move } != 0;
        if willy_moving {
            let dir = ROPE.dir.load(Ordering::Relaxed);
            let willy_dir = unsafe { minerWilly.dir };
            let seg = unsafe { minerWillyRope } + ROPE_MOVE[(dir ^ willy_dir) as usize];

            let level_dir = unsafe { Level_Dir(R_ABOVE) };
            let adjusted_seg = if level_dir == 0 && seg < 15 { 15 } else { seg };

            if adjusted_seg < ROPE_SEGS {
                unsafe {
                    minerWillyRope = adjusted_seg;
                }
                return;
            }

            unsafe {
                minerWillyRope = -16;
                minerWilly.y &= 124;
                minerWilly.air = 0;
            }
        }
    }
}

fn do_rope_ticker() {
    let dir = ROPE.dir.load(Ordering::Relaxed);
    let side = ROPE.side.load(Ordering::Relaxed);
    let step = ROPE_MOVE[(dir ^ side) as usize] * 2;

    let new_pos = ROPE.pos.load(Ordering::Relaxed) + step;
    ROPE.pos.store(new_pos, Ordering::Relaxed);

    if new_pos < 16 {
        ROPE.pos.store(new_pos + step, Ordering::Relaxed);
    } else if new_pos == 54 {
        ROPE.dir.store(dir ^ 1, Ordering::Relaxed);
    }
}

unsafe extern "C" fn rope_ticker_fn() {
    do_rope_ticker();
}

unsafe extern "C" fn rope_drawer_fn() {
    do_rope_drawer();
}

#[unsafe(no_mangle)]
pub extern "C" fn Rope_Init() {
    let level = unsafe { gameLevel };
    let (x, ink) = match level {
        QUIRKAFLEEG => (16, 6),
        ONTHEROOF => (16, 4),
        COLDSTORE => (16, 6),
        SWIMMINGPOOL => (16, 7),
        THEBEACH => (14, 5),
        _ => {
            unsafe {
                Rope_Ticker = Some(DoNothing);
                Rope_Drawer = Some(DoNothing);
            }
            return;
        }
    };

    ROPE.x.store(x, Ordering::Relaxed);
    ROPE.ink.store(ink, Ordering::Relaxed);
    ROPE.dir.store(0, Ordering::Relaxed);
    ROPE.pos.store(34, Ordering::Relaxed);
    ROPE.side.store(0, Ordering::Relaxed);
    ROPE.hold.store(0, Ordering::Relaxed);

    unsafe {
        Rope_Ticker = Some(rope_ticker_fn);
        Rope_Drawer = Some(rope_drawer_fn);
    }
}

extern "C" fn DoNothing() {}

unsafe extern "C" {
    fn Level_Dir(dir: i32) -> i32;
}
