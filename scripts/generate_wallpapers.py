#!/usr/bin/env python3
# Copyright 2026 SuperFedora Contributors
# SPDX-License-Identifier: Apache-2.0
"""Generate 4K (3840x2160) 16-bit SuperFedora Hero and Ambient wallpapers across all 8 palettes."""

import math
from pathlib import Path
from PIL import Image, ImageDraw

WIDTH, HEIGHT = 3840, 2160
CELL_PX = 36
GAP_PX = 2
WM_W, WM_H = 96, 24
GRID_COLS = WIDTH // CELL_PX + 1
GRID_ROWS = HEIGHT // CELL_PX
ORIGIN_COL = (GRID_COLS - WM_W) // 2
ORIGIN_ROW = (GRID_ROWS - WM_H) // 2
BG_COLOR = (7, 8, 11)

BAYER = [
    0, 32, 8, 40, 2, 34, 10, 42,
    48, 16, 56, 24, 50, 18, 58, 26,
    12, 44, 4, 36, 14, 46, 6, 38,
    60, 28, 52, 20, 62, 30, 54, 22,
    3, 35, 11, 43, 1, 33, 9, 41,
    51, 19, 59, 27, 49, 17, 57, 25,
    15, 47, 7, 39, 13, 45, 5, 37,
    63, 31, 55, 23, 61, 29, 53, 21,
]

WM_ROWS = [
    "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "000000111110110011011111001111110111110000000000000000000000000000000000000000000000000000000000",
    "000001100000110011011001101100000110011000000000000000000000000000000000000000000000000000000000",
    "000001111100110011011111101111100111111000000000000000000000000000000000000000000000000000000000",
    "000000000110110011011000001100000110110000000000000000000000000000000000000000000000000000000000",
    "000000000110110011011000001100000110011000000000000000000000000000000000000000000000000000000000",
    "000001111100011110011000001111110110011000000000000000000000000000000000000000000000000000000000",
    "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "000111111111111000111111111111000111111111000000000111111110000001111111110000000000011110000000",
    "000111111111111000111111111111000111111111110000001111111111000001111111111100000001111111100000",
    "000111111111111000111111111111000111111111111000011111111111100001111111111100000001111111100000",
    "000111000000000000111000000000000111000000111000011100000011100001110000011100000111110011111000",
    "000111000000000000111000000000000111000000111000011100000011100001110000011100000111000000111000",
    "000111000000000000111000000000000111000000111000011100000011100001111111111100000111000000111000",
    "000111111111000000111111111000000111000000111000011100000011100001111111110000000111000000111000",
    "000111111111000000111111111000000111000000111000011100000011100001111111110000000111111111111000",
    "000111111111000000111111111000000111000000111000011100000011100001110001110000000111111111111000",
    "000111000000000000111000000000000111000000111000011100000011100001110000011100000111111111111000",
    "000111000000000000111000000000000111000000111000011100000011100001110000011100000111000000111000",
    "000111000000000000111000000000000111000000111000011100000011100001110000011100000111000000111000",
    "000111000000000000111111111111000111111111111000011111111111100001110000000111000111000000111000",
    "000111000000000000111111111111000111111111110000001111111111000001110000000111000111000000111000",
    "000111000000000000111111111111000111111111000000000111111110000001110000000111000111000000111000",
    "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
]

PALETTES = [
    ("01-super-blue", (122, 162, 247)),
    ("02-metroid-violet", (187, 154, 247)),
    ("03-super-ruby", (247, 118, 142)),
    ("04-emerald-green", (158, 206, 106)),
    ("05-star-gold", (224, 175, 104)),
    ("06-electric-cyan", (125, 207, 255)),
    ("07-flame-orange", (255, 158, 100)),
    ("08-neon-coral", (244, 63, 94)),
]


