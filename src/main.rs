mod audio;
mod cheat;
mod codes;
mod common;
mod die;
mod gameover;
mod levels;
mod loader;
mod misc;
mod rope;
mod title;
mod video;

unsafe extern "C" {
    fn game_main();
}

fn main() {
    unsafe {
        game_main();
    }
}
