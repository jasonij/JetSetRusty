#![allow(non_snake_case, dead_code, non_upper_case_globals)]

use crate::common::{HEIGHT, WIDTH};

unsafe extern "C" {
    pub fn System_SetPixel(point: i32, index: i32);
}

///////////////
// From video.h

const B_LEVEL: u8 = 1;
const B_ROBOT: u8 = 2;
const B_WILLY: u8 = 4;

#[inline]
pub const fn TILE2PIXEL(t: i32) -> i32 {
    ((t & 992) << 6) | ((t & 31) << 3)
}

#[inline]
pub const fn YALIGN(y: i32) -> i32 {
    4 | ((y & 4) >> 1) | (y & 2) | ((y & 1) << 1)
}

// We want to respect the original C types
//
// void Video_Write(int, char *);
// void Video_WriteLarge(int, int, char *);
// void Video_DrawSprite(int, u16 *, u8, u8);
// void Video_DrawRobot(int, u16 *, u8);
// int Video_DrawMiner(int, u16 *, int);
// void Video_DrawTile(int, u8 *, u8, u8);
// void Video_DrawArrow(int, int);
// void Video_DrawRopeSeg(int, u8);
// void Video_PixelFill(int, int);
// void Video_PixelInkFill(int, int, u8);
// void Video_PixelPaperFill(int, int, u8);
// int Video_CycleColours(void);
// int Video_TextWidth(char *);
// int Video_GetPixel(int);
//
///////////////

#[repr(C)]
#[derive(Clone, Copy)]
struct Pixel {
    ink: u8,
    point: u8,
}

static mut videoPixel: [Pixel; (WIDTH * HEIGHT) as usize] =
    [Pixel { ink: 0, point: 0 }; (WIDTH * HEIGHT) as usize];

