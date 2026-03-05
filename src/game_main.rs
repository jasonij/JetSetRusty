#![allow(non_snake_case, non_upper_case_globals)]

use crate::common::{Event, Key, HEIGHT, WIDTH};
use crate::misc::{videoColour, Timer, Timer_Set, Timer_Update, Video_Viewport};
use sdl2::sys as sdl;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

const SAMPLERATE: i32 = 22050;
const TICKRATE: i32 = 60;

// SDL key code table indexed by Key enum discriminant (Key::Left=0 .. Key::T=35)
const SDL_KEYS: [sdl::SDL_Keycode; 36] = [
    sdl::SDL_KeyCode::SDLK_LEFT as i32,
    sdl::SDL_KeyCode::SDLK_RIGHT as i32,
    sdl::SDL_KeyCode::SDLK_SPACE as i32,
    sdl::SDL_KeyCode::SDLK_RETURN as i32,
    sdl::SDL_KeyCode::SDLK_LSHIFT as i32,
    sdl::SDL_KeyCode::SDLK_RSHIFT as i32,
    sdl::SDL_KeyCode::SDLK_1 as i32,
    sdl::SDL_KeyCode::SDLK_2 as i32,
    sdl::SDL_KeyCode::SDLK_3 as i32,
    sdl::SDL_KeyCode::SDLK_4 as i32,
    sdl::SDL_KeyCode::SDLK_5 as i32,
    sdl::SDL_KeyCode::SDLK_6 as i32,
    sdl::SDL_KeyCode::SDLK_7 as i32,
    sdl::SDL_KeyCode::SDLK_8 as i32,
    sdl::SDL_KeyCode::SDLK_9 as i32,
    sdl::SDL_KeyCode::SDLK_0 as i32,
    sdl::SDL_KeyCode::SDLK_a as i32,
    sdl::SDL_KeyCode::SDLK_b as i32,
    sdl::SDL_KeyCode::SDLK_c as i32,
    sdl::SDL_KeyCode::SDLK_d as i32,
    sdl::SDL_KeyCode::SDLK_e as i32,
    sdl::SDL_KeyCode::SDLK_f as i32,
    sdl::SDL_KeyCode::SDLK_g as i32,
    sdl::SDL_KeyCode::SDLK_h as i32,
    sdl::SDL_KeyCode::SDLK_i as i32,
    sdl::SDL_KeyCode::SDLK_j as i32,
    sdl::SDL_KeyCode::SDLK_k as i32,
    sdl::SDL_KeyCode::SDLK_l as i32,
    sdl::SDL_KeyCode::SDLK_m as i32,
    sdl::SDL_KeyCode::SDLK_n as i32,
    sdl::SDL_KeyCode::SDLK_o as i32,
    sdl::SDL_KeyCode::SDLK_p as i32,
    sdl::SDL_KeyCode::SDLK_q as i32,
    sdl::SDL_KeyCode::SDLK_r as i32,
    sdl::SDL_KeyCode::SDLK_s as i32,
    sdl::SDL_KeyCode::SDLK_t as i32,
];

unsafe extern "C" {
    fn Loader_Action();
    fn Audio_Output(output: *mut i16);
    fn Audio_Init();
    fn rand() -> i32;
    fn srand(seed: u32);
    fn time(t: *mut i64) -> i64;
}

// ---- Exported globals -------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut Action: Event = Some(Loader_Action);
#[unsafe(no_mangle)]
pub static mut Responder: Event = None;
#[unsafe(no_mangle)]
pub static mut Ticker: Event = None;
#[unsafe(no_mangle)]
pub static mut Drawer: Event = None;

#[unsafe(no_mangle)]
pub static mut gameInput: i32 = Key::None as i32;
#[unsafe(no_mangle)]
pub static mut videoFlash: i32 = 0;

// ---- Module-private state ---------------------------------------------------

// SDL surface and key state are raw pointers needed by exported functions, but
// only ever accessed on the main thread — thread_local Cell avoids static mut.
thread_local! {
    static SDL_SURFACE: Cell<*mut sdl::SDL_Surface> = const { Cell::new(std::ptr::null_mut()) };
    static KEY_STATE:   Cell<*const u8>              = const { Cell::new(std::ptr::null()) };
}

static BORDER_INDEX: AtomicUsize = AtomicUsize::new(0);
static GAME_RUNNING: AtomicBool  = AtomicBool::new(true);

// ---- Exported C-facing functions --------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn DoNothing() {}