def build_noise_table():
    n = 128
    seed = 10407530
    buf = [0.0] * (n * n)
    for i in range(len(buf)):
        seed = (seed * 1664525 + 1013904223) & 0xFFFFFFFF
        buf[i] = seed / 4294967296.0
    for _ in range(2):
        nxt = [0.0] * (n * n)
        for y in range(n):
            for x in range(n):
                s = 0.0
                for dy in (-1, 0, 1):
                    for dx in (-1, 0, 1):
                        s += buf[((y + dy) % n) * n + ((x + dx) % n)]
                nxt[y * n + x] = s / 9.0
        buf = nxt
    mn, mx = min(buf), max(buf)
    span = (mx - mn) or 1.0
    return [(v - mn) / span for v in buf]


NOISE = build_noise_table()


def sample_noise(x, y):
    n = 128
    ix, iy = int(math.floor(x)), int(math.floor(y))
    fx, fy = x - ix, y - iy
    x0, y0 = ix % n, iy % n
    x1, y1 = (x0 + 1) % n, (y0 + 1) % n
    sx = fx * fx * (3.0 - 2.0 * fx)
    sy = fy * fy * (3.0 - 2.0 * fy)
    top = NOISE[y0 * n + x0] * (1.0 - sx) + NOISE[y0 * n + x1] * sx
    bot = NOISE[y1 * n + x0] * (1.0 - sx) + NOISE[y1 * n + x1] * sx
    return top * (1.0 - sy) + bot * sy


def blend_on_bg(rgb, alpha):
    return (
        int(round(BG_COLOR[0] * (1.0 - alpha) + rgb[0] * alpha)),
        int(round(BG_COLOR[1] * (1.0 - alpha) + rgb[1] * alpha)),
        int(round(BG_COLOR[2] * (1.0 - alpha) + rgb[2] * alpha)),
    )


def has_word_neighbor(wm_row, wm_col):
    for dr in (-1, 0, 1):
        for dc in (-1, 0, 1):
            if dr == 0 and dc == 0:
                continue
            nr, nc = wm_row + dr, wm_col + dc
            if 0 <= nr < WM_H and 0 <= nc < WM_W and WM_ROWS[nr][nc] == "1":
                return True
    return False


