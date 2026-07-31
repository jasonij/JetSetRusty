// Port of miner.c — Willy (the miner): input handling, jump/fall/walk physics,
// conveyor & ramp handling, collision, item pickup, and sprite rendering.
//
// This is a faithful, behaviour-preserving port. It still operates on the raw
// `minerWilly` global (not GAME_STATE) exactly as the C did, because it runs
// mid-frame inside DoGameTicker/do_game_drawer while the C globals are the live
// source of truth (see the GAME_STATE sync model in CLAUDE.md). still-C
// robots.c reads minerWilly.{y,air}, and levels.rs/die.rs/rope.rs/cheat.rs all
// import `minerWilly`, so `minerWilly` / `minerWillyRope` remain #[no_mangle]
// C-ABI globals defined here until robots.c is ported. `minerAttrSplit` is
// owned by GAME_STATE.miner_attr_split (game.rs) — read via GAME_STATE here
// rather than through a shared global.

use crate::audio::{Audio_WillySfx, audioPanX};
use crate::common::{Key, c_game_level, c_game_mode};
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
static mut MINER_STORE: Miner = MINER_ZERO;
static mut MINER_FRAME: usize = 0; // base row into MINER_SPRITE (0 or 8)
static mut MINER_SEQ_INDEX: u8 = 0;
static mut MINER_TIMER: Timer = Timer { rate: 0, acc: 0, remainder: 0, divisor: 0 };

// Shared globals (C ABI: robots.c / levels.rs / die.rs / rope.rs / cheat.rs).
// C: `MINER minerWilly = {.frame = 0, .dir = D_RIGHT}` — all-zero (D_RIGHT == 0).
#[unsafe(no_mangle)]
pub static mut minerWilly: Miner = MINER_ZERO;
#[unsafe(no_mangle)]
pub static mut minerWillyRope: i32 = 0;

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

