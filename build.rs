use std::process::Command;

fn main() {
    // No C is compiled anymore — the whole game is Rust. We still link SDL2 and
    // SDL2_mixer, which the Rust code calls (game_main.rs owns the window and
    // audio setup; the SDL2_mixer symbols come in via raw FFI, since the sdl2
    // crate's `mixer` feature is off). probe_library emits the
    // cargo:rustc-link-{search,lib} directives as a side effect.
    pkg_config::probe_library("sdl2").unwrap();
    pkg_config::probe_library("SDL2_mixer").unwrap();

    // Datestamped version string, exposed to Rust as env!("BUILD") and shown on
    // the loader screen.
    let date = Command::new("date")
        .arg("+%y.%m.%d")
        .output()
        .expect("failed to run date!")
        .stdout;
    let date = String::from_utf8(date).unwrap().trim().to_string();
    println!("cargo:rustc-env=BUILD=1.0.0 {date}");
}
