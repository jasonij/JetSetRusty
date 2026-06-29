#![allow(unused)]
use crate::audio::{Audio_Music, Audio_Play, MUS_PLAY, MUS_STOP};
use crate::cheat::Cheat_Responder;
use crate::game_main::gameInput;
use crate::gameover::Gameover_Action;
use crate::levels::{self, Level_Dir, Level_Drawer};
use crate::misc::Timer;
use crate::rope;
use crate::video::{
    Video_CycleColours, Video_PixelInkFill, Video_PixelPaperFill, Video_WriteLarge,
};

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
    // C functions
    Level_SetBorder,
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

// Music constants (from audio.h enum: MUS_TITLE=0, MUS_GAME=1, MUS_LOADER=2)
const MUS_GAME: i32 = 1;

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

// life ink colors for drawing lives
const LIFE_INK: [u8; 7] = [0x2, 0x4, 0x6, 0x1, 0x3, 0x5, 0x7];

// Static callback variable(s)
pub static mut DO_CLOCK_UPDATE: Option<unsafe extern "C" fn() -> ()> = None;
pub static mut ROPE_DRAWER: Option<extern "C" fn() -> ()> = None;
pub static mut ROPE_TICKER: Option<extern "C" fn() -> ()> = None;

// Probably a lot of functions go here!
unsafe extern "C" {
    fn Level_Ticker();
    fn Miner_DrawSeqSprite(pos: i32, paper: u8, ink: u8);
    fn Miner_Drawer();
    fn Miner_IncSeq();
    fn Miner_Save();
    fn Miner_SetSeq(index: i32, speed: i32);
    fn Miner_Ticker();
    fn Robots_DrawCheat();
    fn Robots_Flush();
    fn Robots_Init();
    fn Robots_Ticker();
    fn Rope_Ticker();
    fn Robots_Drawer();
    fn System_Border(x: i32);
    fn Timer_Update(timer: *mut Timer) -> i32;
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
    pub game_paused: AtomicI32,
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
        *self.timer.lock().unwrap()
    }

    pub fn update_timer<F>(&self, f: F)
    where
        F: FnOnce(&mut Timer),
    {
        let mut timer = self.timer.lock().unwrap();
        f(&mut timer);
    }
}

// WARN: LLM Slop for debugging purposes
impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cheat_enabled = self.cheat_enabled.load(Ordering::Relaxed);
        let clock_ticks = self.clock_ticks.load(Ordering::Relaxed);
        let frame = self.frame.load(Ordering::Relaxed);
        let game_paused = self.game_paused.load(Ordering::Relaxed);
        let inactivity_timer = self.inactivity_timer.load(Ordering::Relaxed);
        let item_count = self.item_count.load(Ordering::Relaxed);
        let level = self.level.load(Ordering::Relaxed);
        let lives = self.lives.load(Ordering::Relaxed);
        let miner_attr_split = self.miner_attr_split.load(Ordering::Relaxed);
        let miner_willy_rope = self.miner_willy_rope.load(Ordering::Relaxed);
        let mode = self.mode.load(Ordering::Relaxed);
        let music = self.music.load(Ordering::Relaxed);

        let miner = self.miner.lock().unwrap();
        let level_border = self.level_border.lock().unwrap();
        let score_clock = self.score_clock.lock().unwrap();
        let score_items = self.score_items.lock().unwrap();
        let timer = self.timer.lock().unwrap();

        // This is so ugly
        write!(
            f,
            "GameState {{\n  cheat_enabled: {},\n  clock_ticks: {},\n  frame: {},\n  game_paused: {},\n  inactivity_timer: {},\n  item_count: {},\n  level: {},\n  lives: {},\n  miner_attr_split: {},\n  miner_willy_rope: {},\n  mode: {},\n  music: {},\n  miner: {:?},\n  level_border: {:?},\n  score_clock: {:?},\n  score_items: {},\n  timer: {:?}\n}}",
            cheat_enabled,
            clock_ticks,
            frame,
            game_paused,
            inactivity_timer,
            item_count,
            level,
            lives,
            miner_attr_split,
            miner_willy_rope,
            mode,
            music,
            *miner,
            *level_border,
            *score_clock,
            *score_items,
            *timer
        )
    }
}