static CHAR_SET: [[u8; 10]; 128] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // paper
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // ink
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [8, 128, 128, 192, 192, 224, 224, 240, 240, 0],
    [8, 248, 248, 252, 252, 254, 254, 255, 255, 0],
    [8, 255, 255, 254, 254, 252, 252, 248, 248, 0],
    [8, 240, 240, 224, 224, 192, 192, 128, 128, 0],
    [8, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [8, 255, 255, 255, 255, 255, 255, 255, 255, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [3, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [2, 47, 0, 0, 0, 0, 0, 0, 0, 0],
    [4, 3, 0, 3, 0, 0, 0, 0, 0, 0],
    [7, 18, 63, 18, 18, 63, 18, 0, 0, 0],
    [6, 46, 42, 127, 42, 58, 0, 0, 0, 0],
    [7, 35, 19, 8, 4, 50, 49, 0, 0, 0],
    [7, 16, 42, 37, 42, 16, 40, 0, 0, 0],
    [3, 2, 1, 0, 0, 0, 0, 0, 0, 0],
    [3, 30, 33, 0, 0, 0, 0, 0, 0, 0],
    [3, 33, 30, 0, 0, 0, 0, 0, 0, 0],
    [6, 8, 42, 28, 42, 8, 0, 0, 0, 0],
    [6, 8, 8, 62, 8, 8, 0, 0, 0, 0],
    [3, 64, 32, 0, 0, 0, 0, 0, 0, 0],
    [6, 8, 8, 8, 8, 8, 0, 0, 0, 0],
    [2, 32, 0, 0, 0, 0, 0, 0, 0, 0],
    [6, 32, 16, 8, 4, 2, 0, 0, 0, 0],
    [6, 12, 18, 33, 18, 12, 0, 0, 0, 0],
    [4, 34, 63, 32, 0, 0, 0, 0, 0, 0],
    [6, 50, 41, 41, 41, 38, 0, 0, 0, 0],
    [6, 18, 33, 37, 37, 26, 0, 0, 0, 0],
    [5, 15, 8, 60, 8, 0, 0, 0, 0, 0],
    [6, 23, 37, 37, 37, 25, 0, 0, 0, 0],
    [6, 30, 37, 37, 37, 24, 0, 0, 0, 0],
    [6, 1, 1, 49, 13, 3, 0, 0, 0, 0],
    [6, 26, 37, 37, 37, 26, 0, 0, 0, 0],
    [6, 6, 41, 41, 41, 30, 0, 0, 0, 0],
    [2, 20, 0, 0, 0, 0, 0, 0, 0, 0],
    [3, 32, 20, 0, 0, 0, 0, 0, 0, 0],
    [4, 8, 20, 34, 0, 0, 0, 0, 0, 0],
    [6, 20, 20, 20, 20, 20, 0, 0, 0, 0],
    [4, 34, 20, 8, 0, 0, 0, 0, 0, 0],
    [6, 2, 1, 41, 5, 2, 0, 0, 0, 0],
    [7, 30, 33, 45, 43, 45, 14, 0, 0, 0],
    [6, 48, 14, 9, 14, 48, 0, 0, 0, 0],
    [6, 63, 37, 37, 37, 26, 0, 0, 0, 0],
    [6, 30, 33, 33, 33, 18, 0, 0, 0, 0],
    [6, 63, 33, 33, 18, 12, 0, 0, 0, 0],
    [6, 63, 37, 37, 37, 33, 0, 0, 0, 0],
    [6, 63, 5, 5, 5, 1, 0, 0, 0, 0],
    [6, 30, 33, 33, 41, 26, 0, 0, 0, 0],
    [6, 63, 4, 4, 4, 63, 0, 0, 0, 0],
    [4, 33, 63, 33, 0, 0, 0, 0, 0, 0],
    [6, 16, 32, 32, 32, 31, 0, 0, 0, 0],
    [6, 63, 4, 10, 17, 32, 0, 0, 0, 0],
    [6, 63, 32, 32, 32, 32, 0, 0, 0, 0],
    [8, 56, 7, 12, 16, 12, 7, 56, 0, 0],
    [7, 63, 2, 4, 8, 16, 63, 0, 0, 0],
    [6, 30, 33, 33, 33, 30, 0, 0, 0, 0],
    [6, 63, 9, 9, 9, 6, 0, 0, 0, 0],
    [7, 30, 33, 41, 49, 33, 30, 0, 0, 0],
    [6, 63, 9, 9, 25, 38, 0, 0, 0, 0],
    [6, 18, 37, 37, 37, 24, 0, 0, 0, 0],
    [6, 1, 1, 63, 1, 1, 0, 0, 0, 0],
    [6, 31, 32, 32, 32, 31, 0, 0, 0, 0],
    [6, 7, 24, 32, 24, 7, 0, 0, 0, 0],
    [8, 7, 24, 32, 24, 32, 24, 7, 0, 0],
    [7, 33, 18, 12, 12, 18, 33, 0, 0, 0],
    [6, 3, 4, 56, 4, 3, 0, 0, 0, 0],
    [7, 33, 49, 41, 37, 35, 33, 0, 0, 0],
    [3, 63, 33, 0, 0, 0, 0, 0, 0, 0],
    [6, 2, 4, 8, 16, 32, 0, 0, 0, 0],
    [3, 33, 63, 0, 0, 0, 0, 0, 0, 0],
    [6, 4, 2, 63, 2, 4, 0, 0, 0, 0],
    [7, 64, 64, 64, 64, 64, 64, 0, 0, 0],
    [6, 36, 62, 37, 33, 34, 0, 0, 0, 0],
    [5, 16, 42, 42, 60, 0, 0, 0, 0, 0],
    [5, 63, 34, 34, 28, 0, 0, 0, 0, 0],
    [5, 28, 34, 34, 34, 0, 0, 0, 0, 0],
    [5, 28, 34, 34, 63, 0, 0, 0, 0, 0],
    [5, 28, 42, 42, 36, 0, 0, 0, 0, 0],
    [4, 62, 5, 1, 0, 0, 0, 0, 0, 0],
    [5, 28, 162, 162, 126, 0, 0, 0, 0, 0],
    [5, 63, 2, 2, 60, 0, 0, 0, 0, 0],
    [2, 61, 0, 0, 0, 0, 0, 0, 0, 0],
    [4, 32, 64, 61, 0, 0, 0, 0, 0, 0],
    [5, 63, 12, 18, 32, 0, 0, 0, 0, 0],
    [2, 63, 0, 0, 0, 0, 0, 0, 0, 0],
    [6, 62, 2, 60, 2, 60, 0, 0, 0, 0],
    [5, 62, 2, 2, 60, 0, 0, 0, 0, 0],
    [5, 28, 34, 34, 28, 0, 0, 0, 0, 0],
    [5, 254, 34, 34, 28, 0, 0, 0, 0, 0],
    [5, 28, 34, 34, 254, 128, 0, 0, 0, 0],
    [4, 60, 2, 2, 0, 0, 0, 0, 0, 0],
    [5, 36, 42, 42, 16, 0, 0, 0, 0, 0],
    [4, 2, 63, 2, 0, 0, 0, 0, 0, 0],
    [5, 30, 32, 32, 30, 0, 0, 0, 0, 0],
    [6, 6, 24, 32, 24, 6, 0, 0, 0, 0],
    [6, 30, 32, 28, 32, 30, 0, 0, 0, 0],
    [6, 34, 20, 8, 20, 34, 0, 0, 0, 0],
    [5, 30, 160, 160, 126, 0, 0, 0, 0, 0],
    [6, 34, 50, 42, 38, 34, 0, 0, 0, 0],
    [4, 4, 59, 33, 0, 0, 0, 0, 0, 0],
    [2, 63, 0, 0, 0, 0, 0, 0, 0, 0],
    [4, 33, 59, 4, 0, 0, 0, 0, 0, 0],
    [5, 16, 8, 16, 8, 0, 0, 0, 0, 0],
    [9, 60, 66, 153, 165, 165, 129, 66, 60, 0],
];

static CHAR_SET_LARGE: [[u16; 8]; 96] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 60, 7166, 7166, 60, 0, 0],
    [0, 7, 15, 0, 0, 15, 7, 0],
    [528, 8190, 8190, 528, 8190, 8190, 528, 0],
    [1592, 3196, 2116, 6214, 6214, 4044, 1928, 0],
    [6158, 7694, 1920, 480, 120, 7198, 7174, 0],
    [3968, 8156, 4222, 4578, 4030, 8156, 4160, 0],
    [0, 0, 8, 15, 7, 0, 0, 0],
    [0, 0, 2040, 4092, 6150, 4098, 0, 0],
    [0, 0, 4098, 6150, 4092, 2040, 0, 0],
    [128, 672, 992, 448, 992, 672, 128, 0],
    [0, 128, 128, 992, 992, 128, 128, 0],
    [0, 0, 8192, 14336, 6144, 0, 0, 0],
    [128, 128, 128, 128, 128, 128, 128, 0],
    [0, 0, 0, 6144, 6144, 0, 0, 0],
    [6144, 7680, 1920, 480, 120, 30, 6, 0],
    [2040, 4092, 6150, 4098, 6150, 4092, 2040, 0],
    [0, 4104, 4108, 8190, 8190, 4096, 4096, 0],
    [7684, 7942, 4482, 4290, 4194, 6206, 6172, 0],
    [2052, 6150, 4162, 4162, 4162, 8190, 4028, 0],
    [510, 510, 4352, 8160, 8160, 4352, 256, 0],
    [2174, 6270, 4162, 4162, 4162, 8130, 3970, 0],
    [4088, 8188, 4166, 4162, 4162, 8128, 3968, 0],
    [6, 6, 7682, 8066, 450, 126, 62, 0],
    [4028, 8190, 4162, 4162, 4162, 8190, 4028, 0],
    [60, 4222, 4162, 4162, 6210, 4094, 2044, 0],
    [0, 0, 0, 3096, 3096, 0, 0, 0],
    [0, 0, 4096, 7192, 3096, 0, 0, 0],
    [0, 192, 480, 816, 1560, 3084, 2052, 0],
    [576, 576, 576, 576, 576, 576, 576, 0],
    [0, 2052, 3084, 1560, 816, 480, 192, 0],
    [12, 14, 2, 7042, 7106, 126, 60, 0],
    [4088, 8188, 4100, 5060, 5060, 5116, 504, 0],
    [8176, 8184, 140, 134, 140, 8184, 8176, 0],
    [4098, 8190, 8190, 4162, 4162, 8190, 4028, 0],
    [2040, 4092, 6150, 4098, 4098, 6150, 3084, 0],
    [4098, 8190, 8190, 4098, 6150, 4092, 2040, 0],
    [4098, 8190, 8190, 4162, 4322, 6150, 7182, 0],
    [4098, 8190, 8190, 4162, 226, 6, 14, 0],
    [2040, 4092, 6150, 4226, 4226, 3974, 8076, 0],
    [8190, 8190, 64, 64, 64, 8190, 8190, 0],
    [0, 0, 4098, 8190, 8190, 4098, 0, 0],
    [3072, 7168, 4096, 4098, 8190, 4094, 2, 0],
    [4098, 8190, 8190, 192, 1008, 8126, 7182, 0],
    [4098, 8190, 8190, 4098, 4096, 6144, 7168, 0],
    [8190, 8190, 28, 120, 28, 8190, 8190, 0],
    [8190, 8190, 120, 480, 1920, 8190, 8190, 0],
    [4092, 8190, 4098, 4098, 4098, 8190, 4092, 0],
    [4098, 8190, 8190, 4162, 66, 126, 60, 0],
    [4092, 8190, 4098, 7170, 30722, 32766, 20476, 0],
    [4098, 8190, 8190, 66, 450, 8190, 7740, 0],
    [3100, 7230, 4194, 4162, 4290, 8078, 3852, 0],
    [14, 6, 4098, 8190, 8190, 4098, 6, 14],
    [4094, 8190, 4096, 4096, 4096, 8190, 4094, 0],
    [1022, 2046, 3072, 6144, 3072, 2046, 1022, 0],
    [2046, 8190, 7168, 2016, 7168, 8190, 2046, 0],
    [7182, 7998, 1008, 192, 1008, 7998, 7182, 0],
    [30, 62, 4192, 8128, 8128, 4192, 62, 30],
    [7694, 7942, 4482, 4290, 4194, 6206, 7198, 0],
    [0, 0, 8190, 8190, 4098, 4098, 0, 0],
    [6, 30, 120, 480, 1920, 7680, 6144, 0],
    [0, 0, 4098, 4098, 8190, 8190, 0, 0],
    [8, 12, 6, 3, 6, 12, 8, 0],
    [16384, 16384, 16384, 16384, 16384, 16384, 16384, 16384],
    [6176, 8190, 8191, 4129, 4099, 6150, 2048, 0],
    [3584, 7968, 4384, 4384, 4064, 8128, 4096, 0],
    [4098, 8190, 4094, 4128, 4192, 8128, 3968, 0],
    [4032, 8160, 4128, 4128, 4128, 6240, 2112, 0],
    [3968, 8128, 4192, 4130, 4094, 8190, 4096, 0],
    [4032, 8160, 4384, 4384, 4384, 6624, 2496, 0],
    [4128, 8188, 8190, 4130, 6, 12, 0, 0],
    [20416, 57312, 36896, 36896, 65472, 32736, 32, 0],
    [4098, 8190, 8190, 64, 32, 8160, 8128, 0],
    [0, 0, 4128, 8166, 8166, 4096, 0, 0],
    [0, 24576, 57344, 32768, 32800, 65510, 32742, 0],
    [4098, 8190, 8190, 768, 1920, 7392, 6240, 0],
    [0, 0, 4098, 8190, 8190, 4096, 0, 0],
    [8160, 8160, 96, 8128, 96, 8160, 8128, 0],
    [32, 8160, 8128, 32, 32, 8160, 8128, 0],
    [4032, 8160, 4128, 4128, 4128, 8160, 4032, 0],
    [32800, 65504, 65472, 36896, 4128, 8160, 4032, 0],
    [4032, 8160, 4128, 36896, 65472, 65504, 32800, 0],
    [4128, 8160, 8128, 4192, 32, 224, 192, 0],
    [2112, 6368, 4512, 4384, 4896, 7776, 3136, 0],
    [32, 32, 4092, 8190, 4128, 6176, 2048, 0],
    [4064, 8160, 4096, 4096, 4064, 8160, 4096, 0],
    [0, 2016, 4064, 6144, 6144, 4064, 2016, 0],
    [4064, 8160, 6144, 3840, 6144, 8160, 4064, 0],
    [6240, 7392, 1920, 768, 1920, 7392, 6240, 0],
    [4064, 40928, 36864, 36864, 53248, 32736, 16352, 0],
    [6240, 7264, 5664, 4896, 4512, 6368, 6240, 0],
    [0, 192, 192, 4092, 7998, 4098, 4098, 0],
    [0, 0, 0, 8190, 8190, 0, 0, 0],
    [0, 4098, 4098, 7998, 4092, 192, 192, 0],
    [4, 6, 2, 6, 4, 6, 2, 0],
    [2032, 3096, 6604, 4644, 4644, 6476, 3096, 2032],
];

