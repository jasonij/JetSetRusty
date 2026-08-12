use crate::common::{Action, Event, Key, MinerWilly};
use crate::game::{game_init_room, game_pause, Game_CheatEnabled, GAME_STATE};
use std::sync::atomic::{AtomicUsize, Ordering};

// These will move to game.rs when we port that
unsafe extern "C" {
    static mut gameInput: i32;
    static minerWilly: MinerWilly;
    fn System_IsKey(key: i32) -> i32;
}

const FIRSTLANDING: i32 = 28;

static CHEAT_CODE: &[u8] = b"writetyper";

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
pub static mut cheatEnabled: i32 = 0;

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
pub static mut Cheat_Responder: Event = Some(cheat_disabled);

#[unsafe(no_mangle)]
pub extern "C" fn Cheat_Enabled() {
    let mut level: i32 = 0;

    for i in 0..30 {
        if unsafe { System_IsKey(Key::K1 as i32 + i) } != 0 {
            level = i + 1;
            break;
        }
    }

    if unsafe { System_IsKey(Key::Enter as i32) } == 0 {
        game_pause(false);
        return;
    }

    if level == 0 {
        return;
    }

    if unsafe { System_IsKey(Key::LShift as i32) } != 0
        || unsafe { System_IsKey(Key::RShift as i32) } != 0
    {
        level += 30;
    }

    level -= 1;
    if level == GAME_STATE.level.load(Ordering::Relaxed) {
        return;
    }

    GAME_STATE.level.store(level, Ordering::Relaxed);

    unsafe {
        Action = Some(game_init_room);
    }
}

pub extern "C" fn cheat_disabled() {
    static CHEAT_POS: AtomicUsize = AtomicUsize::new(0);

    if GAME_STATE.level.load(Ordering::Relaxed) != FIRSTLANDING
        || unsafe { minerWilly.y } != 104
        || CHEAT_CODE[CHEAT_POS.load(Ordering::Relaxed)]
            != (unsafe { gameInput } - Key::A as i32 + b'a' as i32) as u8
    {
        game_pause(false);
        return;
    }

    CHEAT_POS.fetch_add(1, Ordering::Relaxed);

    if CHEAT_POS.load(Ordering::Relaxed) < CHEAT_CODE.len() {
        return;
    }

    Game_CheatEnabled();

    unsafe {
        Cheat_Responder = Some(Cheat_Enabled);
    }
}
