#![allow(non_snake_case, dead_code, non_upper_case_globals)]

use crate::common::WIDTH;
use crate::video::{Video_DrawSprite, Video_PixelInkFill, Video_PixelPaperFill};

unsafe extern "C" {
    static mut gameLives: i32;
    static mut audioPanX: i32;
    static mut Action: Option<unsafe extern "C" fn()>;
    static mut Ticker: Option<unsafe extern "C" fn()>;
    static mut Drawer: Option<unsafe extern "C" fn()>;
    static mut Responder: Option<unsafe extern "C" fn()>;
    static mut minerWilly: MinerWilly;
    fn Gameover_Action();
    fn Game_Action();
    fn DoNothing();
    fn Miner_Restore();
    fn Audio_ReduceMusicSpeed();
    fn Audio_Sfx(sfx: i32);
    fn System_Border(colour: i32);
}

// TODO: move to audio when ported
const LIVES: i32 = (18 * 8 + 4) * WIDTH + 4; // Defined in game.h

// from audio.h
#[repr(C)]
enum Sfx {
    Item = 0,
    Die,
    Gameover,
    Arrow,
    Willy,
    None,
}

#[repr(C)]
struct MinerWilly {
    x: i32,
    // other fields we don't care about yet
}

static dieBlank: [u16; 16] = [0; 16];
static mut dieLevel: i32 = 0;

#[unsafe(no_mangle)]
extern "C" fn Die_Drawer() {
    Video_PixelInkFill(0, 128 * WIDTH, unsafe { dieLevel >> 1 } as u8);
}

#[unsafe(no_mangle)]
extern "C" fn Die_Ticker() {
    unsafe {
        dieLevel -= 1;
        if dieLevel > 0 {
            return;
        }

        if gameLives < 0 {
            Action = Some(Gameover_Action);
            return;
        }

        Video_DrawSprite(LIVES + gameLives * 16, dieBlank.as_ptr(), 0x0, 0x0);
        Miner_Restore();
        Audio_ReduceMusicSpeed();
        Action = Some(Game_Action);
    }
}

#[unsafe(no_mangle)]
extern "C" fn Die_Init() {
    unsafe {
        gameLives -= 1;
        dieLevel = 15;
        System_Border(0x0);
        Video_PixelPaperFill(0, 128 * WIDTH, 0x0);
        audioPanX = minerWilly.x;
        Audio_Sfx(Sfx::Die as i32);
        Ticker = Some(Die_Ticker);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Die_Action() {
    unsafe {
        Responder = Some(DoNothing);
        Ticker = Some(Die_Init);
        Drawer = Some(Die_Drawer);
        Action = Some(DoNothing);
    }
}