static mut textInk: [u8; 2] = [0x0, 0x0];

// rope.c calls this
#[unsafe(no_mangle)]
pub extern "C" fn Video_GetPixel(pos: i32) -> i32 {
    unsafe { videoPixel[pos as usize] }.point as i32
}

fn Video_SetPixel(pos: i32, ink: u8) {
    unsafe {
        videoPixel[pos as usize].ink = ink;
        System_SetPixel(pos, ink as i32);
    }
}

// levels.c and loader.c
#[unsafe(no_mangle)]
pub extern "C" fn Video_TextWidth(text: *const i8) -> i32 {
    let mut l = 0i32;
    let mut p = text;
    while unsafe { *p } != 0 {
        l += CHAR_SET[(unsafe { *p } as u8) as usize][0] as i32;
        unsafe {
            p = p.add(1);
        }
    }
    l
}

// game.c and title.c
#[unsafe(no_mangle)]
pub extern "C" fn Video_CycleColours() -> i32 {
    for pos in 0..(WIDTH * HEIGHT) as usize {
        Video_SetPixel(
            pos as i32,
            ((unsafe { videoPixel[pos] }.ink + 3) & 0xf) as u8,
        );
    }
    unsafe { videoPixel[0] }.ink as i32
}

// gameover.c, die.c, game.c, loader.c
#[unsafe(no_mangle)]
pub extern "C" fn Video_PixelPaperFill(mut pos: i32, mut size: i32, ink: u8) {
    while size > 0 {
        if (Video_GetPixel(pos) & 1) == 0 {
            Video_SetPixel(pos, ink);
        }
        size -= 1;
        pos += 1;
    }
}

