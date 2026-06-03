#![allow(unused)]
use crate::audio::{Audio_Play, MUS_STOP};
use crate::cheat::Cheat_Responder;
use crate::game_main::gameInput;
use crate::gameover::Gameover_Action;
use crate::levels;
use crate::misc::Timer;
use crate::rope;

use crate::cheat::cheatEnabled;
use crate::common::{
    // C globals
    c_game_clock_ticks,
    c_game_frame,
    c_game_inactivity_timer,
    c_game_level,
    c_game_lives,
    c_game_mode,
    c_game_music,
    c_game_paused,
    c_game_score_clock,
    c_game_score_items,
    c_game_timer,
    c_item_count,
    c_level_border,
    c_miner_attr_split,
    c_miner_willy,
    c_miner_willy_rope,

    // Types
    Event,
    Key,
    MinerWilly,

    // Constant(s)
    WIDTH,
};

use crate::game_main::{Action, DoNothing, Drawer, Responder, Ticker, TICKRATE};
use crate::levels::level_init;
use crate::misc::Timer_Set;
use crate::rope::Rope_Init;
use crate::title::Title_Action;

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

// Static callback variable(s)
pub static mut DO_CLOCK_UPDATE: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut ROPE_DRAWER: Option<extern "C" fn() -> ()> = None;
pub static mut ROPE_TICKER: Option<extern "C" fn() -> ()> = None;

// Probably a lot of functions go here!
unsafe extern "C" {
    fn DoDrawClock();
    fn DoDrawOnce();
    fn DoGameDrawer();
    fn DoGameTicker();
    fn DoPauseDrawer();
    fn DoPauseTicker();
    fn Game_DrawStatus();
    fn Miner_Save();
    fn Robots_Init();
    fn System_Border(x: i32);
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
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
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
//
// Installed as the `Ticker` by Game_Action and runs exactly once: it sets up the
// room, then swaps `Ticker` to DoGameTicker (still C) and `Action` to DoNothing —
// that DoNothing handoff is what stops the title's `game_start` from re-firing
// (and restarting the music) every frame. Must be `extern "C"` to fit `Event`.
//
// sync_c_to_rust() pulls the latest C globals (title/game_start wrote gameLevel,
// itemCount, &c. directly) into GAME_STATE before we read them; sync_rust_to_c()
// pushes our writes (gameFrame, gameTimer, minerAttrSplit, …) back out so the C
// ticker/drawer see them.
#[unsafe(no_mangle)]
pub extern "C" fn game_init_room() {
    sync_c_to_rust();

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
            Ticker = Some(DoGameTicker);
        }
        Action = Some(DoNothing);
    }

    sync_rust_to_c();
}

