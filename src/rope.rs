#![allow(non_snake_case)]

/// rope.rs - Modernized version with proper FFI safety
use crate::common::{MinerWilly, WIDTH};
use crate::game::{COLDSTORE, ONTHEROOF, QUIRKAFLEEG, SWIMMINGPOOL, THEBEACH};
use crate::video::{video_draw_rope_seg, video_get_pixel, VIDEO_PIXEL};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

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
    RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3),
    RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3),
    RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3),
    RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3),
    RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3),
    RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3),
    RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3),
    RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(0, 3), RopeData::new(1, 3),
    RopeData::new(1, 3), RopeData::new(1, 3), RopeData::new(1, 3), RopeData::new(1, 3),
    RopeData::new(1, 3), RopeData::new(1, 3), RopeData::new(1, 3), RopeData::new(1, 3),
    RopeData::new(1, 3), RopeData::new(1, 3), RopeData::new(1, 3), RopeData::new(2, 3),
    RopeData::new(2, 3), RopeData::new(2, 3), RopeData::new(2, 2), RopeData::new(2, 3),
    RopeData::new(2, 2), RopeData::new(2, 3), RopeData::new(2, 2), RopeData::new(2, 3),
    RopeData::new(2, 2), RopeData::new(2, 2), RopeData::new(2, 2), RopeData::new(2, 3),
    RopeData::new(2, 2), RopeData::new(2, 2), RopeData::new(2, 2), RopeData::new(1, 2),
    RopeData::new(2, 2), RopeData::new(1, 2), RopeData::new(1, 2), RopeData::new(2, 2),
    RopeData::new(1, 2), RopeData::new(1, 2), RopeData::new(2, 2), RopeData::new(3, 2),
    RopeData::new(2, 2), RopeData::new(3, 2), RopeData::new(3, 2), RopeData::new(3, 2),
    RopeData::new(3, 2), RopeData::new(3, 2), RopeData::new(3, 2),
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
// FFI function pointers - stored as static Option
// ----------------------------------------------------------------------------

static mut ROPE_TICKER: Option<extern "C" fn()> = None;
static mut ROPE_DRAWER: Option<extern "C" fn()> = None;

// ----------------------------------------------------------------------------
// Extern declarations
// ----------------------------------------------------------------------------

static MINER_WILLY_ROPE: AtomicI32 = AtomicI32::new(0);
static MINER_WILLY: Mutex<MinerWilly> = Mutex::new(MinerWilly {
    x: 0,
    y: 0,
    tile: 0,
    align: 0,
    frame: 0,
    dir: 0,
    r#move: 0,
    air: 0,
    jump: 0,
});

// Level constants
const B_WILLY: i32 = 4;
const R_ABOVE: i32 = 0;

#[inline]
fn yalign(y: i32) -> i32 {
    y & !7
}

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
        ROPE.side.store(ROPE.side.load(Ordering::Relaxed) ^ 1, Ordering::Relaxed);
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
        let willy_rope_zero = MINER_WILLY_ROPE.load(Ordering::Relaxed) == 0;
        if willy_rope_zero && (pixel_val & B_WILLY) != 0 {
            MINER_WILLY_ROPE.store(seg, Ordering::Relaxed);
            ROPE.hold.store(1, Ordering::Relaxed);
        }

        // Handle Willy position if holding rope
        let willy_on_rope = MINER_WILLY_ROPE.load(Ordering::Relaxed) == seg;
        if willy_on_rope && ROPE.hold.load(Ordering::Relaxed) != 0 {
            let willy_x = x & 248;
            let willy_y = y - 8;

            let frame = if (x & 6) == 6 {
                1
            } else if (x & 4) != 0 {
                0
            } else {
                if (x & 2) != 0 { 3 } else { 2 }
            };

            let mut willy = MINER_WILLY.lock().unwrap();
            willy.x = if frame < 2 { willy_x } else { willy_x - 8 };
            willy.y = willy_y;
            willy.frame = frame;
            willy.tile = willy.y / 8 * 32 + willy.x / 8;
            willy.align = yalign(y);
        }

        video_draw_rope_seg(pos, ink);
    }

    // Handle negative minerWillyRope
    if MINER_WILLY_ROPE.load(Ordering::Relaxed) < 0 {
        MINER_WILLY_ROPE.fetch_add(1, Ordering::Relaxed);
        ROPE.hold.store(0, Ordering::Relaxed);
        return;
    }

    // Handle rope movement when holding
    if ROPE.hold.load(Ordering::Relaxed) != 0 {
        let willy_moving = MINER_WILLY.lock().unwrap().r#move != 0;
        if willy_moving {
            let dir = ROPE.dir.load(Ordering::Relaxed);
            let willy_dir = MINER_WILLY.lock().unwrap().dir;
            let seg = MINER_WILLY_ROPE.load(Ordering::Relaxed) + ROPE_MOVE[(dir ^ willy_dir) as usize];

            let level_dir = unsafe { Level_Dir(R_ABOVE) };
            let adjusted_seg = if level_dir == 0 && seg < 15 { 15 } else { seg };

            if adjusted_seg < ROPE_SEGS {
                MINER_WILLY_ROPE.store(adjusted_seg, Ordering::Relaxed);
                return;
            }

            MINER_WILLY_ROPE.store(-16, Ordering::Relaxed);
            let mut willy = MINER_WILLY.lock().unwrap();
            willy.y &= 124;
            willy.air = 0;
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

#[no_mangle]
pub extern "C" fn rope_ticker_trampoline() {
    do_rope_ticker();
}

#[no_mangle]
pub extern "C" fn rope_drawer_trampoline() {
    do_rope_drawer();
}

#[no_mangle]
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
                ROPE_TICKER = Some(DoNothing);
                ROPE_DRAWER = Some(DoNothing);
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
        ROPE_TICKER = Some(rope_ticker_trampoline);
        ROPE_DRAWER = Some(rope_drawer_trampoline);
    }
}

extern "C" {
    fn DoNothing();
    static gameLevel: i32;
    fn Level_Dir(dir: i32) -> i32;
}
