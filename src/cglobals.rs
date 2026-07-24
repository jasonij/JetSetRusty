// Transitional shared game globals, relocated verbatim from the now-deleted
// game.c. These keep their exact C names and C ABI via `#[unsafe(no_mangle)]`,
// so the referents still link unchanged: common.rs's `c_*` `#[link_name]`
// aliases and the per-file `extern` blocks in title/cheat/die/rope/levels.
//
// Scaffolding for the C→Rust migration. Milestone 2 dissolves each of these into
// GAME_STATE and deletes this file. The 7 globals only game.rs touched (music,
// frame, inactivity_timer, level_border, score_clock, score_items, timer) are
// already gone; what remains has at least one reader outside game.rs
// (title/cheat/die/rope/levels/miner/robots) and gets dissolved once those
// readers move onto GAME_STATE.
//
// (`#[no_mangle]` statics are exempt from the non_upper_case_globals lint, so
// the C names need no `#[allow]` — same as miner.rs's `minerWilly`.)

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
pub static mut itemCount: i32 = 0;