#[unsafe(no_mangle)]
pub extern "C" fn Miner_Restore() {
    unsafe {
        minerWilly.x = MINER_STORE.x;
        minerWilly.y = MINER_STORE.y;
        minerWilly.tile = MINER_STORE.tile;
        minerWilly.align = MINER_STORE.align;
        minerWilly.frame = MINER_STORE.frame;
        minerWilly.dir = MINER_STORE.dir;
        minerWilly.move_ = MINER_STORE.move_;
        minerWilly.air = MINER_STORE.air;
        minerWilly.jump = MINER_STORE.jump;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_Save() {
    unsafe {
        MINER_STORE.x = minerWilly.x;
        MINER_STORE.y = minerWilly.y;
        MINER_STORE.tile = minerWilly.tile;
        MINER_STORE.align = minerWilly.align;
        MINER_STORE.frame = minerWilly.frame;
        MINER_STORE.dir = minerWilly.dir;
        MINER_STORE.move_ = minerWilly.move_;
        MINER_STORE.air = minerWilly.air;
        MINER_STORE.jump = minerWilly.jump;

        MINER_FRAME = if c_game_level == NIGHTMAREROOM { 8 } else { 0 };
    }
}

fn is_solid(tile: i32) -> bool {
    unsafe {
        if tile < 0 || tile == 512 {
            return false;
        }

        if Level_GetTileType(tile as usize) == TileType::Solid {
            return true;
        }

        if Level_GetTileType((tile + 32) as usize) == TileType::Solid {
            return true;
        }

        if tile + 64 > 511 {
            return false;
        }

        if Level_GetTileType((tile + 64) as usize) != TileType::Solid {
            return false;
        }

        if minerWilly.align == 6 {
            return true;
        }

        if minerWilly.air == 1 && minerWilly.jump > 9 {
            minerWilly.air = 0;
        }

        false
    }
}

fn move_left_right() {
    unsafe {
        let mut y = 0;
        let mut offset = 0;

        if minerWilly.move_ == 0 {
            return;
        }

        if minerWillyRope > 0 {
            return;
        }

        if minerWilly.dir == D_RIGHT {
            if minerWilly.frame < 3 {
                minerWilly.frame += 1;
                return;
            }

            if minerWilly.air == 0 {
                if Level_GetTileRamp((minerWilly.tile + 64) as usize) == TileType::RampL {
                    y = 8;
                    offset = 32;
                } else if Level_GetTileRamp((minerWilly.tile + 34) as usize) == TileType::RampR {
                    y = -8;
                    offset = -32;
                }
            }

            if minerWilly.x == 30 * 8 {
                Game_ChangeLevel(Direction::Right as i32);
                return;
            }

            if is_solid(minerWilly.tile + offset + 2) {
                return;
            }

            minerWilly.x += 8;
            minerWilly.tile += 1;
            minerWilly.frame = 0;
        } else if c_game_mode != GameMode::Running as i32 {
            if minerWilly.frame > 0 {
                minerWilly.frame -= 1;
                return;
            }

            if minerWilly.air == 0 {
                if Level_GetTileRamp((minerWilly.tile + 31) as usize) == TileType::RampL {
                    y = -8;
                    offset = -32;
                } else if Level_GetTileRamp((minerWilly.tile + 65) as usize) == TileType::RampR {
                    y = 8;
                    offset = 32;
                }
            }

            if minerWilly.x == 0 {
                Game_ChangeLevel(Direction::Left as i32);
                return;
            }

            if is_solid(minerWilly.tile + offset - 1) {
                return;
            }

            minerWilly.x -= 8;
            minerWilly.tile -= 1;
            minerWilly.frame = 3;
        }

        minerWilly.y += y;
        minerWilly.tile += offset;
    }
}

fn update_dir(convey_dir: i32) {
    unsafe {
        let mut dir = 0;

        if (System_IsKey(Key::Left as i32) != 0 || convey_dir == C_LEFT)
            && c_game_mode < GameMode::Running as i32
        {
            dir += 1;
        }

        if System_IsKey(Key::Right as i32) != 0
            || convey_dir == C_RIGHT
            || c_game_mode == GameMode::Running as i32
        {
            dir += 2;
        }

        if dir == 0 {
            minerWilly.move_ = 0;
        } else if dir == 1 {
            if minerWilly.dir == D_RIGHT {
                minerWilly.dir = D_LEFT;
                minerWilly.move_ = 0;
            } else {
                minerWilly.move_ = 1;
            }
        } else if dir == 2 {
            if minerWilly.dir == D_LEFT {
                minerWilly.dir = D_RIGHT;
                minerWilly.move_ = 0;
            } else {
                minerWilly.move_ = 1;
            }
        }

        if System_IsKey(Key::Jump as i32) != 0 && c_game_mode < GameMode::Running as i32 {
            minerWilly.air = 1;
            minerWilly.jump = 0;
            if minerWillyRope > 0 {
                minerWillyRope = -16;
                minerWilly.y &= 120;
                minerWilly.align = 4;
                minerWilly.move_ = 1;
            }
        }
    }
}

fn do_miner_ticker() {
    unsafe {
        let mut convey_dir = C_NONE;

        if minerWillyRope > 0 {
            update_dir(convey_dir);
            return;
        }

        if minerWilly.air == 1 {
            let y = minerWilly.y + JUMP_INFO[minerWilly.jump as usize].jump;

            if y < 0 {
                Game_ChangeLevel(Direction::Above as i32);
                return;
            }

            let tile = minerWilly.tile + JUMP_INFO[minerWilly.jump as usize].tile;
            if Level_GetTileType(tile as usize) == TileType::Solid
                || Level_GetTileType((tile + 1) as usize) == TileType::Solid
            {
                // we need to re-align Willy
                minerWilly.y = (y + 8) & 120;
                minerWilly.tile = tile + 32;
                minerWilly.align = 4;

                minerWilly.air = 2;
                minerWilly.move_ = 0;
                return;
            }

            audioPanX = minerWilly.x;
            Audio_WillySfx(
                JUMP_INFO[minerWilly.jump as usize].pitch,
                JUMP_INFO[minerWilly.jump as usize].length,
            );

            minerWilly.y = y;
            minerWilly.tile = tile;
            minerWilly.align = JUMP_INFO[minerWilly.jump as usize].align;
            minerWilly.jump += 1;

            if minerWilly.jump == 18 {
                minerWilly.air = 6;
                return;
            }

            if minerWilly.jump != 13 && minerWilly.jump != 16 {
                move_left_right();
                return;
            }
        }

        if minerWilly.align == 4 {
            let tile = minerWilly.tile + 64;
            if tile & 512 != 0 {
                Game_ChangeLevel(Direction::Below as i32);
                return;
            }

            let type0 = Level_GetTileType(tile as usize);
            let type1 = Level_GetTileType((tile + 1) as usize);
            if type0 == TileType::Harm || type1 == TileType::Harm {
                if minerWilly.air == 1
                    && (type0 as i32 <= TileType::Space as i32
                        || type1 as i32 <= TileType::Space as i32)
                {
                    move_left_right();
                } else {
                    Action = Some(Die_Action);
                }
                return;
            }

            if type0 as i32 > TileType::Space as i32 || type1 as i32 > TileType::Space as i32 {
                if minerWilly.air >= 12 {
                    Action = Some(Die_Action);
                    return;
                }

                minerWilly.air = 0;

                if type0 == TileType::ConveyL || type1 == TileType::ConveyL {
                    convey_dir = C_LEFT;
                } else if type0 == TileType::ConveyR || type1 == TileType::ConveyR {
                    convey_dir = C_RIGHT;
                }

                update_dir(convey_dir);
                move_left_right();
                return;
            }
        }

        if minerWilly.air == 1 {
            move_left_right();
            return;
        }

        minerWilly.move_ = 0;
        if minerWilly.air == 0 {
            minerWilly.air = 2;
            return;
        }

        minerWilly.air += 1;
        if minerWilly.air == 16 {
            // this affects the falling sound effect
            minerWilly.air = 12;
        }

        audioPanX = minerWilly.x;
        Audio_WillySfx(78 - minerWilly.air, 4);
        minerWilly.y += 4;
        minerWilly.align = 4;
        if minerWilly.y & 7 != 0 {
            minerWilly.align += 2;
        } else {
            minerWilly.tile += 32;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_Ticker() {
    do_miner_ticker();

    unsafe {
        if minerWilly.y < 0 {
            Game_ChangeLevel(Direction::Above as i32);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_Drawer() {
    unsafe {
        let mut offset = 0;
        let mut align = minerWilly.align;

        if minerWilly.air == 0 {
            if Level_GetTileRamp((minerWilly.tile + 64) as usize) == TileType::RampL {
                offset = minerWilly.frame << 1;
                align = yalign(offset);
            } else if Level_GetTileRamp((minerWilly.tile + 65) as usize) == TileType::RampR {
                offset = 6 - (minerWilly.frame << 1);
                align = yalign(offset);
            }
        }

        let row = MINER_FRAME + ((minerWilly.dir << 2) | minerWilly.frame) as usize;
        if Video_DrawMiner(
            ((minerWilly.y + offset) << 8) | minerWilly.x,
            MINER_SPRITE[row].as_ptr(),
            GAME_STATE.miner_attr_split.load(Ordering::Relaxed),
        ) != 0
        {
            Action = Some(Die_Action);
            return;
        }

        let mut tile = minerWilly.tile;
        let mut adj = 1;
        for _ in 0..align {
            if Level_GetTileType(tile as usize) == TileType::Harm {
                Action = Some(Die_Action);
                return;
            }
            tile += adj;
            adj ^= 30;
        }

        let mut tile = minerWilly.tile;
        let mut adj = 1;
        for _ in 0..align {
            if Level_GetTileType(tile as usize) == TileType::Item {
                Level_EraseItem(tile as usize);
                Game_GotItem();
            }
            tile += adj;
            adj ^= 30;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Miner_Init() {
    unsafe {
        minerWilly.x = 20 * 8;
        minerWilly.y = 13 * 8;
        minerWilly.tile = 13 * 32 + 20;
        minerWilly.align = 4;
        minerWilly.move_ = 0;
        minerWilly.air = 0;
    }

    Miner_Save();
}
