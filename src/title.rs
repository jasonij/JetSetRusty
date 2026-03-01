use crate::*;

static TITLE_JSW: [i32; 100] = [
    100, 101, 102, 104, 105, 106, 108, 109, 110, 113, 114, 115, 117, 118, 119, 121, 122, 123,
    133, 136, 141, 145, 149, 154, 165, 168, 169, 170, 173, 177, 178, 179, 181, 182, 183, 186,
    197, 200, 205, 211, 213, 218, 228, 229, 232, 233, 234, 237, 241, 242, 243, 245, 246, 247,
    250, 326, 330, 332, 334, 338, 341, 345, 358, 362, 364, 366, 370, 373, 377, 390, 392, 394,
    396, 398, 402, 405, 406, 407, 408, 409, 422, 424, 426, 428, 430, 434, 439, 454, 455, 456,
    457, 458, 460, 462, 463, 464, 466, 467, 468, 471,
];

static mut TEXT_JSW: [u8; 6] = [b'\x01', b'\x02', b'\x02', b'\x0b', b'\x14', 0];

static TEXT_TICKER: &[u8] = b"      Press ENTER to Start                                JET-SET WILLY by Matthew Smith   1984 SOFTWARE PROJECTS Ltd                                Guide Willy to collect all the items around the house before Midnight so Maria will let you get to your bed                                Press ENTER to Start      \0";

static TEXT_END: i32 = (TEXT_TICKER.len() - 33) as i32;

static mut TEXT_POS: i32 = 0;
static mut TEXT_FRAME: i32 = 0;
static mut COLOUR_CYCLE: u8 = 0;

static COLOUR_CYCLE_ADJ: [u8; 6] = [1, 2, 3, 4, 5, 1];

unsafe fn game_start() {
    Video_PixelFill(128 * WIDTH, 64 * WIDTH);

    Game_GameReset();
    Game_DrawStatus();

    gameLevel = THEBATHROOM;
    itemCount = Level_ItemCount();
    Level_RestoreItems();

    Miner_Init();

    if cheatEnabled != 0 {
        Robots_DrawCheat();
    }

    gameMode = GM_NORMAL;
    gamePaused = 0;

    Game_Action();
}

unsafe fn do_title_ticker() {
    if audioMusicPlaying != 0 {
        if videoFlash != 0 {
            TEXT_JSW[1] = b'\x0b';
            TEXT_JSW[3] = b'\x02';
        } else {
            TEXT_JSW[1] = b'\x02';
            TEXT_JSW[3] = b'\x0b';
        }
        return;
    }

    COLOUR_CYCLE = COLOUR_CYCLE_ADJ[COLOUR_CYCLE as usize];

    if TEXT_POS < TEXT_END {
        if TEXT_FRAME < 6 {
            TEXT_FRAME += 2;
            return;
        }
        TEXT_POS += 1;
        TEXT_FRAME = 0;
        return;
    }

    Action = Some(Title_Action);
}

unsafe fn do_title_drawer() {
    if audioMusicPlaying != 0 {
        for i in 0..100 {
            let tile = TITLE_JSW[i];
            Video_Write(TILE2PIXEL(tile), TEXT_JSW.as_ptr() as *const i8);
        }
        return;
    }

    if COLOUR_CYCLE == 1 {
        System_Border(Video_CycleColours());
    }

    Video_WriteLarge(0, 0, b"\x01\x01\x02\x07\0".as_ptr() as *const i8);
    Video_WriteLarge(
        -(TEXT_FRAME & 6),
        19 * 8,
        TEXT_TICKER.as_ptr().add(TEXT_POS as usize) as *const i8,
    );
}

unsafe fn do_title_responder() {
    if gameInput == KEY_ENTER {
        Action = Some(game_start);
    } else if gameInput == KEY_ESCAPE {
        DoQuit();
    }
}

