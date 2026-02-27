use crate::common::{Action, Event, Key};
use std::sync::atomic::{AtomicUsize, Ordering};

// These will move to game.rs when we port that
unsafe extern "C" {
    static mut gameLevel: i32;
    static mut gameInput: i32;
    static minerWilly: MinerWilly;
    fn Game_Pause(x: i32);
    fn Game_InitRoom();
    fn Game_CheatEnabled();
    fn System_IsKey(key: i32) -> i32;
}

// Temporary until we port game.rs
#[repr(C)]
pub struct MinerWilly {
    pub x: i32,
    pub y: i32,
    pub tile: i32,
    pub align: i32,
    pub frame: i32,
    pub dir: i32,
    pub move_: i32, // 'move' is a Rust keyword so we suffix with _
    pub air: i32,
    pub jump: i32,
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
        unsafe {
            Game_Pause(0);
        }
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
    if level == unsafe { gameLevel } {
        return;
    }

    unsafe {
        gameLevel = level;
    }

    unsafe {
        Action = Some(Game_InitRoom);
    }
}

// #[unsafe(no_mangle)]
pub extern "C" fn cheat_disabled() {
    static CHEAT_POS: AtomicUsize = AtomicUsize::new(0);

    if unsafe { gameLevel } != FIRSTLANDING
        || unsafe { minerWilly.y } != 104
        || CHEAT_CODE[CHEAT_POS.load(Ordering::Relaxed)]
            != (unsafe { gameInput } - Key::A as i32 + b'a' as i32) as u8
    {
        unsafe {
            Game_Pause(0);
        }
        return;
    }

    CHEAT_POS.fetch_add(1, Ordering::Relaxed);

    if CHEAT_POS.load(Ordering::Relaxed) < CHEAT_CODE.len() {
        return;
    }

    unsafe {
        Game_CheatEnabled();
    }

    unsafe {
        Cheat_Responder = Some(Cheat_Enabled);
    }
}
