#!/usr/bin/env python3
"""
Convert levelData array from levels.c to Rust.
Usage: python3 convert_levels.py levels.c > level_data.rs
"""

import sys
import re

def parse_levels_c(text):
    # Extract just the levelData block
    start = text.index('static LEVEL    levelData[60] =')
    # Find the closing "};" of the array
    end = text.index('\n};\n', start) + 4
    return text[start:end]

def parse_info_entries(info_block):
    """Parse .info = { {attr, type}, ... } entries"""
    entries = re.findall(r'\{(0x[0-9a-fA-F]+),\s*(T_\w+)\}', info_block)
    return entries

def parse_gfx_entries(gfx_block):
    """Parse .gfx = { {bytes...}, ... } entries, handling SPACE macro"""
    # Replace SPACE macro (keep braces so the regex can find it)
    gfx_block = gfx_block.replace('SPACE', '{0, 0, 0, 0, 0, 0, 0, 0}')
    rows = re.findall(r'\{([^}]+)\}', gfx_block)
    result = []
    for row in rows:
        nums = [n.strip() for n in row.split(',') if n.strip()]
        # Pad to 8 bytes if needed
        while len(nums) < 8:
            nums.append('0')
        result.append(nums[:8])
    return result

def parse_data_array(data_block):
    """Parse the 512-element data array"""
    # Extract all numbers
    nums = re.findall(r'\d+', data_block)
    return nums[:512]

def parse_item_array(item_block):
    """Parse item array"""
    nums = re.findall(r'\d+', item_block)
    return nums

def parse_map_array(map_block):
    nums = re.findall(r'\d+', map_block)
    return nums[:4]

def parse_title(title_block):
    m = re.search(r'"([^"]*)"', title_block)
    return m.group(1) if m else ""

def parse_single_level(level_text):
    """Parse one { .title=..., .map=..., .data=..., .gfx=..., .info=..., .itemCount=..., .item=... } block"""
    result = {}

    # title
    m = re.search(r'\.title\s*=\s*"([^"]*)"', level_text)
    result['title'] = m.group(1) if m else ""

    # map
    m = re.search(r'\.map\s*=\s*\{([^}]*)\}', level_text)
    result['map'] = re.findall(r'\d+', m.group(1)) if m else ['0','0','0','0']

    # data - multi-line, need to find the full block
    m = re.search(r'\.data\s*=\s*\{(.*?)\}(?=\s*,\s*\.gfx)', level_text, re.DOTALL)
    if m:
        nums = re.findall(r'\d+', m.group(1))
        result['data'] = nums[:512]
    else:
        result['data'] = ['0'] * 512

    # gfx
    m = re.search(r'\.gfx\s*=\s*\{(.*?)\}(?=\s*,\s*\.info)', level_text, re.DOTALL)
    if m:
        result['gfx'] = parse_gfx_entries(m.group(1))
    else:
        result['gfx'] = []

    # info
    m = re.search(r'\.info\s*=\s*\{(.*?)\}(?=\s*,\s*\.itemCount)', level_text, re.DOTALL)
    if m:
        result['info'] = parse_info_entries(m.group(1))
    else:
        result['info'] = []

    # itemCount
    m = re.search(r'\.itemCount\s*=\s*(\d+)', level_text)
    result['itemCount'] = int(m.group(1)) if m else 0

    # item
    m = re.search(r'\.item\s*=\s*\{([^}]*)\}', level_text)
    result['item'] = re.findall(r'\d+', m.group(1)) if m else []

    return result

def split_levels(block):
    """Split the levelData array into individual level text blocks."""
    # Strip the outer "static LEVEL levelData[60] = {" ... "};"
    # Find the first '{' after the '='
    start = block.index('{') + 1
    end = block.rindex('}')
    inner = block[start:end]

    levels = []
    depth = 0
    current_start = None

    for i, ch in enumerate(inner):
        if ch == '{':
            if depth == 0:
                current_start = i
            depth += 1
        elif ch == '}':
            depth -= 1
            if depth == 0 and current_start is not None:
                levels.append(inner[current_start+1:i])
                current_start = None

    return levels

def format_gfx(gfx_rows):
    """Format gfx as a Rust [[u8;8];10], padding to 10 rows"""
    while len(gfx_rows) < 10:
        gfx_rows.append(['0','0','0','0','0','0','0','0'])
    lines = []
    for row in gfx_rows[:10]:
        line = '        [' + ', '.join(row) + '],'
        lines.append(line)
    return '[\n' + '\n'.join(lines) + '\n    ]'

def format_info(info_entries):
    """Format info as a Rust [LevelInfo;10], padding to 10"""
    while len(info_entries) < 10:
        info_entries.append(('0x00', 'T_SPACE'))
    lines = []
    for attr, tile_type in info_entries[:10]:
        lines.append(f'        LevelInfo {{ attr: {attr}, tile_type: {tile_type} }},')
    return '[\n' + '\n'.join(lines) + '\n    ]'

def format_data(data):
    """Format as a Rust [i32;512]"""
    while len(data) < 512:
        data.append('0')
    chunks = [data[i:i+32] for i in range(0, 512, 32)]
    lines = []
    for chunk in chunks:
        lines.append('        ' + ', '.join(chunk) + ',')
    return '[\n' + '\n'.join(lines) + '\n    ]'

def format_item(item, item_count):
    """Format as [i32;12]"""
    items = list(item)
    while len(items) < 12:
        items.append('0')
    return '[' + ', '.join(items[:12]) + ']'

def level_to_rust(level, idx):
    title = level['title'].replace('\\', '\\\\').replace('"', '\\"')
    map_arr = ', '.join(level['map'][:4])
    data_str = format_data(level['data'])
    gfx_str = format_gfx(level['gfx'])
    info_str = format_info(level['info'])
    item_count = level['itemCount']
    item_str = format_item(level['item'], item_count)

    return f'''    // Level {idx}: {title}
    LevelData {{
        title: "{title}",
        map: [{map_arr}],
        data: {data_str},
        gfx: {gfx_str},
        info: {info_str},
        item_count: {item_count},
        item: {item_str},
    }},'''

def generate_rust(levels):
    header = '''\
// Auto-generated from levels.c by convert_levels.py
// Do not edit by hand.

// NOTE: T_* constants and LevelInfo must be defined in game.rs / common.rs
// Check game.h for the actual numeric values of T_SPACE, T_ITEM, etc.
// and replace the use of T_* here with the appropriate Rust enum/const.

use crate::game::TileType; // TODO: verify this import path

#[derive(Clone)]
pub struct LevelInfo {
    pub attr: u16,
    pub tile_type: TileType,
}

#[derive(Clone)]
pub struct LevelData {
    pub title: &\'static str,
    pub map: [i32; 4],
    pub data: [i32; 512],
    pub gfx: [[u8; 8]; 10],
    pub info: [LevelInfo; 10],
    pub item_count: i32,
    pub item: [i32; 12],
}

pub static LEVEL_DATA: [LevelData; 60] = [
'''
    footer = '];\n'

    entries = []
    for i, level in enumerate(levels):
        entries.append(level_to_rust(level, i))

    return header + '\n'.join(entries) + '\n' + footer

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 convert_levels.py levels.c", file=sys.stderr)
        sys.exit(1)

    with open(sys.argv[1], 'r') as f:
        text = f.read()

    block = parse_levels_c(text)
    level_texts = split_levels(block)
    print(f"Found {len(level_texts)} levels", file=sys.stderr)

    levels = [parse_single_level(lt) for lt in level_texts]
    rust = generate_rust(levels)
    print(rust)

if __name__ == '__main__':
    main()