def render_wallpaper(rgb, show_wordmark, sec=4.2):
    ar, ag, ab = rgb
    hr = min(255, int(round(ar + (255 - ar) * 0.55)))
    hg = min(255, int(round(ag + (255 - ag) * 0.55)))
    hb = min(255, int(round(ab + (255 - ab) * 0.55)))

    color_word_spec = (255, 255, 255)
    color_word_hot = (hr, hg, hb)
    color_word_mid = (ar, ag, ab)
    color_word_base = blend_on_bg((ar, ag, ab), 0.72)

    # Background dither palette kept recessed so wordmark & windows pop cleanly
    color_halo_high = blend_on_bg((hr, hg, hb), 0.34 if show_wordmark else 0.48)
    color_halo_mid = blend_on_bg((ar, ag, ab), 0.22 if show_wordmark else 0.30)
    color_field_high = blend_on_bg((ar, ag, ab), 0.14)
    color_field_low = blend_on_bg((ar, ag, ab), 0.06)

    img = Image.new("RGB", (WIDTH, HEIGHT), BG_COLOR)
    draw = ImageDraw.Draw(img)

    s1_x, s1_y, s1_r = WIDTH * 0.32, HEIGHT * 0.42, 1450.0
    s2_x, s2_y, s2_r = WIDTH * 0.68, HEIGHT * 0.54, 1350.0
    rip_x, rip_y, rip_r, ring_w = WIDTH * 0.32, HEIGHT * 0.42, 980.0, 90.0

    for r in range(GRID_ROWS):
        py = r * CELL_PX
        wm_row = r - ORIGIN_ROW
        for c in range(GRID_COLS):
            px = c * CELL_PX
            if px >= WIDTH:
                continue
            wm_col = c - ORIGIN_COL
            in_word_box = show_wordmark and (0 <= wm_row < WM_H) and (0 <= wm_col < WM_W)
            is_word = in_word_box and WM_ROWS[wm_row][wm_col] == "1"
            is_outline = show_wordmark and not is_word and (-1 <= wm_row <= WM_H) and (-1 <= wm_col <= WM_W) and has_word_neighbor(wm_row, wm_col)

            cx = px + CELL_PX * 0.5
            cy = py + CELL_PX * 0.5

            d1 = math.hypot(cx - s1_x, cy - s1_y)
            sm1 = ((1.0 - d1 / s1_r) ** 2) if d1 < s1_r else 0.0
            d2 = math.hypot(cx - s2_x, cy - s2_y)
            sm2 = (((1.0 - d2 / s2_r) ** 2) * 0.78) if d2 < s2_r else 0.0
            sm = max(sm1, sm2)

            dr = math.hypot(cx - rip_x, cy - rip_y)
            rip_glow = max(0.0, (1.0 - abs(dr - rip_r) / ring_w) * 0.45) if abs(dr - rip_r) <= ring_w else 0.0

            box = [px, py, px + CELL_PX - GAP_PX - 1, py + CELL_PX - GAP_PX - 1]
            if is_word:
                lit = 0.30 + sm * 2.10 + rip_glow * 1.35
                # Subtle top-to-bottom 16-bit SNES bevel gradient across each glyph row
                bevel = 0.18 if (wm_row in (1, 2, 8, 9)) else 0.0
                score = lit + bevel
                fill = (
                    color_word_spec if score > 1.42
                    else color_word_hot if score > 0.85
                    else color_word_mid if score > 0.45
                    else color_word_base
                )
                draw.rectangle(box, fill=fill)
                continue

            if is_outline:
                # Crisp 1-cell obsidian trench around SUPER FEDORA glyphs for unmistakable contrast
                continue

            nx = c * 0.07 + sec * 0.16
            ny = r * 0.07 - sec * 0.11
            n_val = 0.6 * sample_noise(nx, ny) + 0.4 * sample_noise(nx * 0.5 - sec * 0.08, ny * 0.5 + sec * 0.07)
            energy = n_val * 0.75 + (0.5 + 0.5 * math.sin(sec * 1.8 + ((r * 37 + c * 11) % 6.283))) * 0.25
            field_val = energy * 0.38 + sm * 0.82 + rip_glow * 0.75
            dither = BAYER[(r & 7) * 8 + (c & 7)] / 64.0
            if field_val > dither * 0.76:
                fill = (
                    color_halo_high if (sm > 0.42 or rip_glow > 0.38)
                    else color_halo_mid if (sm > 0.15 or rip_glow > 0.15)
                    else color_field_high if field_val > 0.52
                    else color_field_low
                )
                draw.rectangle(box, fill=fill)
    return img


def main():
    repo_root = Path(__file__).resolve().parent.parent
    hero_dir = repo_root / "wallpapers" / "hero"
    ambient_dir = repo_root / "wallpapers" / "ambient"
    hero_dir.mkdir(parents=True, exist_ok=True)
    ambient_dir.mkdir(parents=True, exist_ok=True)

    for idx, (slug, rgb) in enumerate(PALETTES):
        sec = 3.5 + idx * 1.1
        hero_img = render_wallpaper(rgb, True, sec=sec)
        hero_img.save(hero_dir / f"{slug}.png", optimize=True)

        amb_img = render_wallpaper(rgb, False, sec=sec)
        amb_img.save(ambient_dir / f"{slug}.png", optimize=True)
        print(f"[OK] Generated {slug} (hero + ambient)")

    showcase_map = {
        "pixel-superfedora-blue.png": hero_dir / "01-super-blue.png",
        "pixel-crateria-synth.png": hero_dir / "02-metroid-violet.png",
        "pixel-retro-night.png": hero_dir / "06-electric-cyan.png",
        "pixel-dark-slate.png": ambient_dir / "01-super-blue.png",
    }
    for dst_name, src_path in showcase_map.items():
        dst_path = repo_root / "wallpapers" / dst_name
        dst_path.write_bytes(src_path.read_bytes())


if __name__ == "__main__":
    main()
