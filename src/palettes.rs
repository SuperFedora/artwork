// Copyright 2026 SuperFedora Contributors
// SPDX-License-Identifier: Apache-2.0

use ratatui::style::Color;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub struct Palette {
    pub id: &'static str,
    pub name: &'static str,
    pub wall_key: &'static str,
    pub d_acc: &'static str,
    pub d_hot: &'static str,
    pub l_acc: &'static str,
    pub l_hot: &'static str,
    pub tui_c: &'static str,
    pub pri_rgb: (u8, u8, u8),
    pub sec_rgb: (u8, u8, u8),
}

pub const PALETTES: &[Palette] = &[
    Palette { id: "01-crimson", name: "Crimson", wall_key: "03-super-ruby", d_acc: "f7768e", d_hot: "ffbac7", l_acc: "b91c3c", l_hot: "e11d48", tui_c: "red", pri_rgb: (247, 118, 142), sec_rgb: (255, 186, 199) },
    Palette { id: "02-scarlet", name: "Scarlet", wall_key: "08-neon-coral", d_acc: "ff5a5f", d_hot: "ff8fa3", l_acc: "c81e32", l_hot: "e11d48", tui_c: "lightred", pri_rgb: (255, 90, 95), sec_rgb: (255, 143, 163) },
    Palette { id: "03-ember", name: "Ember", wall_key: "07-flame-orange", d_acc: "ff7b54", d_hot: "ffb26b", l_acc: "c2410c", l_hot: "ea580c", tui_c: "lightred", pri_rgb: (255, 123, 84), sec_rgb: (255, 178, 107) },
    Palette { id: "04-amber", name: "Amber", wall_key: "07-flame-orange", d_acc: "ff9e64", d_hot: "ffc98b", l_acc: "b84309", l_hot: "ea580c", tui_c: "yellow", pri_rgb: (255, 158, 100), sec_rgb: (255, 201, 139) },
    Palette { id: "05-topaz", name: "Topaz", wall_key: "05-star-gold", d_acc: "e0af68", d_hot: "ffdf9e", l_acc: "946210", l_hot: "ca8a04", tui_c: "yellow", pri_rgb: (224, 175, 104), sec_rgb: (255, 223, 158) },
    Palette { id: "06-citrine", name: "Citrine", wall_key: "05-star-gold", d_acc: "f9e2af", d_hot: "fff3b0", l_acc: "854d0e", l_hot: "ca8a04", tui_c: "yellow", pri_rgb: (249, 226, 175), sec_rgb: (255, 243, 176) },
    Palette { id: "07-peridot", name: "Peridot", wall_key: "04-emerald-green", d_acc: "a6e3a1", d_hot: "d8f3dc", l_acc: "15803d", l_hot: "16a34a", tui_c: "lightgreen", pri_rgb: (166, 227, 161), sec_rgb: (216, 243, 220) },
    Palette { id: "08-emerald", name: "Emerald", wall_key: "04-emerald-green", d_acc: "9ece6a", d_hot: "cbf0a3", l_acc: "2d7a31", l_hot: "16a34a", tui_c: "green", pri_rgb: (158, 206, 106), sec_rgb: (203, 240, 163) },
    Palette { id: "09-verdant", name: "Verdant", wall_key: "04-emerald-green", d_acc: "73daca", d_hot: "a8f9ff", l_acc: "0f766e", l_hot: "0d9488", tui_c: "cyan", pri_rgb: (115, 218, 202), sec_rgb: (168, 249, 255) },
    Palette { id: "10-lagoon", name: "Lagoon", wall_key: "06-electric-cyan", d_acc: "2ac3de", d_hot: "89dceb", l_acc: "0e7490", l_hot: "0891b2", tui_c: "cyan", pri_rgb: (42, 195, 222), sec_rgb: (137, 220, 235) },
    Palette { id: "11-glacier", name: "Glacier", wall_key: "06-electric-cyan", d_acc: "7dcfff", d_hot: "bfeaff", l_acc: "0369a1", l_hot: "0284c7", tui_c: "lightcyan", pri_rgb: (125, 207, 255), sec_rgb: (191, 234, 255) },
    Palette { id: "12-cobalt", name: "Cobalt", wall_key: "01-super-blue", d_acc: "7aa2f7", d_hot: "bcd3ff", l_acc: "2554a3", l_hot: "3c6eb4", tui_c: "blue", pri_rgb: (122, 162, 247), sec_rgb: (188, 211, 255) },
    Palette { id: "13-indigo", name: "Indigo", wall_key: "01-super-blue", d_acc: "6c71c4", d_hot: "a5abff", l_acc: "3730a3", l_hot: "4f46e5", tui_c: "lightblue", pri_rgb: (108, 113, 196), sec_rgb: (165, 171, 255) },
    Palette { id: "14-violet", name: "Violet", wall_key: "02-metroid-violet", d_acc: "bb9af7", d_hot: "decbff", l_acc: "6b38b8", l_hot: "8b5cf6", tui_c: "magenta", pri_rgb: (187, 154, 247), sec_rgb: (222, 203, 255) },
    Palette { id: "15-orchid", name: "Orchid", wall_key: "02-metroid-violet", d_acc: "cba6f7", d_hot: "f5c2e7", l_acc: "7e22ce", l_hot: "a855f7", tui_c: "lightmagenta", pri_rgb: (203, 166, 247), sec_rgb: (245, 194, 231) },
    Palette { id: "16-fuchsia", name: "Fuchsia", wall_key: "08-neon-coral", d_acc: "f43f5e", d_hot: "ff99ac", l_acc: "9f1239", l_hot: "be123c", tui_c: "lightmagenta", pri_rgb: (244, 63, 94), sec_rgb: (255, 153, 172) },
];

