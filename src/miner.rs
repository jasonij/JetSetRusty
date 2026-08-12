// Port of miner.c — Willy (the miner): input handling, jump/fall/walk physics,
// conveyor & ramp handling, collision, item pickup, and sprite rendering.
//
// This is a faithful, behaviour-preserving port. It now operates directly on
// GAME_STATE.miner (the Rust-owned state) rather than a C-ABI global. The
// minerWillyRope field lives in GAME_STATE.miner_willy_rope (atomic).
// Other modules (robots.rs, levels.rs, die.rs, rope.rs, cheat.rs) read miner
// state through GAME_STATE as well.

use crate::audio::{Audio_WillySfx, audioPanX};
use crate::common::Key;
use crate::die::Die_Action;
use crate::game::{
    Direction, GAME_STATE, GameMode, Game_ChangeLevel, Game_GotItem, Miner, NIGHTMAREROOM,
};
use crate::game_main::{Action, System_IsKey};
use crate::levels::{Level_EraseItem, Level_GetTileRamp, Level_GetTileType, TileType};
use crate::misc::{Timer, Timer_Set, Timer_Update};
use crate::video::{Video_DrawMiner, Video_DrawSprite};
use std::sync::atomic::Ordering;

const D_RIGHT: i32 = 0;
const D_LEFT: i32 = 1;

// Conveyor direction (game.h C_NONE/C_LEFT/C_RIGHT).
const C_NONE: i32 = 0;
const C_LEFT: i32 = 1;
const C_RIGHT: i32 = 2;

#[derive(Clone, Copy)]
struct Jump {
    jump: i32,
    tile: i32,
    align: i32,
    // sfx
    length: i32,
    pitch: i32,
}

const fn jmp(jump: i32, tile: i32, align: i32, length: i32, pitch: i32) -> Jump {
    Jump { jump, tile, align, length, pitch }
}

#[rustfmt::skip]
static MINER_SPRITE: [[u16; 16]; 16] = [
    [15360, 15360, 32256, 13312, 15872, 15360, 6144, 15360, 32256, 32256, 63232, 64256, 15360, 30208, 28160, 30464],
    [3840, 3840, 8064, 3328, 3968, 3840, 1536, 3840, 7040, 7040, 7040, 7552, 3840, 1536, 1536, 1792],
    [960, 960, 2016, 832, 992, 960, 384, 960, 2016, 2016, 3952, 4016, 960, 1888, 1760, 1904],
    [240, 240, 504, 208, 248, 240, 96, 240, 504, 1020, 2046, 1782, 248, 474, 782, 908],
    [3840, 3840, 8064, 2816, 7936, 3840, 1536, 3840, 8064, 16320, 32736, 28512, 7936, 23424, 28864, 12736],
    [960, 960, 2016, 704, 1984, 960, 384, 960, 2016, 2016, 3824, 3568, 960, 1760, 1888, 3808],
    [240, 240, 504, 176, 496, 240, 96, 240, 472, 472, 472, 440, 240, 96, 96, 224],
    [60, 60, 126, 44, 124, 60, 24, 60, 126, 126, 239, 223, 60, 110, 118, 238],
    [32768, 20480, 43008, 20480, 43008, 54528, 27136, 55040, 43648, 55232, 65472, 32256, 17408, 17408, 0, 0],
    [0, 0, 0, 0, 0, 11328, 7808, 16320, 10912, 22000, 11248, 24448, 4352, 8320, 0, 0],
    [0, 0, 0, 0, 0, 2832, 1952, 4080, 3432, 2748, 5500, 2784, 5440, 10816, 5120, 10240],
    [0, 0, 0, 0, 0, 712, 488, 1020, 682, 1375, 703, 1528, 272, 160, 0, 0],
    [0, 0, 0, 0, 0, 4928, 6016, 16320, 21824, 64160, 64832, 8096, 2176, 1280, 0, 0],
    [0, 0, 0, 0, 0, 2256, 1504, 4080, 5808, 15696, 16040, 1872, 680, 596, 40, 20],
    [0, 0, 0, 0, 0, 564, 376, 1020, 1364, 4010, 4052, 506, 136, 260, 0, 0],
    [3, 10, 21, 10, 21, 171, 86, 235, 341, 1003, 1023, 126, 34, 34, 0, 0],
];

