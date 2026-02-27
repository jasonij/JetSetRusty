mod cheat;
mod common;
mod misc;

unsafe extern "C" {
    fn game_main();
}

fn main() {
    unsafe {
        game_main();
    }
}