#[unsafe(no_mangle)]
pub extern "C" fn DoQuit() {
    GAME_RUNNING.store(false, Relaxed);
    // Setting these to None is equivalent to DoNothing — fire() is a no-op on None.
    unsafe {
        *(&raw mut Drawer) = None;
        *(&raw mut Ticker) = None;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn System_Rnd() -> i32 {
    unsafe { rand() }
}

#[unsafe(no_mangle)]
pub extern "C" fn System_IsKey(key: i32) -> i32 {
    let key_state = KEY_STATE.with(Cell::get);
    unsafe {
        let scancode = sdl::SDL_GetScancodeFromKey(SDL_KEYS[key as usize]);
        *key_state.add(scancode as usize) as i32
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn System_SetPixel(pos: i32, index: i32) {
    let surface_ptr = SDL_SURFACE.with(Cell::get);
    unsafe {
        let surface = &*surface_ptr;
        let pixels = surface.pixels as *mut u8;
        let bpp = (*surface.format).BytesPerPixel as i32;
        let offset = (pos / WIDTH) * surface.pitch + (pos & 255) * bpp;
        let pixel = pixels.add(offset as usize);
        let c = &videoColour[index as usize];
        *pixel        = c.b;
        *pixel.add(1) = c.g;
        *pixel.add(2) = c.r;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn System_Border(index: i32) {
    BORDER_INDEX.store(index as usize, Relaxed);
}

// ---- Private SDL helpers ----------------------------------------------------

// Must be `unsafe extern "C"` — that is the required signature for SDL_AudioCallback.
unsafe extern "C" fn sdl_audio_callback(
    _userdata: *mut core::ffi::c_void,
    stream: *mut u8,
    mut length: i32,
) {
    let mut output = stream as *mut i16;
    while length > 0 {
        unsafe { Audio_Output(output) };
        unsafe { output = output.add(2) };
        length -= 4;
    }
}

// Call an Event's function pointer if it is set.
fn fire(event: Event) {
    if let Some(f) = event {
        unsafe { f() };
    }
}

// Poll one SDL event, update gameInput, return false when the queue is empty.
fn system_get_event() -> bool {
    unsafe {
        *(&raw mut gameInput) = Key::None as i32;

        let mut event: sdl::SDL_Event = std::mem::zeroed();
        if sdl::SDL_PollEvent(&raw mut event) == 0 {
            return false;
        }

        let event_type = event.type_;

        if event_type == sdl::SDL_EventType::SDL_QUIT as u32 {
            DoQuit();
            return true;
        }
        if event_type != sdl::SDL_EventType::SDL_KEYDOWN as u32 {
            return true;
        }
        if event.key.repeat != 0 {
            return true;
        }

        let sym = event.key.keysym.sym;
        let k1 = sdl::SDL_KeyCode::SDLK_1 as i32;
        let ka = sdl::SDL_KeyCode::SDLK_a as i32;

        *(&raw mut gameInput) = if sym == sdl::SDL_KeyCode::SDLK_RETURN as i32 {
            Key::Enter as i32
        } else if sym == sdl::SDL_KeyCode::SDLK_ESCAPE as i32 {
            Key::Escape as i32
        } else if sym == sdl::SDL_KeyCode::SDLK_PAUSE as i32
            || sym == sdl::SDL_KeyCode::SDLK_TAB as i32
        {
            Key::Pause as i32
        } else if sym == sdl::SDL_KeyCode::SDLK_LALT as i32
            || sym == sdl::SDL_KeyCode::SDLK_RALT as i32
        {
            Key::Mute as i32
        } else if sym >= k1 && sym <= sdl::SDL_KeyCode::SDLK_4 as i32 {
            Key::K1 as i32 + (sym - k1)
        } else if matches!(sym - ka, 4 | 8 | 15 | 17 | 19 | 22 | 24) {
            // e i p r t w y
            Key::A as i32 + (sym - ka)
        } else {
            Key::Else as i32
        };

        true
    }
}

// ---- Entry point ------------------------------------------------------------

pub fn run() {
    // SDL init ----------------------------------------------------------------
    unsafe { sdl::SDL_Init(sdl::SDL_INIT_VIDEO | sdl::SDL_INIT_AUDIO) };
    unsafe { sdl::SDL_SetHint(c"SDL_VIDEO_MINIMIZE_ON_FOCUS_LOSS".as_ptr(), c"0".as_ptr()) };

    let mut mode = sdl::SDL_DisplayMode {
        format: 0,
        w: 0,
        h: 0,
        refresh_rate: 0,
        driverdata: std::ptr::null_mut(),
    };
    unsafe { sdl::SDL_GetDesktopDisplayMode(0, &raw mut mode) };

    let (mut vx, mut vy, mut vw, mut vh) = (0i32, 0i32, 0i32, 0i32);
    let multiply = Video_Viewport(mode.w, mode.h, &raw mut vx, &raw mut vy, &raw mut vw, &raw mut vh);
    let viewport = sdl::SDL_Rect { x: vx, y: vy, w: vw, h: vh };

    // Window and renderer -----------------------------------------------------
    let window = unsafe {
        sdl::SDL_CreateWindow(
            c"Jet-Set Willy".as_ptr(),
            sdl::SDL_WINDOWPOS_CENTERED_MASK as i32,
            sdl::SDL_WINDOWPOS_CENTERED_MASK as i32,
            0,
            0,
            sdl::SDL_WindowFlags::SDL_WINDOW_FULLSCREEN_DESKTOP as u32,
        )
    };

    let renderer = unsafe {
        sdl::SDL_CreateRenderer(
            window,
            -1,
            (sdl::SDL_RendererFlags::SDL_RENDERER_TARGETTEXTURE as u32)
                | (sdl::SDL_RendererFlags::SDL_RENDERER_ACCELERATED as u32)
                | (sdl::SDL_RendererFlags::SDL_RENDERER_PRESENTVSYNC as u32),
        )
    };

    unsafe { sdl::SDL_SetHint(c"SDL_RENDER_SCALE_QUALITY".as_ptr(), c"2".as_ptr()) };
    let target = unsafe {
        sdl::SDL_CreateTexture(
            renderer,
            sdl::SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as u32,
            sdl::SDL_TextureAccess::SDL_TEXTUREACCESS_TARGET as i32,
            WIDTH * multiply,
            HEIGHT * multiply,
        )
    };

    unsafe { sdl::SDL_SetHint(c"SDL_RENDER_SCALE_QUALITY".as_ptr(), c"0".as_ptr()) };
    let texture = unsafe {
        sdl::SDL_CreateTexture(
            renderer,
            sdl::SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as u32,
            sdl::SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32,
            WIDTH,
            HEIGHT,
        )
    };

    unsafe { sdl::SDL_ShowCursor(sdl::SDL_DISABLE as i32) };

    // Audio -------------------------------------------------------------------
    let want = sdl::SDL_AudioSpec {
        freq: SAMPLERATE,
        format: sdl::AUDIO_S16SYS as sdl::SDL_AudioFormat,
        channels: 2,
        silence: 0,
        samples: 256,
        padding: 0,
        size: 0,
        callback: Some(sdl_audio_callback),
        userdata: std::ptr::null_mut(),
    };
    let audio = unsafe {
        sdl::SDL_OpenAudioDevice(std::ptr::null(), 0, &raw const want, std::ptr::null_mut(), 0)
    };
    unsafe { sdl::SDL_PauseAudioDevice(audio, 0) };

    // Keyboard and RNG --------------------------------------------------------
    KEY_STATE.with(|c| {
        c.set(unsafe { sdl::SDL_GetKeyboardState(std::ptr::null_mut()) } as *const u8);
    });
    unsafe { srand(time(std::ptr::null_mut::<i64>()) as u32) };

    unsafe { Audio_Init() };

    // Timers ------------------------------------------------------------------
    let mut timer_frame = Timer { acc: 0, rate: 0, remainder: 0, divisor: 0 };
    let mut timer_flash = Timer { acc: 0, rate: 0, remainder: 0, divisor: 0 };
    Timer_Set(&raw mut timer_frame, TICKRATE, mode.refresh_rate);
    Timer_Set(&raw mut timer_flash, 25, TICKRATE * 8);

    // Main loop ---------------------------------------------------------------
    while GAME_RUNNING.load(Relaxed) {
        let frame = Timer_Update(&raw mut timer_frame);

        let mut sdl_surface: *mut sdl::SDL_Surface = std::ptr::null_mut();
        unsafe { sdl::SDL_LockTextureToSurface(texture, std::ptr::null(), &raw mut sdl_surface) };
        SDL_SURFACE.with(|c| c.set(sdl_surface));

        for _ in 0..frame {
            unsafe {
                fire(*(&raw const Action));
                while system_get_event() {
                    if *(&raw const gameInput) != Key::None as i32 {
                        fire(*(&raw const Responder));
                    }
                }
                fire(*(&raw const Ticker));
                fire(*(&raw const Drawer));
                *(&raw mut videoFlash) ^= Timer_Update(&raw mut timer_flash);
            }
        }

        unsafe { sdl::SDL_UnlockTexture(texture) };

        let border = &videoColour[BORDER_INDEX.load(Relaxed)];
        unsafe {
            sdl::SDL_SetRenderDrawColor(renderer, border.r, border.g, border.b, 0xff);
            sdl::SDL_RenderClear(renderer);
            sdl::SDL_SetRenderTarget(renderer, target);
            sdl::SDL_RenderCopy(renderer, texture, std::ptr::null(), std::ptr::null());
            sdl::SDL_SetRenderTarget(renderer, std::ptr::null_mut());
            sdl::SDL_RenderCopy(renderer, target, std::ptr::null(), &raw const viewport);
            sdl::SDL_RenderPresent(renderer);
        }
    }

    // Teardown ----------------------------------------------------------------
    unsafe {
        sdl::SDL_CloseAudioDevice(audio);
        sdl::SDL_DestroyTexture(texture);
        sdl::SDL_DestroyTexture(target);
        sdl::SDL_DestroyRenderer(renderer);
        sdl::SDL_DestroyWindow(window);
        sdl::SDL_Quit();
    }
}
