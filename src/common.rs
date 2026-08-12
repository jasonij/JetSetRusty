#![allow(dead_code)]

use crate::levels;

// Screen dimensions, i32 as per original C
pub const WIDTH: i32 = 256;
pub const HEIGHT: i32 = 192;

// Function pointer type — equivalent to typedef void (*EVENT)(void)
pub type Event = Option<unsafe extern "C" fn()>;

// Key codes enum
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Jump,
    Enter,
    LShift,
    RShift,
    K1,
    K2,
    K3,
    K4,
    K5,
    K6,
    K7,
    K8,
    K9,
    K0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Escape,
    Pause,
    Mute,
    Quit,
    Else,
    None,
}

// Globals defined in game_main.rs, re-exported here for convenience
pub use crate::game_main::{
    Action, DoNothing, DoQuit, Drawer, Responder, System_Rnd, System_SetPixel, Ticker, gameInput,
    videoFlash,
};

// Forward declarations of remaining C functions
unsafe extern "C" {
    pub fn Codes_Action();
    pub fn Title_Action();
    pub fn Die_Action();
    pub fn Gameover_Action();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Level_SetBorder() {
    levels::level_set_border();
}

pub fn system_set_pixel(pos: i32, ink: i32) {
    System_SetPixel(pos, ink)
}

// These `c_*` bindings previously aliased the shared C-ABI globals.
// All globals have now been dissolved into GAME_STATE. This block is kept
// empty as a placeholder and will be removed once the migration is complete.
//
// `cheatEnabled` is deliberately absent: it is Rust-owned (cheat.rs)
// and already a single shared symbol.
//
// All globals have been dissolved into GAME_STATE (music/frame/inactivity_timer/
// level_border/score_clock/score_items/timer, then clock_ticks/game_paused/
// item_count/lives, then miner_attr_split, then gameLevel, then gameMode,
// then minerWilly and minerWillyRope).