pub fn resolve_color(raw: &str) -> Option<&'static Palette> {
    let s = raw.trim().to_ascii_lowercase();
    let id = match s.as_str() {
        "1" | "01" | "crimson" | "01-crimson" | "ruby" | "red" | "03-super-ruby" => "01-crimson",
        "2" | "02" | "scarlet" | "02-scarlet" => "02-scarlet",
        "3" | "03" | "ember" | "03-ember" => "03-ember",
        "4" | "04" | "amber" | "04-amber" | "orange" | "07-flame-orange" => "04-amber",
        "5" | "05" | "topaz" | "05-topaz" | "gold" | "05-star-gold" => "05-topaz",
        "6" | "06" | "citrine" | "06-citrine" | "yellow" => "06-citrine",
        "7" | "07" | "peridot" | "07-peridot" | "lime" => "07-peridot",
        "8" | "08" | "emerald" | "08-emerald" | "green" | "04-emerald-green" => "08-emerald",
        "9" | "09" | "verdant" | "09-verdant" | "mint" => "09-verdant",
        "10" | "lagoon" | "10-lagoon" | "teal" => "10-lagoon",
        "11" | "glacier" | "11-glacier" | "cyan" | "06-electric-cyan" | "nord" => "11-glacier",
        "12" | "cobalt" | "12-cobalt" | "blue" | "01-super-blue" | "tokyo" | "tokyo-night" => "12-cobalt",
        "13" | "indigo" | "13-indigo" => "13-indigo",
        "14" | "violet" | "14-violet" | "purple" | "02-metroid-violet" | "snes" => "14-violet",
        "15" | "orchid" | "15-orchid" | "catppuccin" => "15-orchid",
        "16" | "fuchsia" | "16-fuchsia" | "coral" | "magenta" | "08-neon-coral" => "16-fuchsia",
        _ => return None,
    };
    PALETTES.iter().find(|p| p.id == id)
}

pub fn resolve_mode(raw: &str, current: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "day" | "light" => Some("light"),
        "night" | "dark" => Some("dark"),
        "toggle" => Some(if current == "light" { "dark" } else { "light" }),
        _ => None,
    }
}

pub fn ansi_rgb(rgb: (u8, u8, u8)) -> String {
    format!("\x1b[38;2;{};{};{}m", rgb.0, rgb.1, rgb.2)
}

#[derive(Clone)]
pub struct ThemeState {
    pub home: PathBuf,
    pub state_dir: PathBuf,
    pub color: String,
    pub mode: String,
    pub corners: String,
    pub nightlight: String,
    pub current_wall: String,
    pub sfx_enabled: bool,
}

impl ThemeState {
    pub fn load() -> Self {
        let home = PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/home/jeryd".into()));
        let state_dir = env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".config"))
            .join("superfedora");
        let mut st = Self {
            home: home.clone(),
            state_dir: state_dir.clone(),
            color: "12-cobalt".into(),
            mode: "dark".into(),
            corners: "square".into(),
            nightlight: "off".into(),
            current_wall: String::new(),
            sfx_enabled: true,
        };
        st.refresh();
        st
    }

    pub fn refresh(&mut self) {
        if let Ok(txt) = fs::read_to_string(self.state_dir.join("theme.env")) {
            for line in txt.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    let val = v.trim().trim_matches('"').trim_matches('\'');
                    match k.trim() {
                        "THEME_COLOR" => {
                            if let Some(p) = resolve_color(val) {
                                self.color = p.id.to_string();
                            }
                        }
                        "THEME_MODE" => self.mode = if val == "light" { "light".into() } else { "dark".into() },
                        "THEME_CORNERS" => self.corners = if val == "rounded" { "rounded".into() } else { "square".into() },
                        "THEME_NIGHTLIGHT" => self.nightlight = if val == "on" { "on".into() } else { "off".into() },
                        _ => {}
                    }
                }
            }
        }
        self.current_wall = fs::read_to_string(self.home.join(".config/hypr/.current-wallpaper"))
            .unwrap_or_default()
            .trim()
            .to_string();
        self.sfx_enabled = fs::read_to_string(self.state_dir.join("sfx.conf"))
            .map(|s| !s.contains("SFX_ENABLED=0"))
            .unwrap_or(true);
    }

    pub fn palette(&self) -> &'static Palette {
        resolve_color(&self.color).unwrap_or(&PALETTES[11])
    }

    pub fn primary_color(&self) -> Color {
        let p = self.palette().pri_rgb;
        Color::Rgb(p.0, p.1, p.2)
    }

    pub fn secondary_color(&self) -> Color {
        let s = self.palette().sec_rgb;
        Color::Rgb(s.0, s.1, s.2)
    }
}
