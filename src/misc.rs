use crate::common::{HEIGHT, WIDTH};

// Colour table ----------------------------------------------------------------
#[repr(C)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub _padding: u8,
}

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static videoColour: [Colour; 16] = [
    Colour {
        r: 0x00,
        g: 0x00,
        b: 0x00,
        _padding: 0,
    }, // black
    Colour {
        r: 0x00,
        g: 0x00,
        b: 0xff,
        _padding: 0,
    }, // blue
    Colour {
        r: 0xff,
        g: 0x00,
        b: 0x00,
        _padding: 0,
    }, // red
    Colour {
        r: 0xff,
        g: 0x00,
        b: 0xff,
        _padding: 0,
    }, // magenta
    Colour {
        r: 0x00,
        g: 0xff,
        b: 0x00,
        _padding: 0,
    }, // green
    Colour {
        r: 0x00,
        g: 0xaa,
        b: 0xff,
        _padding: 0,
    }, // light blue
    Colour {
        r: 0xff,
        g: 0xff,
        b: 0x00,
        _padding: 0,
    }, // yellow
    Colour {
        r: 0xff,
        g: 0xff,
        b: 0xff,
        _padding: 0,
    }, // white
    Colour {
        r: 0xcc,
        g: 0xcc,
        b: 0xcc,
        _padding: 0,
    }, // mid grey (only for Willy or BT_SOLID)
    Colour {
        r: 0x00,
        g: 0x55,
        b: 0xff,
        _padding: 0,
    }, // mid blue
    Colour {
        r: 0xaa,
        g: 0x00,
        b: 0x00,
        _padding: 0,
    }, // mid red
    Colour {
        r: 0x55,
        g: 0x00,
        b: 0x00,
        _padding: 0,
    }, // dark red
    Colour {
        r: 0x00,
        g: 0xaa,
        b: 0x00,
        _padding: 0,
    }, // mid green
    Colour {
        r: 0x00,
        g: 0x55,
        b: 0x00,
        _padding: 0,
    }, // dark green
    Colour {
        r: 0xff,
        g: 0x80,
        b: 0x00,
        _padding: 0,
    }, // orange
    Colour {
        r: 0x80,
        g: 0x40,
        b: 0x00,
        _padding: 0,
    }, // brown
];

// Timer -----------------------------------------------------------------------
#[repr(C)]
pub struct Timer {
    pub acc: i32,
    pub rate: i32,
    pub remainder: i32,
    pub divisor: i32,
}

#[unsafe(no_mangle)]
pub extern "C" fn Timer_Set(timer: *mut Timer, numerator: i32, divisor: i32) {
    let t = unsafe { &mut *timer };
    t.acc = 0;
    t.rate = numerator / divisor;
    t.remainder = numerator - t.rate * divisor;
    t.divisor = divisor;
}

#[unsafe(no_mangle)]
pub extern "C" fn Timer_Update(timer: *mut Timer) -> i32 {
    let t = unsafe { &mut *timer };
    t.acc += t.remainder;
    if t.acc < t.divisor {
        return t.rate;
    }
    t.acc -= t.divisor;
    t.rate + 1
}

// External --------------------------------------------------------------------
#[unsafe(no_mangle)]
pub extern "C" fn Video_Viewport(
    width: i32,
    height: i32,
    x: *mut i32,
    y: *mut i32,
    w: *mut i32,
    h: *mut i32,
) -> i32 {
    if height * 4 / 3 <= width {
        // landscape
        unsafe {
            *h = (HEIGHT * height) as i32 / (HEIGHT + 16) as f32 as i32;
            *w = *h * 4 / 3;
        }
    } else {
        // portrait
        unsafe {
            *w = (WIDTH * width) as i32 / (WIDTH + 16) as f32 as i32;
            *h = *w * 3 / 4;
        }
    }

    unsafe {
        *x = (width - *w) / 2;
        *y = (height - *h) / 2;
    }

    let mut multiply = unsafe { *h / HEIGHT };
    if multiply < 1 {
        multiply = 1;
    }
    multiply
}