#[rustfmt::skip]
static JUMP_INFO: [Jump; 18] = [
    jmp(-4, -32, 6, 5, 72),
    jmp(-4, 0, 4, 5, 74),
    jmp(-3, -32, 6, 4, 76),
    jmp(-3, 0, 6, 4, 78),
    jmp(-2, 0, 4, 3, 80),
    jmp(-2, -32, 6, 3, 82),
    jmp(-1, 0, 6, 2, 84),
    jmp(-1, 0, 6, 2, 86),
    jmp(0, 0, 6, 1, 88),
    jmp(0, 0, 6, 1, 88),
    jmp(1, 0, 6, 2, 86),
    jmp(1, 0, 6, 2, 84),
    jmp(2, 32, 4, 3, 82),
    jmp(2, 0, 6, 3, 80),
    jmp(3, 0, 6, 4, 78),
    jmp(3, 32, 4, 4, 76),
    jmp(4, 0, 6, 5, 74),
    jmp(4, 32, 4, 5, 72),
];

static MINER_SEQUENCE: [usize; 8] = [0, 1, 2, 3, 7, 6, 5, 4];

const MINER_ZERO: Miner = Miner {
    x: 0,
    y: 0,
    tile: 0,
    align: 0,
    frame: 0,
    dir: D_RIGHT,
    move_: 0,
    air: 0,
    jump: 0,
};

// File-static miner state (private; no C ABI needed).
static mut MINER_FRAME: usize = 0; // base row into MINER_SPRITE (0 or 8)
static mut MINER_SEQ_INDEX: u8 = 0;
static mut MINER_TIMER: Timer = Timer { rate: 0, acc: 0, remainder: 0, divisor: 0 };

