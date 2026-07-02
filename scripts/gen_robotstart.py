#!/usr/bin/env python3
"""Transcribe robotStart[][8] from src/robots.c into the Rust ROBOT_START
static in src/robots.rs.

Each C slot is a positional ROBOT initializer:
  {pos, min, max, DoMove, DoDraw, speed, gfx, ink, fUpdate, fIndex, fMask}
or the macro NOROBOT. This maps deterministically onto Fable's named `Robot {}`.

Because fields contain commas *inside* POS(...) / robotGfx[...], we can't use
literal_eval; we brace-match and split on depth-0 commas.

Verification (always): independently re-parse the emitted Rust and, per slot,
compare a canonical tuple against the C — positions are *evaluated numerically*
(POS(x,y) == pos(x,y) == y*WIDTH + x*8), fn names folded across C/Rust spelling,
gfx reduced to its index, ints normalized. Catches dropped slots, transposed
fields, and mistyped coordinates.

No args = report + write emitted static to a .txt. --apply also splices it in.
"""
import ast
import re
import sys

ROOMS, SLOTS, FIELDS = 60, 8, 11
WIDTH = 256
ROBOTS_C = "src/robots.c"
ROBOTS_RS = "src/robots.rs"
SCRATCH = "/tmp/claude-1000/-home-jasonkroll-Code-JetSetRusty/0466a6c8-d59e-49c8-a078-ba57101ea36d/scratchpad/robot_start.rs.txt"

MOVE_MAP = {
    "DoMoveLeft": "do_move_left", "DoMoveRight": "do_move_right",
    "DoMoveUp": "do_move_up", "DoMoveDown": "do_move_down",
    "DoMoveStatic": "do_move_static", "DoMoveArrowLeft": "do_move_arrow_left",
    "DoMoveArrowRight": "do_move_arrow_right", "DoMoveMaria": "do_move_maria",
    "DoNothing": "do_move_nothing",
}
DRAW_MAP = {
    "DoDrawRobot": "do_draw_robot", "DoDrawToilet": "do_draw_toilet",
    "DoDrawArrow": "do_draw_arrow", "DoNothing": "do_draw_nothing",
}


def strip_c_comments(t):
    t = re.sub(r"/\*.*?\*/", "", t, flags=re.DOTALL)
    return re.sub(r"//[^\n]*", "", t)


def extract_initializer(text, decl):
    i = text.index(decl)
    start = text.index("{", text.index("=", i))
    depth = 0
    for j in range(start, len(text)):
        depth += (text[j] == "{") - (text[j] == "}")
        if depth == 0:
            return text[start : j + 1]
    raise ValueError("unbalanced braces")


def split_top(s):
    """Split on depth-0 commas, respecting (), [], {} nesting."""
    items, depth, cur = [], 0, ""
    for ch in s:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            items.append(cur)
            cur = ""
        else:
            cur += ch
    items.append(cur)
    return [x.strip() for x in items if x.strip()]


def unwrap(s):
    s = s.strip()
    assert s[0] == "{" and s[-1] == "}", f"not brace-wrapped: {s[:40]!r}"
    return s[1:-1]


def parse_start():
    src = strip_c_comments(open(ROBOTS_C).read())
    init = extract_initializer(src, "robotStart[][8]")
    rooms = []
    for room_tok in split_top(unwrap(init)):
        slots = []
        for slot in split_top(unwrap(room_tok)):
            if slot == "NOROBOT":
                slots.append(None)
            else:
                slots.append(split_top(unwrap(slot)))
        rooms.append(slots)
    return rooms


# --- numeric evaluation of a pos/min/max expression, C or Rust spelling ------
def eval_pos(expr):
    e = re.sub(
        r"(?:POS|pos)\(\s*(\d+)\s*,\s*(\d+)\s*\)",
        lambda m: f"({int(m.group(2)) * WIDTH + int(m.group(1)) * 8})",
        expr,
    )
    assert re.fullmatch(r"[0-9+\-*()\s]+", e), f"unsafe pos expr: {expr!r}"
    return eval(e)


def fold_fn(tok):
    if "nothing" in tok.lower():
        return "nothing"
    return tok.lower().replace("_", "")


def gfx_index(tok):
    m = re.search(r"\[(\d+)\]", tok)
    return int(m.group(1)) if m else None


def canon(fields):
    """Canonical, spelling-independent tuple for one slot (C or Rust)."""
    p, mn, mx, mv, dr, sp, gf, ink, fu, fi, fm = fields
    return (
        eval_pos(p), eval_pos(mn), eval_pos(mx),
        fold_fn(mv), fold_fn(dr),
        int(sp, 0), gfx_index(gf),
        int(ink, 0), int(fu, 0), int(fi, 0), int(fm, 0),
    )


def emit_slot(f):
    p, mn, mx, mv, dr, sp, gf, ink, fu, fi, fm = f
    p, mn, mx = (re.sub(r"POS\(", "pos(", x) for x in (p, mn, mx))
    gi = gfx_index(gf)
    gfx = f"Some(&ROBOT_GFX[{gi}])" if gi is not None else "None"
    return (
        f"Robot {{ pos: {p}, min: {mn}, max: {mx}, "
        f"do_move: {MOVE_MAP[mv]}, do_draw: {DRAW_MAP[dr]}, speed: {sp}, "
        f"gfx: {gfx}, ink: {ink}, f_update: {fu}, f_index: {fi}, f_mask: {fm} }}"
    )


