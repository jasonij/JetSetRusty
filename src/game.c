// game.c is now empty: all functions were ported to game.rs, and the shared
// game globals it used to define were relocated to src/cglobals.rs (as
// #[no_mangle] Rust statics). This translation unit will be deleted and dropped
// from build.rs — it is kept for one build only so the relocate lands as a
// self-contained, still-runnable step.