unsafe fn do_title_init() {
    System_Border(0x0);
    Video_PixelFill(0, WIDTH * HEIGHT);

    Video_Write(
        16 * WIDTH + 144,
        b"\x01\x00\x02\x05\x10\x11\x12\x13\0".as_ptr() as *const i8,
    );
    Video_Write(
        24 * WIDTH + 128,
        b"\x10\x14\x01\x05\x14\x14\x02\x09\x10\x14\0".as_ptr() as *const i8,
    );
    Video_Write(
        32 * WIDTH + 112,
        b"\x01\x00\x02\x05\x10\x11\x01\x05\x14\x14\x02\x09\x10\x11\x01\x09\x14\x14\0".as_ptr()
            as *const i8,
    );
    Video_Write(
        40 * WIDTH + 96,
        b"\x01\x00\x02\x05\x10\x14\x01\x05\x14\x14\x02\x09\x10\x14\x01\x09\x14\x14\x14\x14\0"
            .as_ptr() as *const i8,
    );
    Video_Write(
        48 * WIDTH + 80,
        b"\x01\x00\x02\x05\x10\x11\x01\x05\x14\x14\x02\x09\x10\x11\x01\x09\x14\x14\x02\x01\x10\x14\x14\x14\0"
            .as_ptr() as *const i8,
    );
    Video_Write(
        56 * WIDTH + 64,
        b"\x01\x00\x02\x05\x14\x14\x01\x05\x14\x14\x02\x09\x10\x14\x01\x09\x14\x14\x02\x00\x10\x14\x01\x01\x14\x14\x01\x09\x14\x14\0"
            .as_ptr() as *const i8,
    );
    Video_Write(
        64 * WIDTH + 64,
        b"\x01\x05\x02\x01\x12\x13\x14\x14\x01\x09\x02\x05\x12\x13\x02\x00\x10\x11\x01\x00\x14\x14\x01\x01\x14\x14\x01\x09\x14\x14\0"
            .as_ptr() as *const i8,
    );
    Video_Write(
        72 * WIDTH + 64,
        b"\x01\x01\x14\x14\x01\x05\x02\x01\x12\x13\x14\x14\x01\x00\x02\x05\x12\x13\x14\x14\x01\x01\x14\x14\x01\x09\x14\x14\0"
            .as_ptr() as *const i8,
    );
    Video_Write(
        80 * WIDTH + 64,
        b"\x01\x01\x02\x00\x12\x13\x14\x14\x01\x05\x02\x01\x14\x13\x14\x14\x01\x00\x02\x05\x12\x13\x01\x01\x14\x14\x01\x09\x14\x14\0"
            .as_ptr() as *const i8,
    );
    Video_Write(
        88 * WIDTH + 80,
        b"\x01\x01\x02\x00\x14\x13\x14\x14\x01\x05\x02\x01\x14\x13\x14\x14\x01\x01\x14\x14\x01\x09\x14\x14\0"
            .as_ptr() as *const i8,
    );
    Video_Write(
        96 * WIDTH + 96,
        b"\x01\x01\x02\x00\x14\x13\x14\x14\x01\x05\x02\x01\x12\x13\x01\x01\x14\x14\x01\x09\x14\x14\0"
            .as_ptr() as *const i8,
    );
    Video_Write(
        104 * WIDTH + 112,
        b"\x01\x01\x02\x00\x14\x13\x14\x14\x14\x14\x01\x09\x14\x14\0".as_ptr() as *const i8,
    );
    Video_Write(
        112 * WIDTH + 128,
        b"\x01\x01\x14\x13\x14\x14\x01\x09\x14\x14\0".as_ptr() as *const i8,
    );
    Video_Write(
        120 * WIDTH + 144,
        b"\x01\x01\x12\x13\x01\x09\x10\x11\0".as_ptr() as *const i8,
    );

    Video_WriteLarge(0, 0, b"\x01\x00\x02\x04\0".as_ptr() as *const i8);
    Video_WriteLarge(0, 19 * 8, TEXT_TICKER.as_ptr() as *const i8);

    TEXT_POS = 0;
    TEXT_FRAME = -1;
    COLOUR_CYCLE = 1;

    Audio_Music(MUS_TITLE, MUS_PLAY);

    Ticker = Some(do_title_ticker);
}

#[no_mangle]
pub extern "C" fn Title_Action() {
    unsafe {
        Responder = Some(do_title_responder);
        Ticker = Some(do_title_init);
        Drawer = Some(do_title_drawer);
        Action = Some(DoNothing);
    }
}
