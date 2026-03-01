use crate::common::{Action, DoNothing, Drawer, Responder, Ticker, WIDTH};

// External functions
extern "C" {
    fn System_Border(index: i32);
    fn Video_PixelFill(pos: i32, size: i32);
    fn Video_DrawSprite(pos: i32, line: *const u16, paper: u8, ink: u8);
    fn Video_WriteLarge(x: i32, y: i32, text: *const i8);
    fn Video_PixelPaperFill(pos: i32, size: i32, ink: u8);
    fn Audio_Play(playing: i32);
    fn Audio_Sfx(sfx: i32);
    fn Title_Action();
}

const MUS_STOP: i32 = 0;
const SFX_GAMEOVER: i32 = 5;

static PLINTH_SPRITE: [u16; 16] = [
    14316, 30702, 0, 28662, 61431, 61431, 54619, 56251, 54619, 57339, 60791, 61175, 28022, 0,
    30702, 14316,
];

static BOOT_SPRITE: [u16; 16] = [
    4224, 4224, 4224, 4224, 4224, 4224, 4224, 8320, 8320, 18498, 34869, 33801, 32769, 32770,
    17293, 15478,
];

static MINER_SPRITE: [u16; 16] = [
    960, 960, 2016, 832, 992, 960, 384, 960, 2016, 2016, 3952, 4016, 960, 1888, 1760, 1904,
];

static mut BOOT_TICKS: i32 = 0;

static mut TEXT_GAME: [i8; 18] = [
    0x1, 0x0, 0x2, 0x0, b'G' as i8, b' ' as i8, 0x2, 0x0, b'a' as i8, b' ' as i8, 0x2, 0x0,
    b'm' as i8, b' ' as i8, 0x2, 0x0, b'e' as i8, 0,
];

static mut TEXT_OVER: [i8; 18] = [
    0x1, 0x0, 0x2, 0x0, b'O' as i8, b' ' as i8, 0x2, 0x0, b'v' as i8, b' ' as i8, 0x2, 0x0,
    b'e' as i8, b' ' as i8, 0x2, 0x0, b'r' as i8, 0,
];

unsafe extern "C" fn gameover_drawer() {
    if BOOT_TICKS <= 96 {
        Video_DrawSprite(
            (BOOT_TICKS & 126) * WIDTH + 15 * 8,
            BOOT_SPRITE.as_ptr(),
            0x0,
            0x7,
        );
        Video_PixelPaperFill(0, 128 * WIDTH, ((BOOT_TICKS & 12) >> 2) as u8);
    }

    if BOOT_TICKS < 96 {
        return;
    }

    Video_WriteLarge(7 * 8, 6 * 8, TEXT_GAME.as_ptr());
    Video_WriteLarge(18 * 8, 6 * 8, TEXT_OVER.as_ptr());
}

unsafe extern "C" fn gameover_ticker() {
    let mut c = BOOT_TICKS >> 2;

    TEXT_GAME[3] = (c & 0x7) as i8;
    c += 1;
    TEXT_GAME[7] = (c & 0x7) as i8;
    c += 1;
    TEXT_GAME[11] = (c & 0x7) as i8;
    c += 1;
    TEXT_GAME[15] = (c & 0x7) as i8;
    c += 1;
    TEXT_OVER[3] = (c & 0x7) as i8;
    c += 1;
    TEXT_OVER[7] = (c & 0x7) as i8;
    c += 1;
    TEXT_OVER[11] = (c & 0x7) as i8;
    c += 1;
    TEXT_OVER[15] = (c & 0x7) as i8;

    BOOT_TICKS += 1;

    if BOOT_TICKS < 256 {
        return;
    }

    Action = Some(Title_Action);
}

unsafe extern "C" fn gameover_init() {
    System_Border(0x0);
    Video_PixelFill(0, 128 * WIDTH);
    Video_DrawSprite(96 * WIDTH + 15 * 8, MINER_SPRITE.as_ptr(), 0x0, 0x7);
    Video_DrawSprite(112 * WIDTH + 15 * 8, PLINTH_SPRITE.as_ptr(), 0x0, 0x2);
    BOOT_TICKS = 0;

    Audio_Play(MUS_STOP);
    Audio_Sfx(SFX_GAMEOVER);

    Ticker = Some(gameover_ticker);
}

#[unsafe(no_mangle)]
pub extern "C" fn Gameover_Action() {
    unsafe {
        Responder = Some(DoNothing);
        Ticker = Some(gameover_init);
        Drawer = Some(gameover_drawer);
        Action = Some(DoNothing);
    }
}
