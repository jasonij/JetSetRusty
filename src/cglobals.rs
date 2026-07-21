// Transitional shared game globals, relocated verbatim from the now-gutted
// game.c. These 13 symbols keep their exact C names and C ABI via
// `#[unsafe(no_mangle)]`, so every existing referent still links to them
// unchanged: common.rs's `c_*` `#[link_name]` aliases, the sync functions in
// game.rs, and the per-file `extern` blocks in title/cheat/die/rope/levels.
//
// This module is scaffolding for the C→Rust migration. The endgame
// (Milestone 2) dissolves each of these into GAME_STATE and deletes this file;
// Willy's state already lives in miner.rs, so this is the last raw C-owned
// game state. Values here mirror game.c's static initializers exactly — do not
// "improve" them, or you change gameplay (e.g. gameScoreClock is 0,0,0 at load
// and set to 7:00 by Game_GameReset, not initialized to 7 here).
//
// (`#[no_mangle]` statics are exempt from the non_upper_case_globals lint, so
// the C names need no `#[allow]` — same as miner.rs's `minerWilly`.)

use crate::misc::Timer;

// C: `int gameMusic = MUS_PLAY;` — audio::MUS_PLAY == 1.
#[unsafe(no_mangle)]
pub static mut gameMusic: i32 = 1;

// C: `int gamePaused = 0;`
#[unsafe(no_mangle)]
pub static mut gamePaused: i32 = 0;

// C zero-initialized ints.
#[unsafe(no_mangle)]
pub static mut gameLevel: i32 = 0;
#[unsafe(no_mangle)]
pub static mut gameLives: i32 = 0;
#[unsafe(no_mangle)]
pub static mut gameMode: i32 = 0;
#[unsafe(no_mangle)]
pub static mut gameClockTicks: i32 = 0;
#[unsafe(no_mangle)]
pub static mut gameFrame: i32 = 0;
#[unsafe(no_mangle)]
pub static mut gameInactivityTimer: i32 = 0;
#[unsafe(no_mangle)]
pub static mut itemCount: i32 = 0;

// C: `char gameScoreItems;` / `char gameScoreClock[3];` — u8 to match the
// c_game_score_* aliases in common.rs.
#[unsafe(no_mangle)]
pub static mut gameScoreItems: u8 = 0;
#[unsafe(no_mangle)]
pub static mut gameScoreClock: [u8; 3] = [0, 0, 0];

// C: `TIMER gameTimer;` — zero-init; layout {rate, acc, remainder, divisor}.
#[unsafe(no_mangle)]
pub static mut gameTimer: Timer = Timer {
    rate: 0,
    acc: 0,
    remainder: 0,
    divisor: 0,
};

// C: `int levelBorder[60]` — per-room border colour, indexed by level.
#[unsafe(no_mangle)]
pub static mut levelBorder: [i32; 60] = [
    5, 4, 6, 2, 3, 1, 2, 1, 4, 2, //
    2, 4, 6, 5, 1, 3, 2, 1, 2, 1, //
    2, 1, 4, 4, 1, 1, 5, 2, 3, 2, //
    2, 2, 2, 2, 1, 1, 5, 6, 2, 2, //
    1, 1, 2, 5, 3, 4, 1, 2, 4, 5, //
    5, 2, 1, 2, 5, 1, 2, 2, 5, 5, //
];
