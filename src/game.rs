#![allow(unused)]
use crate::common::WIDTH;
use crate::levels;
use crate::misc::Timer;
use crate::rope;

use crate::game_main::{Action, DoNothing, Drawer, Ticker, TICKRATE};
use crate::levels::level_init;
use crate::misc::Timer_Set;
use crate::rope::Rope_Init;

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering};
use std::sync::{LazyLock, Mutex};

// Public constants (from game.h)
pub const LIVES: usize = (18 * 8 + 4) * WIDTH as usize + 4;
pub const STATUS: usize = 21 * 8 + 4;

// Level constants
pub const THEDRIVE: i32 = 4;
pub const QUIRKAFLEEG: i32 = 16;
pub const ONTHEROOF: i32 = 18;
pub const BALLROOMEAST: i32 = 20;
pub const COLDSTORE: i32 = 25;
pub const THECHAPEL: i32 = 27;
pub const FIRSTLANDING: i32 = 28;
pub const NIGHTMAREROOM: i32 = 29;
pub const SWIMMINGPOOL: i32 = 31;
pub const EASTWALL: i32 = 32;
pub const THEBATHROOM: i32 = 33;
pub const MASTERBEDROOM: i32 = 35;
pub const THEBEACH: i32 = 57;

// Probably a lot of functions go here!
unsafe extern "C" {
    fn DoGameDrawer();
    fn DoDrawOnce();
    fn Robots_Init();
    fn System_Border(x: i32);
    fn Miner_Save();
}

// Enums (from game.h)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    Normal = 0,
    Maria,
    Running,
    Toilet,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Above = 0,
    Right,
    Below,
    Left,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    Item = 0,
    Space,
    Solid,
    Floor,
    SolidFloor,
    ConveyL,
    ConveyR,
    RampL,
    RampR,
    RampLC,
    RampRC,
    Harm,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConveyorDir {
    None = 0,
    Left,
    Right,
}

// miner struct (from game.h)
#[derive(Debug, Default)]
pub struct Miner {
    // Initialize to all zeros w/ Default
    pub x: i32,
    pub y: i32,
    pub tile: i32,
    pub align: i32,
    pub frame: i32,
    pub dir: i32,
    pub move_: i32, // renamed from 'move' which is a Rust keyword
    pub air: i32,
    pub jump: i32,
}

// Game state structure (private)
pub struct GameState {
    pub miner: Mutex<Miner>,

    // Frequently accessed, thread-safe, atomic fields (cheap access)
    pub cheat_enabled: AtomicBool,
    pub clock_ticks: AtomicI32,
    pub frame: AtomicI32,
    pub game_paused: AtomicBool,
    pub inactivity_timer: AtomicI32,
    pub item_count: AtomicI32,
    pub level: AtomicI32,
    pub lives: AtomicI32,
    pub miner_attr_split: AtomicI32,
    pub miner_willy_rope: AtomicI32,
    pub mode: AtomicU8,
    pub music: AtomicU8,

    // How to use enums in Atomics:
    // MY_ENUM.store(MyEnum::B as i32, Ordering::Release);
    // let val = MyEnum::from_repr(MY_ENUM.load(Ordering::Acquire)).unwrap();

    // Less frequent, needs interior mutability, mutex-protected fields (more expensive)
    pub level_border: Mutex<[i32; 60]>,
    pub score_clock: Mutex<[u8; 3]>,
    pub score_items: Mutex<u8>,
    pub timer: Mutex<Timer>,
}

// Let's only do these for complicated ones
impl GameState {
    // Atomic operations (very fast)
    pub fn increment_clock_ticks(&self) {
        self.clock_ticks.fetch_add(1, Ordering::Relaxed);
    }
    pub fn increment_item_count(&self) {
        self.item_count.fetch_add(1, Ordering::Relaxed);
    }
    pub fn increment_score_items(&self) {
        let mut items = self.score_items.lock().unwrap();
        *items += 1;
    }

    // Timer operations
    pub fn get_timer(&self) -> Timer {
        self.timer.lock().unwrap().clone()
    }

    pub fn update_timer<F>(&self, f: F)
    where
        F: FnOnce(&mut Timer),
    {
        let mut timer = self.timer.lock().unwrap();
        f(&mut *timer);
    }
}

pub static GAME_STATE: LazyLock<GameState> = LazyLock::new(|| GameState {
    cheat_enabled: AtomicBool::new(false),
    clock_ticks: AtomicI32::new(0),
    frame: AtomicI32::new(0),
    game_paused: AtomicBool::new(false),
    inactivity_timer: AtomicI32::new(0),
    item_count: AtomicI32::new(0),
    level: AtomicI32::new(0),
    level_border: Mutex::new([0; 60]),
    lives: AtomicI32::new(7),
    miner: Mutex::new(Miner::default()),
    miner_attr_split: AtomicI32::new(6),
    miner_willy_rope: AtomicI32::new(0),
    mode: AtomicU8::new(0),
    music: AtomicU8::new(0),
    score_clock: Mutex::new([0, 7, 0]), // 7:00 AM
    score_items: Mutex::new(0),
    timer: Mutex::new(Timer::default()),
});

// Game functions
pub fn game_init_room() {
    // Calls to C functions (while still unported)
    level_init();
    unsafe {
        Robots_Init();
    }
    Rope_Init();

    // Read from GAME_STATE (Rust shadow of C state)
    let game = &*GAME_STATE;
    let current_level = game.level.load(Ordering::Relaxed);

    // System_border is a C function into which we send data from Rust
    // levelBorder[gameLevel] -> game.level_border[level]
    unsafe {
        System_Border(game.level_border.lock().unwrap()[current_level as usize]);
    }

    unsafe {
        Miner_Save();
    }

    // Write to GAME_STATE
    game.miner_attr_split.store(6, Ordering::Relaxed);
    if (game.level.load(Ordering::Relaxed) == SWIMMINGPOOL) {
        game.miner_attr_split.store(5, Ordering::Relaxed); // willy goes blue when underwater
    }

    // Timer
    Timer_Set(&mut *game.timer.lock().unwrap(), 12, TICKRATE);
    game.frame.store(1, Ordering::Relaxed);
    game.inactivity_timer.store(0, Ordering::Relaxed);
    game.miner_willy_rope.store(0, Ordering::Relaxed);

    // C Globals
    // Ticker, Drawer, Action: still C
    unsafe {
        if game.game_paused.load(Ordering::Relaxed) {
            Ticker = Some(DoNothing);
            Drawer = Some(DoDrawOnce);
        } else {
            Ticker = Some(DoGameDrawer);
        }
        Action = Some(DoNothing);
    }
}

// Rope
pub static mut ROPE_TICKER: Option<extern "C" fn() -> ()> = None;
pub static mut ROPE_DRAWER: Option<extern "C" fn() -> ()> = None;