pub static GAME_STATE: LazyLock<GameState> = LazyLock::new(|| GameState {
    cheat_enabled: AtomicBool::new(false),
    clock_ticks: AtomicI32::new(0),
    frame: AtomicI32::new(0),
    game_paused: AtomicI32::new(0),
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
    // Use scope to drop level_border guard before sync_rust_to_c()
    {
        let border_value = game.level_border.lock().unwrap()[current_level as usize];
        unsafe {
            System_Border(border_value);
        }
        // border guard dropped here at end of scope
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
    // Use scope to drop timer guard before sync_rust_to_c()
    {
        Timer_Set(&mut *game.timer.lock().unwrap(), 12, TICKRATE);
        // timer guard dropped here at end of scope
    }
    game.frame.store(1, Ordering::Relaxed);
    game.inactivity_timer.store(0, Ordering::Relaxed);
    game.miner_willy_rope.store(0, Ordering::Relaxed);

    // C Globals
    // Ticker, Drawer, Action: still C
    unsafe {
        if game.game_paused.load(Ordering::Relaxed) != 0 {
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

    if (game.game_paused.load(Ordering::Relaxed) != 0) == state
        || game.mode.load(Ordering::Relaxed) >= GameMode::Running as u8
    {
        return;
    }

    game.game_paused
        .store(if state { 1 } else { 0 }, Ordering::Relaxed);

    if game.game_paused.load(Ordering::Relaxed) != 0 {
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
        unsafe { Drawer = Some(do_game_drawer) };
        Audio_Play(game.music.load(Ordering::Relaxed) as i32);

        game.inactivity_timer.store(0, Ordering::Relaxed);
        if (game.cheat_enabled.load(Ordering::Relaxed)) {
            game_draw_status();
            // Use scope to drop level_border guard before function returns
            {
                let border = game.level_border.lock().unwrap();
                unsafe {
                    System_Border(border[game.level.load(Ordering::Relaxed) as usize]);
                }
                // border guard dropped here
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn do_game_responder() {
    // Pull current C state in, act on GAME_STATE (incl. game_pause), push back out
    // so the C code (miner/robots) sees pause/music/mode changes.
    sync_c_to_rust();

    let game = &*GAME_STATE;

    game.inactivity_timer.store(0, Ordering::Relaxed);

    let input = unsafe { gameInput };

    match input {
        x if x == Key::Pause as i32 => game_pause(game.game_paused.load(Ordering::Relaxed) == 0),
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

#[unsafe(no_mangle)]
pub extern "C" fn do_game_drawer() {
    sync_c_to_rust();
    let game = &*GAME_STATE;

    if (game.music.load(Ordering::Relaxed) == MUS_PLAY as u8) {
        game_draw_lives();
    }

    if (game.frame.load(Ordering::Relaxed) == 0) {
        return;
    }

    // WARN: This maybe should be lower
    // TODO: Test the victory sequence!
    if (game.mode.load(Ordering::Relaxed) == GameMode::Toilet as u8) {
        return;
    }

    unsafe {
        sync_rust_to_c();
        Level_Drawer();
        Robots_Drawer();
        Miner_Drawer();

        // Remember we've got function-pointer globals, so let's remember this
        // pattern and not accidentally jump into the variable's storage
        if let Some(func) = rope::Rope_Drawer {
            func();
        }
        if let Some(func) = DO_CLOCK_UPDATE {
            func();
        }

        sync_c_to_rust();
    }
}

pub fn do_draw_clock() {
    // NOTE: Caller must have synced state
    let game = &*GAME_STATE;

    let mut text: Vec<u8> = vec![
        0x1, 0x0, 0x2, 0x7, b' ', 0x2, 0x6, b' ', 0x2, 0x5, b':', 0x2, 0x4, b' ', 0x2, 0x3, b' ',
        0x2, 0x2, b' ', 0x2, 0x1, b'm', 0,
    ];

    let score_clock = *game.score_clock.lock().unwrap();

    text[19] = if score_clock[2] != 0 { b'p' } else { b'a' };
    text[16] = (score_clock[0] % 10) + b'0';
    text[13] = (score_clock[0] / 10) + b'0';
    text[7] = (score_clock[1] % 10) + b'0';
    if score_clock[1] > 9 {
        text[4] = (score_clock[1] / 10) + b'0';
    }

    unsafe {
        Video_WriteLarge(
            (WIDTH - 60) as i32,
            STATUS as i32,
            text.as_ptr() as *const i8,
        );
        DO_CLOCK_UPDATE = Some(DoNothing);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn DoDrawClock() {
    sync_c_to_rust();
    do_draw_clock();
    sync_rust_to_c();
}

#[unsafe(no_mangle)]
pub extern "C" fn DoPauseTicker() {
    sync_c_to_rust();
    let game = &*GAME_STATE;

    let old_paused = game.game_paused.fetch_add(1, Ordering::Relaxed);
    if old_paused == 16 * 5 {
        game.game_paused.store(1, Ordering::Relaxed);
    }

    sync_rust_to_c();
}

#[unsafe(no_mangle)]
pub extern "C" fn DoPauseDrawer() {
    sync_c_to_rust();
    let game = &*GAME_STATE;

    if game.game_paused.load(Ordering::Relaxed) == 16 * 5 {
        unsafe {
            Level_SetBorder();
            Video_CycleColours();
        }
    }

    sync_rust_to_c();
}

#[unsafe(no_mangle)]
pub extern "C" fn Game_GameReset() {
    sync_c_to_rust();
    let game = &*GAME_STATE;

    // gameScoreItems = 0
    *game.score_items.lock().unwrap() = 0;

    // gameScoreClock[0] = 0; gameScoreClock[1] = 7; gameScoreClock[2] = 0
    // Use scope to drop clock guard before sync_rust_to_c()
    {
        let mut clock = game.score_clock.lock().unwrap();
        clock[0] = 0;
        clock[1] = 7;
        clock[2] = 0;
        // clock guard dropped here at end of scope
    }

    // DoClockUpdate = DoDrawClock
    unsafe {
        DO_CLOCK_UPDATE = Some(DoDrawClock);
    }

    // gameClockTicks = 0
    game.clock_ticks.store(0, Ordering::Relaxed);

    // gamePaused = 0
    game.game_paused.store(0, Ordering::Relaxed);

    // Miner_SetSeq(0, 20)
    unsafe {
        Miner_SetSeq(0, 20);
    }

    // gameLives = 7
    game.lives.store(7, Ordering::Relaxed);

    // Audio_Music(MUS_GAME, gameMusic)
    let music = game.music.load(Ordering::Relaxed);
    unsafe {
        Audio_Music(MUS_GAME, music as i32);
    }

    sync_rust_to_c();
}

#[unsafe(no_mangle)]
pub extern "C" fn Game_CheatEnabled() {
    sync_c_to_rust();
    let game = &*GAME_STATE;

    if game.game_paused.load(Ordering::Relaxed) != 0 {
        // gameFrame = 1
        game.frame.store(1, Ordering::Relaxed);

        // Ticker = DoNothing; Drawer = DoNothing;
        unsafe {
            Ticker = Some(DoNothing);
            Drawer = Some(DoNothing);
        }

        // Game_DrawStatus(); System_Border(levelBorder[gameLevel])
        game_draw_status();
        let level = game.level.load(Ordering::Relaxed);
        unsafe {
            System_Border(game.level_border.lock().unwrap()[level as usize]);
        }
    }

    // cheatEnabled = 1
    game.cheat_enabled.store(true, Ordering::Relaxed);

    // Robots_DrawCheat()
    unsafe {
        Robots_DrawCheat();
    }

    sync_rust_to_c();
}

#[unsafe(no_mangle)]
pub extern "C" fn Game_ChangeLevel(dir: i32) {
    sync_c_to_rust();
    let game = &*GAME_STATE;

    // int level = Level_Dir(dir)
    let level = unsafe { Level_Dir(dir as usize) };

    // Special case for R_ABOVE from THEDRIVE or FIRSTLANDING
    if dir == Direction::Above as i32 {
        let miner = game.miner.lock().unwrap();
        if (level == THEDRIVE && miner.x > 22 && miner.x < 32)
            || (level == FIRSTLANDING && miner.x > 182)
        {
            // minerWilly.air = 2; return
            drop(miner); // release the lock before we mutate
            {
                let mut miner = game.miner.lock().unwrap();
                miner.air = 2;
                // miner guard dropped here at end of scope
            }
            sync_rust_to_c();
            return;
        }
    }

    // gameLevel = level
    game.level.store(level, Ordering::Relaxed);

    // Update minerWilly based on direction
    // Use a scope to drop the lock before calling game_init_room()
    {
        let mut miner = game.miner.lock().unwrap();
        match dir {
            x if x == Direction::Above as i32 => {
                // minerWilly.y = 13 * 8 = 104
                miner.y = 13 * 8;
                // minerWilly.x = (minerWilly.tile & 31) * 8
                miner.x = (miner.tile & 31) * 8;
                // minerWilly.tile = 13 * 32 + (minerWilly.tile & 31)
                miner.tile = 13 * 32 + (miner.tile & 31);
                // minerWilly.align = 4
                miner.align = 4;
                // minerWilly.air = 0
                miner.air = 0;
            }
            x if x == Direction::Right as i32 => {
                // minerWilly.x = 0
                miner.x = 0;
                // minerWilly.tile &= ~31
                miner.tile &= !31;
            }
            x if x == Direction::Below as i32 => {
                // if (minerWilly.air < 11) { minerWilly.air = 2; }
                if miner.air < 11 {
                    miner.air = 2;
                }
                // minerWilly.y = 0
                miner.y = 0;
                // minerWilly.tile &= 31
                miner.tile &= 31;
            }
            x if x == Direction::Left as i32 => {
                // minerWilly.x = 30 * 8 = 240
                miner.x = 30 * 8;
                // minerWilly.tile |= 30
                miner.tile |= 30;
            }
            _ => {}
        }
        // miner guard dropped here at end of scope
    }

    // Game_InitRoom()
    game_init_room();

    sync_rust_to_c();
}

#[unsafe(no_mangle)]
pub extern "C" fn DoDrawOnce() {
    sync_c_to_rust();

    do_game_drawer();

    unsafe {
        Drawer = Some(DoNothing);
    }

    sync_rust_to_c();
}

#[unsafe(no_mangle)]
pub extern "C" fn DoGameTicker() {
    sync_c_to_rust();
    let game = &*GAME_STATE;

    // Inactivity check: pause after 5 minutes when music stopped
    if game.music.load(Ordering::Relaxed) == MUS_STOP as u8 {
        let old = game.inactivity_timer.fetch_add(1, Ordering::Relaxed);
        if old == 256 * 5 && game.mode.load(Ordering::Relaxed) < GameMode::Running as u8 {
            game_pause(true);
            sync_rust_to_c();
            return;
        }
    }

    // Miner animation
    if game.music.load(Ordering::Relaxed) == MUS_PLAY as u8 {
        unsafe {
            Miner_IncSeq();
        }
    }

    // Update game frame
    // Use scope to drop timer guard before sync_rust_to_c()
    let frame = unsafe {
        let mut timer = game.timer.lock().unwrap();
        Timer_Update(&mut *timer)
    };
    game.frame.store(frame, Ordering::Relaxed);
    if frame == 0 {
        sync_rust_to_c();
        return;
    }

    // Tick level and robots
    unsafe {
        Level_Ticker();
        Robots_Ticker();
    }

    // GM_TOILET mode: return to title after timeout
    if game.mode.load(Ordering::Relaxed) == GameMode::Toilet as u8 {
        let old_ticks = game.clock_ticks.fetch_add(1, Ordering::Relaxed);
        if old_ticks == 256 {
            unsafe {
                Action = Some(Title_Action);
            }
        }
        sync_rust_to_c();
        return;
    }

    // Tick miner
    unsafe {
        Miner_Ticker();
    }

    // GM_RUNNING mode
    if game.mode.load(Ordering::Relaxed) == GameMode::Running as u8 {
        // Use scope to ensure miner guard is dropped before sync_rust_to_c()
        {
            let mut miner = game.miner.lock().unwrap();
            miner.frame |= 1;

            if miner.x == 224 && game.level.load(Ordering::Relaxed) == THEBATHROOM {
                drop(miner);
                game.mode.store(GameMode::Toilet as u8, Ordering::Relaxed);
                unsafe {
                    Robots_Flush();
                }
                game.clock_ticks.store(0, Ordering::Relaxed);
            }
            // miner guard dropped here at end of scope
        }

        sync_rust_to_c();
        return;
    }

    // GM_MARIA mode: check for victory
    if game.mode.load(Ordering::Relaxed) == GameMode::Maria as u8
        && game.level.load(Ordering::Relaxed) == MASTERBEDROOM
    {
        // Use scope to ensure miner guard is dropped before sync_rust_to_c()
        {
            let miner = game.miner.lock().unwrap();
            if miner.air == 0 && miner.x == 40 {
                drop(miner);
                game.mode.store(GameMode::Running as u8, Ordering::Relaxed);
            }
            // miner guard dropped here at end of scope
        }
    }

    // Tick rope
    unsafe {
        if let Some(f) = rope::Rope_Ticker {
            f();
        }
    }

    // Tick clock - use Rust version now
    clock_ticker();

    sync_rust_to_c();
}

#[unsafe(no_mangle)]
pub extern "C" fn Game_GotItem() {
    sync_c_to_rust();
    let game = &*GAME_STATE;

    // gameScoreItems++
    *game.score_items.lock().unwrap() += 1;

    // Game_DrawStatus()
    game_draw_status();

    // if (--itemCount == 0) { gameMode = GM_MARIA; }
    if game.item_count.fetch_sub(1, Ordering::Relaxed) == 1 {
        game.mode.store(GameMode::Maria as u8, Ordering::Relaxed);
    }

    // audioPanX = minerWilly.x
    // Read x and drop the guard immediately — holding it past here would
    // self-deadlock when sync_rust_to_c() re-locks GAME_STATE.miner below.
    let miner_x = game.miner.lock().unwrap().x;
    unsafe {
        crate::audio::audioPanX = miner_x;
    }

    // Audio_Sfx(SFX_ITEM)
    unsafe {
        crate::audio::Audio_Sfx(0);
    }

    sync_rust_to_c();
}

// Ported from C's ClockTicker, but NOT yet wired in: the live game loop is still
// C's DoGameTicker, which calls C's ClockTicker (game.c). Hook this up when
// DoGameTicker itself is ported — and note it reads/writes GAME_STATE, so it'll
// need the sync bookends (or to run inside an already-synced Rust ticker).
#[unsafe(no_mangle)]
pub extern "C" fn clock_ticker() {
    sync_c_to_rust();

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

    // Use scope to drop clock guard before sync_rust_to_c()
    {
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
        // clock guard dropped here
    }

    unsafe {
        DO_CLOCK_UPDATE = Some(DoDrawClock);
    }
    sync_rust_to_c();
}

pub fn game_draw_lives() {
    // NOTE: Caller must have synced state
    let game = &*GAME_STATE;
    let lives = game.lives.load(Ordering::Relaxed) as usize;

    for l in 0..lives {
        let pos = LIVES + l * 16;
        unsafe {
            Miner_DrawSeqSprite(pos as i32, 0x0, LIFE_INK[l]);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn GameDrawLives() {
    sync_c_to_rust();
    game_draw_lives();
    sync_rust_to_c();
}

pub fn game_draw_status() {
    // NOTE: Caller must have synced state (Game_DrawStatus or Game_GotItem)
    let game = &*GAME_STATE;

    // Video_PixelPaperFill(128 * WIDTH, 64 * WIDTH, 0x0);
    unsafe {
        Video_PixelPaperFill(128 * WIDTH, 64 * WIDTH, 0x0);
    }

    // Video_PixelInkFill(129 * WIDTH, 8 * WIDTH, 0x6);
    unsafe {
        Video_PixelInkFill(129 * WIDTH, 8 * WIDTH, 0x6);
    }

    // Video_WriteLarge(4, STATUS, "\x1\x0\x2\x1" "I" "\x2\x2" "t" "\x2\x3" "e" "\x2\x4" "m" "\x2\x5" "s");
    // Using vec! for the byte string as requested
    let items_label: Vec<u8> = vec![
        0x1, 0x0, 0x2, 0x1, b'I', 0x2, 0x2, b't', 0x2, 0x3, b'e', 0x2, 0x4, b'm', 0x2, 0x5, b's', 0,
    ];
    unsafe {
        Video_WriteLarge(4, STATUS as i32, items_label.as_ptr() as *const i8);
    }

    // DrawItems() equivalent
    let score_items = *game.score_items.lock().unwrap();
    let mut items_text: Vec<u8> = vec![0x1, 0x0, 0x2, 0x6, b' ', 0x2, 0x7, b' ', 0];

    items_text[7] = (score_items % 10) + b'0';
    if score_items > 9 {
        items_text[4] = (score_items / 10) + b'0';
    }

    unsafe {
        Video_WriteLarge(6 * 8 + 4, STATUS as i32, items_text.as_ptr() as *const i8);
    }

    // DoDrawClock();
    do_draw_clock();

    // GameDrawLives();
    game_draw_lives();
}

#[unsafe(no_mangle)]
pub extern "C" fn Game_DrawStatus() {
    sync_c_to_rust();
    game_draw_status();
    sync_rust_to_c();
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
        c_game_timer = *GAME_STATE.timer.lock().unwrap();
        c_miner_willy = *GAME_STATE.miner.lock().unwrap();
    }
}

fn sync_c_to_rust() {
    unsafe {
        // C globals -> Atomic fields
        GAME_STATE.level.store(c_game_level, Ordering::Relaxed);
        GAME_STATE.lives.store(c_game_lives, Ordering::Relaxed);
        GAME_STATE.mode.store(c_game_mode as u8, Ordering::Relaxed);
        GAME_STATE
            .music
            .store(c_game_music as u8, Ordering::Relaxed);
        GAME_STATE
            .game_paused
            .store(c_game_paused, Ordering::Relaxed);
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
    let game = &*GAME_STATE;
    unsafe {
        Responder = Some(do_game_responder);
        // Runs once next frame, inits the room, then installs DoGameTicker and
        // sets Action = DoNothing. Matches C's `Ticker = Game_InitRoom`.
        Ticker = Some(game_init_room);
        Drawer = Some(do_game_drawer);
    }
}
