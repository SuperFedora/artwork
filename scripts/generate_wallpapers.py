#!/usr/bin/env python3
# Copyright 2026 SuperFedora Contributors
# SPDX-License-Identifier: Apache-2.0
"""Generate 4K (3840x2160) 16-bit SuperFedora Dark and Light Hero wallpapers across all 8 palettes."""

import math
import shutil
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

DARK_BG = (7, 8, 11)
LIGHT_BG = (230, 232, 240)
LIGHT_OUTLINE = (26, 27, 38)

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
    ("01-super-blue", (122, 162, 247), (37, 84, 163)),
    ("02-metroid-violet", (187, 154, 247), (107, 56, 184)),
    ("03-super-ruby", (247, 118, 142), (185, 28, 60)),
    ("04-emerald-green", (158, 206, 106), (45, 122, 49)),
    ("05-star-gold", (224, 175, 104), (148, 98, 16)),
    ("06-electric-cyan", (125, 207, 255), (3, 105, 161)),
    ("07-flame-orange", (255, 158, 100), (184, 67, 9)),
    ("08-neon-coral", (244, 63, 94), (159, 18, 57)),
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


def blend(bg, fg, alpha):
    return (
        int(round(bg[0] * (1.0 - alpha) + fg[0] * alpha)),
        int(round(bg[1] * (1.0 - alpha) + fg[1] * alpha)),
        int(round(bg[2] * (1.0 - alpha) + fg[2] * alpha)),
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


def render_wallpaper(dark_rgb, light_rgb, is_light=False, sec=4.2):
    bg = LIGHT_BG if is_light else DARK_BG
    ar, ag, ab = dark_rgb
    lr, lg, lb = light_rgb
    hr = min(255, int(round(ar + (255 - ar) * 0.55)))
    hg = min(255, int(round(ag + (255 - ag) * 0.55)))
    hb = min(255, int(round(ab + (255 - ab) * 0.55)))

    if not is_light:
        color_word_spec = (255, 255, 255)
        color_word_hot = (hr, hg, hb)
        color_word_mid = (ar, ag, ab)
        color_word_base = blend(bg, (ar, ag, ab), 0.74)
        color_field_high = blend(bg, (ar, ag, ab), 0.16)
        color_field_mid = blend(bg, (ar, ag, ab), 0.11)
        color_field_low = blend(bg, (ar, ag, ab), 0.06)
    else:
        color_word_spec = (hr, hg, hb)
        color_word_hot = (ar, ag, ab)
        color_word_mid = blend((lr, lg, lb), (ar, ag, ab), 0.55)
        color_word_base = (lr, lg, lb)
        color_field_high = blend(bg, (lr, lg, lb), 0.15)
        color_field_mid = blend(bg, (lr, lg, lb), 0.10)
        color_field_low = blend(bg, (lr, lg, lb), 0.05)

    img = Image.new("RGB", (WIDTH, HEIGHT), bg)
    draw = ImageDraw.Draw(img)

    for r in range(GRID_ROWS):
        py = r * CELL_PX
        wm_row = r - ORIGIN_ROW
        for c in range(GRID_COLS):
            px = c * CELL_PX
            if px >= WIDTH:
                continue
            wm_col = c - ORIGIN_COL
            in_word_box = (0 <= wm_row < WM_H) and (0 <= wm_col < WM_W)
            is_word = in_word_box and WM_ROWS[wm_row][wm_col] == "1"
            is_outline = (
                not is_word
                and (-1 <= wm_row <= WM_H)
                and (-1 <= wm_col <= WM_W)
                and has_word_neighbor(wm_row, wm_col)
            )

            box = [px, py, px + CELL_PX - GAP_PX - 1, py + CELL_PX - GAP_PX - 1]
            nx = c * 0.065 + sec * 0.16
            ny = r * 0.065 - sec * 0.11
            n_val = (
                0.6 * sample_noise(nx, ny)
                + 0.4 * sample_noise(nx * 0.5 - sec * 0.08, ny * 0.5 + sec * 0.07)
            )

            if is_word:
                # 16-bit SNES 4-tier metallic bevel across SUPER (rows 1..6) and FEDORA (rows 8..22)
                if wm_row in (1, 8):
                    fill = color_word_spec if (wm_col + wm_row) % 3 != 0 else color_word_hot
                elif wm_row in (2, 9, 10):
                    fill = color_word_hot
                elif wm_row in (6, 21, 22):
                    fill = color_word_base
                else:
                    fill = color_word_hot if n_val > 0.68 else color_word_mid
                draw.rectangle(box, fill=fill)
                continue

            if is_outline:
                if is_light:
                    draw.rectangle(box, fill=LIGHT_OUTLINE)
                continue

            # Pure organic Bayer-dithered pixel field (no circles, balls, or rings)
            wave = 0.5 + 0.5 * math.sin(sec * 1.8 + ((r * 37 + c * 11) % 6.283))
            field_val = n_val * 0.78 + wave * 0.22
            dither = BAYER[(r & 7) * 8 + (c & 7)] / 64.0
            if field_val > dither * 0.82:
                fill = (
                    color_field_high if field_val > 0.66
                    else color_field_mid if field_val > 0.48
                    else color_field_low
                )
                draw.rectangle(box, fill=fill)
    return img


def main():
    repo_root = Path(__file__).resolve().parent.parent
    wall_root = repo_root / "wallpapers"
    for old in ("hero", "ambient"):
        if (wall_root / old).exists():
            shutil.rmtree(wall_root / old)

    dark_dir = wall_root / "dark"
    light_dir = wall_root / "light"
    dark_dir.mkdir(parents=True, exist_ok=True)
    light_dir.mkdir(parents=True, exist_ok=True)

    for idx, (slug, dark_rgb, light_rgb) in enumerate(PALETTES):
        sec = 3.5 + idx * 1.1
        dark_img = render_wallpaper(dark_rgb, light_rgb, False, sec=sec)
        dark_img.save(dark_dir / f"{slug}.png", optimize=True)

        light_img = render_wallpaper(dark_rgb, light_rgb, True, sec=sec)
        light_img.save(light_dir / f"{slug}.png", optimize=True)
        print(f"[OK] Generated {slug} (dark + light)")

    showcase_map = {
        "pixel-superfedora-blue.png": dark_dir / "01-super-blue.png",
        "pixel-crateria-synth.png": dark_dir / "02-metroid-violet.png",
        "pixel-retro-night.png": dark_dir / "06-electric-cyan.png",
        "pixel-dark-slate.png": light_dir / "01-super-blue.png",
    }
    for dst_name, src_path in showcase_map.items():
        (wall_root / dst_name).write_bytes(src_path.read_bytes())


if __name__ == "__main__":
    main()
