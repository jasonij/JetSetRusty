mod audio;
mod cheat;
mod common;
mod die;
mod gameover;
mod loader;
mod misc;
mod video;

unsafe extern "C" {
    fn game_main();
}

fn main() {
    unsafe {
        game_main();
    }
}