// game.c and die.c
#[unsafe(no_mangle)]
pub extern "C" fn Video_PixelInkFill(mut pos: i32, mut size: i32, ink: u8) {
    while size > 0 {
        if Video_GetPixel(pos) & 1 != 0 {
            Video_SetPixel(pos, ink);
        }
        size -= 1;
        pos += 1;
    }
}

// title.c, gameover.c, codes.c, levels.c
#[unsafe(no_mangle)]
pub extern "C" fn Video_PixelFill(mut pos: i32, mut size: i32) {
    while size > 0 {
        unsafe { videoPixel[pos as usize] }.point = 0;
        Video_SetPixel(pos, 0x0);
        size -= 1;
        pos += 1;
    }
}

// rope.c
#[unsafe(no_mangle)]
pub extern "C" fn Video_DrawRopeSeg(pos: i32, ink: u8) {
    unsafe { videoPixel[pos as usize] }.point = 1;
    Video_SetPixel(pos, ink);
}

// robots.c
#[unsafe(no_mangle)]
pub extern "C" fn Video_DrawArrow(mut pos: i32, dir: i32) {
    pos += dir;
    unsafe {
        videoPixel[pos as usize].point |= B_ROBOT | 1;
        Video_SetPixel(pos, 7);
        videoPixel[(pos + 6) as usize].point |= B_ROBOT | 1;
        Video_SetPixel(pos + 6, 7);
        pos += WIDTH;
        videoPixel[(pos + WIDTH) as usize].point |= B_ROBOT | 1;
        Video_SetPixel(pos + WIDTH, 7);
        videoPixel[(pos + WIDTH + 6) as usize].point |= B_ROBOT | 1;
        Video_SetPixel(pos + WIDTH + 6, 7);
        pos -= dir;
        for _ in 0..8 {
            videoPixel[pos as usize].point |= B_ROBOT | 1;
            Video_SetPixel(pos, 7);
            pos += 1;
        }
    }
}