def emit_rust(rooms):
    out = ["static ROBOT_START: [[Robot; 8]; 60] = ["]
    for ri, room in enumerate(rooms):
        out.append("    [")
        for slot in room:
            out.append("        " + ("NOROBOT" if slot is None else emit_slot(slot)) + ",")
        out.append(f"    ], // room {ri}")
    out.append("];")
    return "\n".join(out) + "\n"


def reparse_rust_slots(rust_text):
    """Re-read the emitted ROBOT_START back into per-slot field lists."""
    eq = rust_text.index("=")
    body = rust_text[rust_text.index("[", eq) : rust_text.rindex("]") + 1]
    body = re.sub(r"//[^\n]*", "", body)  # drop the `// room N` labels
    rooms = []
    for room_tok in split_top(body[1:-1].strip()):
        # room_tok is `[ slot, slot, ... ]`
        inner = room_tok.strip()
        assert inner[0] == "[" and inner[-1] == "]"
        slots = []
        for slot in split_top(inner[1:-1]):
            if slot.strip() == "NOROBOT":
                slots.append(None)
            else:
                m = re.fullmatch(r"Robot\s*\{(.*)\}", slot.strip(), re.DOTALL)
                assert m, f"bad emitted slot: {slot[:50]!r}"
                fields = {}
                for kv in split_top(m.group(1)):
                    k, v = kv.split(":", 1)
                    fields[k.strip()] = v.strip()
                slots.append([
                    fields["pos"], fields["min"], fields["max"],
                    fields["do_move"], fields["do_draw"], fields["speed"],
                    fields["gfx"], fields["ink"], fields["f_update"],
                    fields["f_index"], fields["f_mask"],
                ])
        rooms.append(slots)
    return rooms


def report(rooms):
    print(f"parsed {len(rooms)} rooms (type wants {ROOMS})")
    slot_counts = {len(r) for r in rooms}
    print(f"slots-per-room set: {sorted(slot_counts)} (want just {{{SLOTS}}})")
    bad = [(ri, si, len(s)) for ri, r in enumerate(rooms)
           for si, s in enumerate(r) if s is not None and len(s) != FIELDS]
    print(f"non-NOROBOT slots whose field count != {FIELDS}: {bad if bad else 'none'}")
    n_robot = sum(1 for r in rooms for s in r if s is not None)
    n_empty = sum(1 for r in rooms for s in r if s is None)
    print(f"populated slots: {n_robot}, NOROBOT slots: {n_empty}")
    moves = sorted({s[3] for r in rooms for s in r if s})
    draws = sorted({s[4] for r in rooms for s in r if s})
    print(f"DoMove fns used: {moves}")
    print(f"DoDraw fns used: {draws}")
    unknown = ([m for m in moves if m not in MOVE_MAP]
               + [d for d in draws if d not in DRAW_MAP])
    assert not unknown, f"unmapped fn(s): {unknown}"
    assert len(rooms) == ROOMS, f"{len(rooms)} rooms != {ROOMS}"
    assert slot_counts == {SLOTS}, f"a room has != {SLOTS} slots"
    assert not bad, "a slot has wrong field count"
    gfx_max = max((s[6] and 0) or (gfx_index(s[6]) or 0)
                  for r in rooms for s in r if s)
    print(f"max gfx index referenced: {gfx_max} (ROBOT_GFX has 45)")
    assert gfx_max < 45, "gfx index out of range"


def verify(rust_text, rooms_c):
    rooms_r = reparse_rust_slots(rust_text)
    assert len(rooms_r) == ROOMS, f"emitted {len(rooms_r)} rooms"
    mism = 0
    for ri in range(ROOMS):
        assert len(rooms_r[ri]) == SLOTS, f"room {ri}: {len(rooms_r[ri])} slots"
        for si in range(SLOTS):
            c, r = rooms_c[ri][si], rooms_r[ri][si]
            if (c is None) != (r is None):
                mism += 1
            elif c is not None and canon(c) != canon(r):
                mism += 1
                if mism <= 5:
                    print(f"  MISMATCH room {ri} slot {si}:\n    C={canon(c)}\n    R={canon(r)}")
    assert mism == 0, f"{mism} slot mismatches C vs emitted"
    print(f"VERIFY OK: all {ROOMS}x{SLOTS} slots match (positions eval-equal, "
          f"fns/gfx/ints canonical)")


def main():
    rooms = parse_start()
    report(rooms)
    rust = emit_rust(rooms)
    verify(rust, rooms)
    open(SCRATCH, "w").write(rust)
    print(f"emitted {rust.count(chr(10))} lines -> {SCRATCH}")
    if "--apply" in sys.argv:
        rs = open(ROBOTS_RS).read()
        stub = re.search(
            r"static ROBOT_START: \[\[Robot; 8\]; 60\] = \[\[NOROBOT; 8\]; 60\];", rs
        )
        assert stub, "could not find ROBOT_START stub in robots.rs"
        rs = rs[: stub.start()] + rust.rstrip("\n") + rs[stub.end():]
        open(ROBOTS_RS, "w").write(rs)
        print(f"spliced ROBOT_START into {ROBOTS_RS}")


if __name__ == "__main__":
    main()
