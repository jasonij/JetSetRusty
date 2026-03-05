#![allow(dead_code)]

use crate::levels;

// Screen dimensions, i32 as per original C
pub const WIDTH: i32 = 256;
pub const HEIGHT: i32 = 192;

// Must match the MinerWilly struct layout in game.h exactly
// levels.rs is using this
#[repr(C)]
pub struct MinerWilly {
    pub x: i32,
    pub y: i32,
    pub tile: i32,
    pub align: i32,
    pub frame: i32,
    pub dir: i32,
    pub r#move: i32,
    pub air: i32,
    pub jump: i32,
}

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
    Action, Drawer, Responder, Ticker, gameInput, videoFlash,
    DoNothing, DoQuit, System_Rnd, System_SetPixel,
};

// Forward declarations of remaining C functions
unsafe extern "C" {
    pub fn Codes_Action();
    pub fn Title_Action();
    pub fn Game_Action();
    pub fn Die_Action();
    pub fn Gameover_Action();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Level_SetBorder() {
    levels::level_set_border();
}

pub fn system_set_pixel(pos: i32, ink: i32) {
    unsafe { System_SetPixel(pos, ink) }
}
