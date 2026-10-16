// Copyright 2026 SuperFedora Contributors
// SPDX-License-Identifier: Apache-2.0

mod engine;
mod palettes;
mod sfx;
mod tui;

use crate::engine::{
    apply_all, apply_wallpaper_file, fuzzel_pick, list_wallpapers, print_status_and_swatches,
    rotate_wallpaper, set_rgb,
};
use crate::palettes::{PALETTES, ThemeState, resolve_color, resolve_mode};
use crate::sfx::handle_sfx_cli;
use crate::tui::run_tui;
use std::env;
use std::io::{self, IsTerminal};
use std::path::Path;
use std::process;

fn print_help() {
    println!("SuperTheme v{} — Native Rust 16-Bit Theme, Wallpaper & SFX Engine", env!("CARGO_PKG_VERSION"));
    println!("Usage:");
    println!("  supertheme                                  Launch TUI (or print status if non-TTY)");
    println!("  supertheme tui                              Launch interactive 16-Bit Theme & SFX Studio");
    println!("  supertheme status | list | swatches         Show active theme & 16-bit rainbow swatches");
    println!("  supertheme <color> [--mode night|day|toggle] [--corners square|rounded|toggle]");
    println!("  supertheme set <color> [night|day] [square|rounded]");
    println!("  supertheme next | prev                      Rotate through 16 rainbow palettes");
    println!("  supertheme toggle | mode [night|day|toggle] Flip between night (dark) & day (light)");
    println!("  supertheme corners [square|rounded|toggle]  Swap between square & rounded corners");
    println!("  supertheme nightlight [on|off|toggle]       Toggle warm eye-care nightlight");
    println!("  supertheme wallpaper [next|prev|list|set <path>] Browse or rotate wallpapers");
    println!("  supertheme rgb [hex|off]                    Sync OpenRGB hardware lighting");
    println!("  supertheme profile [stealth|balanced|forge|arcade] Sync RGB & animation mode hooks");
    println!("  supertheme sfx [status|on|off|toggle|init|play <chime>] Control 16-bit WAV SFX");
    println!("  supertheme pick                             Interactive Fuzzel 16-bit theme selector");
    println!("  supertheme reload                           Re-apply saved theme from state file");
}

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    if let Some(first) = raw_args.first().map(|s| s.as_str()) {
        match first {
            "-h" | "--help" | "help" => {
                print_help();
                return;
            }
            "-V" | "--version" | "version" => {
                println!("supertheme {} (SuperFedora/supertheme — Rust 1.99 Native Engine)", env!("CARGO_PKG_VERSION"));
                return;
            }
            "sfx" | "sound" => {
                process::exit(handle_sfx_cli(&raw_args[1..]));
            }
            "tui" | "studio" => {
                process::exit(run_tui());
            }
            _ => {}
        }
    }

    let mut st = ThemeState::load();
    if raw_args.is_empty() {
        if io::stdin().is_terminal() && io::stdout().is_terminal() {
            process::exit(run_tui());
        } else {
            print_status_and_swatches(&st, false);
            return;
        }
    }

    let mut pos = Vec::new();
    let mut opt_mode: Option<String> = None;
    let mut opt_corners: Option<String> = None;
    let mut opt_nl: Option<String> = None;
    let mut i = 0;
    while i < raw_args.len() {
        let a = &raw_args[i];
        if a == "--mode" {
            i += 1;
            opt_mode = Some(raw_args.get(i).cloned().unwrap_or_else(|| "toggle".into()));
        } else if let Some(v) = a.strip_prefix("--mode=") {
            opt_mode = Some(v.to_string());
        } else if a == "--corners" {
            i += 1;
            opt_corners = Some(raw_args.get(i).cloned().unwrap_or_else(|| "toggle".into()));
        } else if let Some(v) = a.strip_prefix("--corners=") {
            opt_corners = Some(v.to_string());
        } else if a == "--nightlight" || a == "--night-light" {
            i += 1;
            opt_nl = Some(raw_args.get(i).cloned().unwrap_or_else(|| "toggle".into()));
        } else if let Some(v) = a.strip_prefix("--nightlight=").or_else(|| a.strip_prefix("--night-light=")) {
            opt_nl = Some(v.to_string());
        } else {
            pos.push(a.clone());
        }
        i += 1;
    }

    if let Some(m) = &opt_mode {
        let Some(resolved) = resolve_mode(m, &st.mode) else {
            eprintln!("invalid mode: {m}");
            process::exit(2);
        };
        st.mode = resolved.to_string();
    }
    if let Some(c) = &opt_corners {
        st.corners = if c == "toggle" {
            if st.corners == "rounded" { "square".into() } else { "rounded".into() }
        } else {
            c.clone()
        };
    }
    if let Some(nl) = &opt_nl {
        st.nightlight = if nl == "toggle" {
            if st.nightlight == "on" { "off".into() } else { "on".into() }
        } else {
            nl.clone()
        };
    }

    if pos.is_empty() && (opt_mode.is_some() || opt_corners.is_some() || opt_nl.is_some()) {
        let c = st.color.clone();
        let m = st.mode.clone();
        let cr = st.corners.clone();
        let nl = st.nightlight.clone();
        apply_all(&mut st, &c, &m, &cr, &nl);
        return;
    }

    let cmd = pos.first().map(|s| s.as_str()).unwrap_or("status");
    match cmd {
        "status" | "list" => print_status_and_swatches(&st, false),
        "swatches" => print_status_and_swatches(&st, true),
        "set" => {
            let arg_c = pos.get(1).cloned().unwrap_or_else(|| st.color.clone());
            if Path::new(&arg_c).is_file() {
                apply_wallpaper_file(&mut st, Path::new(&arg_c));
                return;
            }
            let Some(pal) = resolve_color(&arg_c) else {
                eprintln!("unknown color: {arg_c}");
                process::exit(2);
            };
            let m = pos.get(2).and_then(|x| resolve_mode(x, &st.mode)).unwrap_or(&st.mode).to_string();
            let cr = pos.get(3).cloned().unwrap_or_else(|| st.corners.clone());
            let nl = st.nightlight.clone();
            apply_all(&mut st, pal.id, &m, &cr, &nl);
        }
        "next" | "prev" => {
            let idx = PALETTES.iter().position(|p| p.id == st.color).unwrap_or(0);
            let step = if cmd == "prev" { PALETTES.len() - 1 } else { 1 };
            let next_id = PALETTES[(idx + step) % PALETTES.len()].id;
            let m = st.mode.clone();
            let cr = st.corners.clone();
            let nl = st.nightlight.clone();
            apply_all(&mut st, next_id, &m, &cr, &nl);
        }
        "toggle" | "mode" | "day" | "night" | "light" | "dark" => {
            let target = if cmd == "toggle" || cmd == "mode" {
                pos.get(1).map(|s| s.as_str()).unwrap_or("toggle")
            } else {
                pos.get(1).map(|s| s.as_str()).unwrap_or(cmd)
            };
            let Some(m) = resolve_mode(target, &st.mode) else {
                eprintln!("mode must be night, day, dark, light, or toggle");
                process::exit(2);
            };
            let c = st.color.clone();
            let cr = st.corners.clone();
            let nl = st.nightlight.clone();
            apply_all(&mut st, &c, m, &cr, &nl);
        }
        "corners" => {
            let target = pos.get(1).map(|s| s.as_str()).unwrap_or("toggle");
            let next_c = if target == "toggle" {
                if st.corners == "rounded" { "square" } else { "rounded" }
            } else if target == "rounded" || target == "square" {
                target
            } else {
                eprintln!("corners must be square, rounded, or toggle");
                process::exit(2);
            };
            let c = st.color.clone();
            let m = st.mode.clone();
            let nl = st.nightlight.clone();
            apply_all(&mut st, &c, &m, next_c, &nl);
        }
        "nightlight" | "night-light" => {
            let target = pos.get(1).map(|s| s.as_str()).unwrap_or("toggle");
            let next_nl = if target == "toggle" {
                if st.nightlight == "on" { "off" } else { "on" }
            } else {
                target
            };
            let c = st.color.clone();
            let m = st.mode.clone();
            let cr = st.corners.clone();
            apply_all(&mut st, &c, &m, &cr, next_nl);
        }
        "wallpaper" | "wall" | "wallpapers" => {
            let sub = pos.get(1).map(|s| s.as_str()).unwrap_or("next");
            match sub {
                "next" => rotate_wallpaper(&mut st, 1),
                "prev" => rotate_wallpaper(&mut st, -1),
                "list" => {
                    for w in list_wallpapers(&st) {
                        println!("{}", w.display());
                    }
                }
                "set" => {
                    if let Some(p) = pos.get(2) {
                        apply_wallpaper_file(&mut st, Path::new(p));
                    }
                }
                other if Path::new(other).is_file() => apply_wallpaper_file(&mut st, Path::new(other)),
                _ => rotate_wallpaper(&mut st, 1),
            }
        }
        "rgb" | "openrgb" => set_rgb(&st, pos.get(1).map(|s| s.as_str())),
        "profile" | "workstation-mode" => {
            let p = pos.get(1).map(|s| s.as_str()).unwrap_or("balanced");
            if p == "stealth" {
                set_rgb(&st, Some("off"));
            } else {
                set_rgb(&st, None);
            }
            println!("[OK] SuperTheme workstation profile synced: {p}");
        }
        "reload" => {
            let c = st.color.clone();
            let m = st.mode.clone();
            let cr = st.corners.clone();
            let nl = st.nightlight.clone();
            apply_all(&mut st, &c, &m, &cr, &nl);
        }
        "pick" => fuzzel_pick(&mut st),
        _ => {
            if let Some(pal) = resolve_color(cmd) {
                let m = st.mode.clone();
                let cr = st.corners.clone();
                let nl = st.nightlight.clone();
                apply_all(&mut st, pal.id, &m, &cr, &nl);
            } else {
                print_help();
                process::exit(2);
            }
        }
    }
}