// YALIGN macro from video.h.
const fn yalign(y: i32) -> i32 {
    4 | ((y & 4) >> 1) | (y & 2) | ((y & 1) << 1)
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_SetSeq(index: i32, speed: i32) {
    unsafe {
        Timer_Set(&raw mut MINER_TIMER, 1, speed);
        MINER_SEQ_INDEX = index as u8;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_IncSeq() {
    unsafe {
        MINER_SEQ_INDEX = MINER_SEQ_INDEX.wrapping_add(Timer_Update(&raw mut MINER_TIMER) as u8) & 7;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_DrawSeqSprite(pos: i32, paper: u8, ink: u8) {
    unsafe {
        let row = MINER_SEQUENCE[MINER_SEQ_INDEX as usize];
        Video_DrawSprite(pos, MINER_SPRITE[row].as_ptr(), paper, ink);
    }
}

// MINER_STORE is used for save/restore of miner state across room transitions.
// It's file-static and doesn't need C ABI.
static mut MINER_STORE: Miner = MINER_ZERO;

#[unsafe(no_mangle)]
pub extern "C" fn Miner_Restore() {
    let mut miner = GAME_STATE.miner.lock().unwrap();
    miner.x = unsafe { MINER_STORE.x };
    miner.y = unsafe { MINER_STORE.y };
    miner.tile = unsafe { MINER_STORE.tile };
    miner.align = unsafe { MINER_STORE.align };
    miner.frame = unsafe { MINER_STORE.frame };
    miner.dir = unsafe { MINER_STORE.dir };
    miner.move_ = unsafe { MINER_STORE.move_ };
    miner.air = unsafe { MINER_STORE.air };
    miner.jump = unsafe { MINER_STORE.jump };
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_Save() {
    let miner = GAME_STATE.miner.lock().unwrap();
    unsafe {
        MINER_STORE.x = miner.x;
        MINER_STORE.y = miner.y;
        MINER_STORE.tile = miner.tile;
        MINER_STORE.align = miner.align;
        MINER_STORE.frame = miner.frame;
        MINER_STORE.dir = miner.dir;
        MINER_STORE.move_ = miner.move_;
        MINER_STORE.air = miner.air;
        MINER_STORE.jump = miner.jump;
    }

    unsafe {
        MINER_FRAME = if GAME_STATE.level.load(Ordering::Relaxed) == NIGHTMAREROOM { 8 } else { 0 };
    }
}

fn is_solid(tile: i32, miner: &mut Miner) -> bool {
    if tile < 0 || tile == 512 {
        return false;
    }

    if unsafe { Level_GetTileType(tile as usize) } == TileType::Solid {
        return true;
    }

    if unsafe { Level_GetTileType((tile + 32) as usize) } == TileType::Solid {
        return true;
    }

    if tile + 64 > 511 {
        return false;
    }

    if unsafe { Level_GetTileType((tile + 64) as usize) } != TileType::Solid {
        return false;
    }

    if miner.align == 6 {
        return true;
    }

    if miner.air == 1 && miner.jump > 9 {
        miner.air = 0;
    }

    false
}

fn move_left_right() {
    let mut miner = GAME_STATE.miner.lock().unwrap();
    
    if miner.move_ == 0 {
        return;
    }

    if GAME_STATE.miner_willy_rope.load(Ordering::Relaxed) > 0 {
        return;
    }

    if miner.dir == D_RIGHT {
        if miner.frame < 3 {
            miner.frame += 1;
            return;
        }

        if miner.air == 0 {
            if unsafe { Level_GetTileRamp((miner.tile + 64) as usize) } == TileType::RampL {
                let y = 8;
                let offset = 32;
                if is_solid(miner.tile + offset + 2, &mut miner) {
                    return;
                }
                miner.y += y;
                miner.tile += offset;
                return;
            } else if unsafe { Level_GetTileRamp((miner.tile + 34) as usize) } == TileType::RampR {
                let y = -8;
                let offset = -32;
                if is_solid(miner.tile + offset + 2, &mut miner) {
                    return;
                }
                miner.y += y;
                miner.tile += offset;
                return;
            }
        }

        if miner.x == 30 * 8 {
            drop(miner);
            Game_ChangeLevel(Direction::Right as i32);
            return;
        }

        if is_solid(miner.tile + 2, &mut miner) {
            return;
        }

        miner.x += 8;
        miner.tile += 1;
        miner.frame = 0;
    } else if GAME_STATE.mode.load(Ordering::Relaxed) != GameMode::Running as u8 {
        if miner.frame > 0 {
            miner.frame -= 1;
            return;
        }

        if miner.air == 0 {
            if unsafe { Level_GetTileRamp((miner.tile + 31) as usize) } == TileType::RampL {
                let y = -8;
                let offset = -32;
                if is_solid(miner.tile + offset - 1, &mut miner) {
                    return;
                }
                miner.y += y;
                miner.tile += offset;
                return;
            } else if unsafe { Level_GetTileRamp((miner.tile + 65) as usize) } == TileType::RampR {
                let y = 8;
                let offset = 32;
                if is_solid(miner.tile + offset - 1, &mut miner) {
                    return;
                }
                miner.y += y;
                miner.tile += offset;
                return;
            }
        }

        if miner.x == 0 {
            drop(miner);
            Game_ChangeLevel(Direction::Left as i32);
            return;
        }

        if is_solid(miner.tile - 1, &mut miner) {
            return;
        }

        miner.x -= 8;
        miner.tile -= 1;
        miner.frame = 3;
    }
}

fn update_dir(convey_dir: i32) {
    let mut miner = GAME_STATE.miner.lock().unwrap();
    let mut dir = 0;

    if (System_IsKey(Key::Left as i32) != 0 || convey_dir == C_LEFT)
        && GAME_STATE.mode.load(Ordering::Relaxed) < GameMode::Running as u8
    {
        dir += 1;
    }

    if System_IsKey(Key::Right as i32) != 0
        || convey_dir == C_RIGHT
        || GAME_STATE.mode.load(Ordering::Relaxed) == GameMode::Running as u8
    {
        dir += 2;
    }

    if dir == 0 {
        miner.move_ = 0;
    } else if dir == 1 {
        if miner.dir == D_RIGHT {
            miner.dir = D_LEFT;
            miner.move_ = 0;
        } else {
            miner.move_ = 1;
        }
    } else if dir == 2 {
        if miner.dir == D_LEFT {
            miner.dir = D_RIGHT;
            miner.move_ = 0;
        } else {
            miner.move_ = 1;
        }
    }

    if System_IsKey(Key::Jump as i32) != 0 && GAME_STATE.mode.load(Ordering::Relaxed) < GameMode::Running as u8 {
        miner.air = 1;
        miner.jump = 0;
        if GAME_STATE.miner_willy_rope.load(Ordering::Relaxed) > 0 {
            GAME_STATE.miner_willy_rope.store(-16, Ordering::Relaxed);
            miner.y &= 120;
            miner.align = 4;
            miner.move_ = 1;
        }
    }
}

fn do_miner_ticker() {
    let mut miner = GAME_STATE.miner.lock().unwrap();
    let mut convey_dir = C_NONE;

    if GAME_STATE.miner_willy_rope.load(Ordering::Relaxed) > 0 {
        drop(miner);
        update_dir(convey_dir);
        return;
    }

    if miner.air == 1 {
        let jump_info = JUMP_INFO[miner.jump as usize];
        let y = miner.y + jump_info.jump;

        if y < 0 {
            drop(miner);
            Game_ChangeLevel(Direction::Above as i32);
            return;
        }

        let tile = miner.tile + jump_info.tile;
        if unsafe { Level_GetTileType(tile as usize) } == TileType::Solid
            || unsafe { Level_GetTileType((tile + 1) as usize) } == TileType::Solid
        {
            // we need to re-align Willy
            miner.y = (y + 8) & 120;
            miner.tile = tile + 32;
            miner.align = 4;

            miner.air = 2;
            miner.move_ = 0;
            return;
        }

        unsafe {
            audioPanX = miner.x;
        }
        Audio_WillySfx(jump_info.pitch, jump_info.length);

        miner.y = y;
        miner.tile = tile;
        miner.align = jump_info.align;
        miner.jump += 1;

        if miner.jump == 18 {
            miner.air = 6;
            return;
        }

        if miner.jump != 13 && miner.jump != 16 {
            drop(miner);
            move_left_right();
            return;
        }
    }

    if miner.align == 4 {
        let tile = miner.tile + 64;
        if tile & 512 != 0 {
            drop(miner);
            Game_ChangeLevel(Direction::Below as i32);
            return;
        }

        let type0 = unsafe { Level_GetTileType(tile as usize) };
        let type1 = unsafe { Level_GetTileType((tile + 1) as usize) };
        if type0 == TileType::Harm || type1 == TileType::Harm {
            if miner.air == 1
                && (type0 as i32 <= TileType::Space as i32
                    || type1 as i32 <= TileType::Space as i32)
            {
                drop(miner);
                move_left_right();
            } else {
                unsafe {
                    Action = Some(Die_Action);
                }
            }
            return;
        }

        if type0 as i32 > TileType::Space as i32 || type1 as i32 > TileType::Space as i32 {
            if miner.air >= 12 {
                unsafe {
                    Action = Some(Die_Action);
                }
                return;
            }

            miner.air = 0;

            if type0 == TileType::ConveyL || type1 == TileType::ConveyL {
                convey_dir = C_LEFT;
            } else if type0 == TileType::ConveyR || type1 == TileType::ConveyR {
                convey_dir = C_RIGHT;
            }

            drop(miner);
            update_dir(convey_dir);
            move_left_right();
            return;
        }
    }

    if miner.air == 1 {
        drop(miner);
        move_left_right();
        return;
    }

    miner.move_ = 0;
    if miner.air == 0 {
        miner.air = 2;
        return;
    }

    miner.air += 1;
    if miner.air == 16 {
        // this affects the falling sound effect
        miner.air = 12;
    }

    unsafe {
        audioPanX = miner.x;
    }
    Audio_WillySfx(78 - miner.air, 4);
    miner.y += 4;
    miner.align = 4;
    if miner.y & 7 != 0 {
        miner.align += 2;
    } else {
        miner.tile += 32;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_Ticker() {
    do_miner_ticker();

    let miner = GAME_STATE.miner.lock().unwrap();
    if miner.y < 0 {
        drop(miner);
        Game_ChangeLevel(Direction::Above as i32);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_Drawer() {
    let miner = GAME_STATE.miner.lock().unwrap();
    let mut offset = 0;
    let mut align = miner.align;

    if miner.air == 0 {
        if unsafe { Level_GetTileRamp((miner.tile + 64) as usize) } == TileType::RampL {
            offset = miner.frame << 1;
            align = yalign(offset);
        } else if unsafe { Level_GetTileRamp((miner.tile + 65) as usize) } == TileType::RampR {
            offset = 6 - (miner.frame << 1);
            align = yalign(offset);
        }
    }

    let row = unsafe { MINER_FRAME } + ((miner.dir << 2) | miner.frame) as usize;
    if Video_DrawMiner(
        ((miner.y + offset) << 8) | miner.x,
        MINER_SPRITE[row].as_ptr(),
        GAME_STATE.miner_attr_split.load(Ordering::Relaxed),
    ) != 0
    {
        unsafe {
            Action = Some(Die_Action);
        }
        return;
    }

    let mut tile = miner.tile;
    let mut adj = 1;
    for _ in 0..align {
        if unsafe { Level_GetTileType(tile as usize) } == TileType::Harm {
            unsafe {
                Action = Some(Die_Action);
            }
            return;
        }
        tile += adj;
        adj ^= 30;
    }

    let mut tile = miner.tile;
    let mut adj = 1;
    for _ in 0..align {
        if unsafe { Level_GetTileType(tile as usize) } == TileType::Item {
            Level_EraseItem(tile as usize);
            Game_GotItem();
        }
        tile += adj;
        adj ^= 30;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_Init() {
    let mut miner = GAME_STATE.miner.lock().unwrap();
    miner.x = 20 * 8;
    miner.y = 13 * 8;
    miner.tile = 13 * 32 + 20;
    miner.align = 4;
    miner.move_ = 0;
    miner.air = 0;
    drop(miner);

    Miner_Save();
}