// levels.c
#[unsafe(no_mangle)]
pub extern "C" fn Video_DrawTile(tile: i32, what: *const u8, paper: u8, ink: u8) {
    let mut pos = TILE2PIXEL(tile) + 7;
    let colour = [paper, ink];
    unsafe {
        for row in 0..8 {
            let mut pixel = pos;
            let mut byte = *what.add(row);
            for _ in 0..8 {
                videoPixel[pixel as usize].point = byte & B_LEVEL;
                Video_SetPixel(pixel, colour[(byte & 1) as usize]);
                pixel -= 1;
                byte >>= 1;
            }
            pos += WIDTH;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Video_DrawMiner(mut pos: i32, line: *const u16, level: i32) -> i32 {
    let attr = [0x8i32, 0x8, 0x8, 0x1];
    let mut die = 0i32;

    pos &= !7;
    let mut y = pos / WIDTH;
    pos += 15;

    unsafe {
        for row in 0..16 {
            let mut pixel = pos;
            let mut word = *line.add(row);
            let ink = attr[(y >> level) as usize];
            for _ in 0..16 {
                if word & 1 != 0 {
                    if Video_GetPixel(pixel) & B_ROBOT as i32 != 0 {
                        die = 1;
                    }
                    videoPixel[pixel as usize].point |= B_WILLY | 1;
                    Video_SetPixel(pixel, ink as u8);
                }
                pixel -= 1;
                word >>= 1;
            }
            pos += WIDTH;
            y += 1;
        }
    }

    die
}

#[unsafe(no_mangle)]
pub extern "C" fn Video_DrawRobot(mut pos: i32, line: *const u16, ink: u8) {
    pos += 15;
    unsafe {
        for row in 0..16 {
            let mut pixel = pos;
            let mut word = *line.add(row);
            for _ in 0..16 {
                if word & 1 != 0 {
                    videoPixel[pixel as usize].point |= B_ROBOT | 1;
                    Video_SetPixel(pixel, ink);
                }
                pixel -= 1;
                word >>= 1;
            }
            pos += WIDTH;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Video_DrawSprite(mut pos: i32, line: *const u16, paper: u8, ink: u8) {
    let colour = [paper, ink];
    pos += 15;
    unsafe {
        for row in 0..16 {
            let mut pixel = pos;
            let mut word = *line.add(row);
            for _ in 0..16 {
                videoPixel[pixel as usize].point = (word & 1) as u8;
                Video_SetPixel(pixel, colour[(word & 1) as usize]);
                pixel -= 1;
                word >>= 1;
            }
            pos += WIDTH;
        }
    }
}

fn TextCode(text: *const i8) -> i32 {
    unsafe {
        match *text as u8 {
            0x1 => {
                textInk[0] = *text.add(1) as u8;
                1
            }
            0x2 => {
                textInk[1] = *text.add(1) as u8;
                1
            }
            _ => 0,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Video_WriteLarge(mut x: i32, y: i32, text: *const i8) {
    let pos = y * WIDTH;
    unsafe {
        let mut p = text;
        while *p != 0 {
            if TextCode(p) != 0 {
                p = p.add(1);
                p = p.add(1);
                continue;
            }
            let ch = (*p as u8 - b' ') as usize;
            let byte = CHAR_SET_LARGE[ch].as_ptr();
            for col in 0..8 {
                if x >= 0 && x < WIDTH {
                    let mut pixel = pos + x;
                    let mut line = *byte.add(col);
                    for _ in 0..16 {
                        Video_SetPixel(pixel, textInk[(line & 1) as usize]);
                        pixel += WIDTH;
                        line >>= 1;
                    }
                }
                x += 1;
            }
            p = p.add(1);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Video_Write(mut pos: i32, text: *const i8) {
    unsafe {
        let mut p = text;
        while *p != 0 {
            if TextCode(p) != 0 {
                p = p.add(1);
                p = p.add(1);
                continue;
            }
            let ch = *p as u8 as usize;
            let row = &CHAR_SET[ch];
            let width = row[0] as usize;
            for col in 1..=width {
                let mut pixel = pos;
                let mut line = row[col];
                for _ in 0..8 {
                    videoPixel[pixel as usize].point = line & 1;
                    Video_SetPixel(pixel, textInk[(line & 1) as usize]);
                    pixel += WIDTH;
                    line >>= 1;
                }
                pos += 1;
            }
            p = p.add(1);
        }
    }
}
