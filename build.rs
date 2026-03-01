use std::process::Command;

fn main() {
    let sdl2 = pkg_config::probe_library("sdl2").unwrap();
    let sdl2_mixer = pkg_config::probe_library("SDL2_mixer").unwrap();
    let date = Command::new("date")
        .arg("+%y.%m.%d")
        .output()
        .expect("failed to run date!")
        .stdout;

    let date = String::from_utf8(date).unwrap().trim().to_string();
    let build_str = format!("1.0.0 {}", date);
    // Expose BUILD to Rust via env!("BUILD")
    println!("cargo:rustc-env=BUILD={}", build_str);
    // Pass it to the C compiler as a string literal
    let build_string = format!("\"{}\"", build_str);

    let mut build = cc::Build::new();

    for path in &sdl2.include_paths {
        build.include(path);
    }

    for path in &sdl2_mixer.include_paths {
        build.include(path);
    }

    build
        .define("BUILD", build_string.as_str())
        .file("src/codes.c")
        .file("src/game.c")
        .file("src/gameover.c")
        .file("src/levels.c")
        .file("src/game_main.c")
        .file("src/miner.c")
        .file("src/robots.c")
        .file("src/rope.c")
        .file("src/title.c")
        .compile("jetsetrusty");

    println!("cargo:rustc-link-lib=SDL2");
    println!("cargo:rustc-link-lib=SDL2_mixer");
}
