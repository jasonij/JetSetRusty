// Transitional shared game globals, relocated verbatim from the now-deleted
// game.c. These keep their exact C names and C ABI via `#[unsafe(no_mangle)]`,
// so the referents still link unchanged: common.rs's `c_*` `#[link_name]`
// aliases and the per-file `extern` blocks in title/cheat/die/rope/levels.
//
// Scaffolding for the C -> Rust migration. Milestone 2 dissolves each of these into
// GAME_STATE and deletes this file. Only gameMode remains (title/miner/robots).
// It dissolves once those readers move onto GAME_STATE.
//
// (`#[no_mangle]` statics are exempt from the non_upper_case_globals lint, so
// the C names need no `#[allow]` — same as miner.rs's `minerWilly`.)

// C zero-initialized ints.
#[unsafe(no_mangle)]
pub static mut gameMode: i32 = 0;
