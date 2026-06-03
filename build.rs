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

    // pants
    let c_sources = ["src/game.c", "src/miner.c", "src/robots.c"];
    for src in &c_sources {
        build.file(src);
    }
    build
        .define("BUILD", build_string.as_str())
        .compile("jetsetrusty");

    // Recompile the C side when any of its sources or headers change. Without
    // these, Cargo only re-runs build.rs on a coarse package scan and the .c
    // objects can go stale (e.g. de-static'ing a global doesn't take effect).
    for src in &c_sources {
        println!("cargo:rerun-if-changed={src}");
    }
    for entry in std::fs::read_dir("src").expect("read src/") {
        let path = entry.expect("dir entry").path();
        if path.extension().is_some_and(|e| e == "h") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    println!("cargo:rustc-link-lib=SDL2");
    println!("cargo:rustc-link-lib=SDL2_mixer");
}
