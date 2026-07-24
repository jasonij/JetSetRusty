#![allow(dead_code)]

use crate::game::Miner;
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

// These `c_*` bindings alias the shared C-ABI globals (defined in cglobals.rs /
// miner.rs) via `#[link_name]`, so the `GAME_STATE` mirror and the modules that
// still read/write the raw globals directly (title/cheat/die/rope/levels/miner/
// robots) hit the same storage. `gameMode` is `int` in the raw global — keep it
// i32 here so we write the full word, and cast at the AtomicU8 boundary in
// game.rs. `cheatEnabled` is deliberately absent: it is Rust-owned (cheat.rs)
// and already a single shared symbol.
//
// Only globals with a reader outside game.rs remain here; the game.rs-only ones
// (music, frame, inactivity_timer, level_border, score_clock, score_items,
// timer) have been dissolved into GAME_STATE.
unsafe extern "C" {
    // Game state
    #[link_name = "gameLevel"]
    pub static mut c_game_level: i32;
    #[link_name = "gameLives"]
    pub static mut c_game_lives: i32;
    #[link_name = "gameMode"]
    pub static mut c_game_mode: i32;
    #[link_name = "gamePaused"]
    pub static mut c_game_paused: i32;
    #[link_name = "gameClockTicks"]
    pub static mut c_game_clock_ticks: i32;
    #[link_name = "itemCount"]
    pub static mut c_item_count: i32;
    #[link_name = "minerAttrSplit"]
    pub static mut c_miner_attr_split: i32;
    #[link_name = "minerWillyRope"]
    pub static mut c_miner_willy_rope: i32;

    // Structs
    #[link_name = "minerWilly"]
    pub static mut c_miner_willy: Miner;
}