pub fn game_pause(state: bool) {
    let game = &*GAME_STATE;

    if (game.game_paused.load(Ordering::Relaxed) == state
        || game.mode.load(Ordering::Relaxed) >= GameMode::Running as u8)
    {
        return;
    }

    game.game_paused.store(state, Ordering::Relaxed);

    if (game.game_paused.load(Ordering::Relaxed)) {
        if (game.cheat_enabled.load(Ordering::Relaxed)) {
            unsafe { Ticker = Some(DoNothing) };
            unsafe { Drawer = Some(DoNothing) };
        } else {
            unsafe { Ticker = Some(DoPauseTicker) };
            unsafe { Drawer = Some(DoPauseDrawer) };
        }
        Audio_Play(MUS_STOP);
    } else {
        unsafe { Ticker = Some(DoGameTicker) };
        unsafe { Drawer = Some(DoGameDrawer) };
        Audio_Play(game.music.load(Ordering::Relaxed) as i32);

        game.inactivity_timer.store(0, Ordering::Relaxed);
        if (game.cheat_enabled.load(Ordering::Relaxed)) {
            unsafe { Game_DrawStatus() };
            unsafe {
                System_Border(
                    game.level_border.lock().unwrap()[game.level.load(Ordering::Relaxed) as usize],
                );
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn do_game_responder() {
    // Pull current C state in, act on GAME_STATE (incl. game_pause), push back out
    // so the still-C DoGameTicker/DoGameDrawer see pause/music/mode changes.
    sync_c_to_rust();

    let game = &*GAME_STATE;

    game.inactivity_timer.store(0, Ordering::Relaxed);

    let input = unsafe { gameInput };

    match input {
        x if x == Key::Pause as i32 => game_pause(!game.game_paused.load(Ordering::Relaxed)),
        x if x == Key::Mute as i32 => {
            let new_music = if game.music.load(Ordering::Relaxed) == 0 {
                1
            } else {
                0
            };
            game.music.store(new_music, Ordering::Relaxed);
            unsafe {
                Audio_Play(new_music as i32);
            }
        }
        x if x == Key::Escape as i32 => unsafe { Action = Some(Title_Action) },
        _ => {
            if let Some(handler) = unsafe { Cheat_Responder } {
                unsafe { handler() };
                // Cheat_Responder writes C state directly (gameLevel on a
                // level-select, cheatEnabled, Action = Game_InitRoom, &c).
                // Re-pull so the round-trip below doesn't clobber it with our
                // stale shadow, then re-apply this frame's inactivity reset.
                sync_c_to_rust();
                game.inactivity_timer.store(0, Ordering::Relaxed);
            }
        }
    }

    sync_rust_to_c();
}

// Ported from C's ClockTicker, but NOT yet wired in: the live game loop is still
// C's DoGameTicker, which calls C's ClockTicker (game.c). Hook this up when
// DoGameTicker itself is ported — and note it reads/writes GAME_STATE, so it'll
// need the sync bookends (or to run inside an already-synced Rust ticker).
#[unsafe(no_mangle)]
pub extern "C" fn clock_ticker() {
    // [ed: porting comments over directly]
    // 256 frames = 1 game minute
    // 19 game hours = 6.75... actual hours (19 * 60 * 256 / 12 / 60 / 60)
    // there's a guy on YouTube that can do it in less than 20m
    //  (2m15s using cheat mode)
    let game = &*GAME_STATE;
    // fetch_add returns the pre-increment value, so `< 256` matches C's
    // `gameClockTicks++ < 256` exactly (a tick fires on the 257th call).
    if game.clock_ticks.fetch_add(1, Ordering::Relaxed) < 256 {
        return;
    }

    game.clock_ticks.store(0, Ordering::Relaxed);

    let mut clock = game.score_clock.lock().unwrap();
    clock[0] += 1;
    if (clock[0] == 60) {
        clock[0] = 0;
        clock[1] += 1;
        if (clock[1] == 12) {
            clock[2] = 1 - clock[2];
            if (clock[2] == 0) && (game.mode.load(Ordering::Relaxed) < GameMode::Maria as u8) {
                unsafe {
                    Action = Some(Gameover_Action);
                }
            }
        } else if (clock[1] == 13) {
            clock[1] = 1;
        }
    }

    unsafe {
        DO_CLOCK_UPDATE = Some(DoDrawClock);
    }
}

fn sync_rust_to_c() {
    unsafe {
        // Atomic fields -> C globals
        c_game_level = GAME_STATE.level.load(Ordering::Relaxed);
        c_game_lives = GAME_STATE.lives.load(Ordering::Relaxed);
        c_game_mode = GAME_STATE.mode.load(Ordering::Relaxed) as i32;
        c_game_music = GAME_STATE.music.load(Ordering::Relaxed) as i32;
        c_game_paused = GAME_STATE.game_paused.load(Ordering::Relaxed) as i32;
        c_game_clock_ticks = GAME_STATE.clock_ticks.load(Ordering::Relaxed);
        c_game_frame = GAME_STATE.frame.load(Ordering::Relaxed);
        c_game_inactivity_timer = GAME_STATE.inactivity_timer.load(Ordering::Relaxed);
        c_item_count = GAME_STATE.item_count.load(Ordering::Relaxed);
        cheatEnabled = GAME_STATE.cheat_enabled.load(Ordering::Relaxed) as i32;
        c_miner_attr_split = GAME_STATE.miner_attr_split.load(Ordering::Relaxed);
        c_miner_willy_rope = GAME_STATE.miner_willy_rope.load(Ordering::Relaxed);

        // Mutex fields -> C globals
        c_level_border = *GAME_STATE.level_border.lock().unwrap();
        c_game_score_clock = *GAME_STATE.score_clock.lock().unwrap();
        c_game_score_items = *GAME_STATE.score_items.lock().unwrap();
        c_game_timer = GAME_STATE.timer.lock().unwrap().clone();
        c_miner_willy = GAME_STATE.miner.lock().unwrap().clone();
    }
}

fn sync_c_to_rust() {
    unsafe {
        // C globals -> Atomic fields
        GAME_STATE.level.store(c_game_level, Ordering::Relaxed);
        GAME_STATE.lives.store(c_game_lives, Ordering::Relaxed);
        GAME_STATE.mode.store(c_game_mode as u8, Ordering::Relaxed);
        GAME_STATE.music.store(c_game_music as u8, Ordering::Relaxed);
        GAME_STATE
            .game_paused
            .store(c_game_paused != 0, Ordering::Relaxed);
        GAME_STATE
            .clock_ticks
            .store(c_game_clock_ticks, Ordering::Relaxed);
        GAME_STATE.frame.store(c_game_frame, Ordering::Relaxed);
        GAME_STATE
            .inactivity_timer
            .store(c_game_inactivity_timer, Ordering::Relaxed);
        GAME_STATE.item_count.store(c_item_count, Ordering::Relaxed);
        GAME_STATE
            .cheat_enabled
            .store(cheatEnabled != 0, Ordering::Relaxed);
        GAME_STATE
            .miner_attr_split
            .store(c_miner_attr_split, Ordering::Relaxed);
        GAME_STATE
            .miner_willy_rope
            .store(c_miner_willy_rope, Ordering::Relaxed);

        // C globals -> Mutex fields
        *GAME_STATE.level_border.lock().unwrap() = c_level_border;
        *GAME_STATE.score_clock.lock().unwrap() = c_game_score_clock;
        *GAME_STATE.score_items.lock().unwrap() = c_game_score_items;
        *GAME_STATE.timer.lock().unwrap() = c_game_timer;
        *GAME_STATE.miner.lock().unwrap() = c_miner_willy;
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn Game_Action() {
    unsafe {
        Responder = Some(do_game_responder);
    }
    unsafe {
        // Runs once next frame, inits the room, then installs DoGameTicker and
        // sets Action = DoNothing. Matches C's `Ticker = Game_InitRoom`.
        Ticker = Some(game_init_room);
    }
    unsafe {
        Drawer = Some(DoGameDrawer);
    }
}
