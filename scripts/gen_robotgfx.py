#!/usr/bin/env python3
"""Transcribe robotGfx[45][8][16] from src/robots.c into the Rust ROBOT_GFX
static in src/robots.rs.

Strategy: strip C comments, brace-match the initializer, turn `{`/`}` into
`[`/`]` so it's a valid Python nested-list literal, ast.literal_eval it, then
pad to the full 45x8x16 (C leaves trailing frames/sets implicit; Rust can't).

Verification (run always): re-parse the emitted Rust and assert every frame the
C table actually provided survived byte-for-byte at the same [set][frame], and
that all padded positions are exactly zero and the dims are 45x8x16.

Run with no args = report + write emitted static to a .txt for inspection.
Run with --apply = additionally splice it into src/robots.rs.
"""
import ast
import re
import sys

SETS, FRAMES, ROW = 45, 8, 16
ROBOTS_C = "src/robots.c"
ROBOTS_RS = "src/robots.rs"


def strip_c_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.DOTALL)
    text = re.sub(r"//[^\n]*", "", text)
    return text


def extract_initializer(text: str, decl: str) -> str:
    """Return the balanced { ... } initializer following `decl` (then `=`)."""
    i = text.index(decl)
    eq = text.index("=", i)
    start = text.index("{", eq)
    depth = 0
    for j in range(start, len(text)):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return text[start : j + 1]
    raise ValueError("unbalanced braces")


def parse_gfx() -> list:
    src = strip_c_comments(open(ROBOTS_C).read())
    init = extract_initializer(src, "robotGfx[45][8][16]")
    py = init.replace("{", "[").replace("}", "]")
    return ast.literal_eval(py)  # nested lists: sets -> frames -> ints


def report(sets: list) -> None:
    print(f"parsed {len(sets)} sprite sets (type wants {SETS})")
    frame_counts = [len(s) for s in sets]
    from collections import Counter
    print(f"frames-per-set distribution: {dict(sorted(Counter(frame_counts).items()))}")
    bad_rows = [
        (si, fi, len(f))
        for si, s in enumerate(sets)
        for fi, f in enumerate(s)
        if len(f) != ROW
    ]
    print(f"rows whose length != {ROW}: {bad_rows if bad_rows else 'none'}")
    total_ints = sum(len(f) for s in sets for f in s)
    print(f"total integers provided by C: {total_ints}")
    lo = min(v for s in sets for f in s for v in f)
    hi = max(v for s in sets for f in s for v in f)
    print(f"value range: {lo}..{hi} (u16 max 65535)")
    assert len(sets) <= SETS, f"{len(sets)} sets > {SETS}"
    assert all(len(s) <= FRAMES for s in sets), "a set has > 8 frames"
    assert not bad_rows, "found rows not exactly 16 wide"
    assert 0 <= lo and hi <= 65535, "value out of u16 range"


def pad(sets: list) -> list:
    zero_row = [0] * ROW
    out = []
    for s in sets:
        frames = [list(f) for f in s] + [list(zero_row) for _ in range(FRAMES - len(s))]
        out.append(frames)
    for _ in range(SETS - len(out)):
        out.append([list(zero_row) for _ in range(FRAMES)])
    return out


def emit_rust(padded: list) -> str:
    lines = ["static ROBOT_GFX: [[[u16; 16]; 8]; 45] = ["]
    for si, s in enumerate(padded):
        lines.append("    [")
        for f in s:
            lines.append("        [" + ", ".join(str(v) for v in f) + "],")
        lines.append(f"    ], // set {si}")
    lines.append("];")
    return "\n".join(lines) + "\n"


def verify(rust_text: str, sets_c: list, padded: list) -> None:
    # Re-parse the emitted Rust independently of how we built it.
    body = rust_text[rust_text.index("=") + 1 : rust_text.rindex(";")].strip()
    body = re.sub(r"//[^\n]*", "", body)  # drop the `// set N` labels we emitted
    reparsed = ast.literal_eval(body)
    assert len(reparsed) == SETS, f"emitted {len(reparsed)} sets"
    assert all(len(s) == FRAMES for s in reparsed), "emitted set != 8 frames"
    assert all(len(f) == ROW for s in reparsed for f in s), "emitted row != 16"
    # Every frame C actually supplied must be identical in the emitted output.
    mism = 0
    for si, s in enumerate(sets_c):
        for fi, f in enumerate(s):
            if reparsed[si][fi] != list(f):
                mism += 1
    assert mism == 0, f"{mism} C frames corrupted in emission"
    # Padding must be all-zero.
    for si in range(SETS):
        supplied = len(sets_c[si]) if si < len(sets_c) else 0
        for fi in range(supplied, FRAMES):
            assert reparsed[si][fi] == [0] * ROW, f"pad set{si} frame{fi} nonzero"
    print(f"VERIFY OK: {sum(len(s) for s in sets_c)} C frames intact, "
          f"padding zero, dims {SETS}x{FRAMES}x{ROW}")


def main() -> None:
    apply = "--apply" in sys.argv
    sets_c = parse_gfx()
    report(sets_c)
    padded = pad(sets_c)
    rust = emit_rust(padded)
    verify(rust, sets_c, padded)

    out_txt = "/tmp/claude-1000/-home-jasonkroll-Code-JetSetRusty/0466a6c8-d59e-49c8-a078-ba57101ea36d/scratchpad/robot_gfx.rs.txt"
    open(out_txt, "w").write(rust)
    print(f"emitted {rust.count(chr(10))} lines -> {out_txt}")

    if apply:
        rs = open(ROBOTS_RS).read()
        stub = re.search(
            r"static ROBOT_GFX: \[\[\[u16; 16\]; 8\]; 45\] = \[\[\[0u16; 16\]; 8\]; 45\];",
            rs,
        )
        assert stub, "could not find ROBOT_GFX stub line in robots.rs"
        rs = rs[: stub.start()] + rust.rstrip("\n") + rs[stub.end() :]
        open(ROBOTS_RS, "w").write(rs)
        print(f"spliced ROBOT_GFX into {ROBOTS_RS}")


if __name__ == "__main__":
    main()
