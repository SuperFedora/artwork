// Copyright 2026 SuperFedora Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::palettes::{PALETTES, ThemeState, ansi_rgb, resolve_color, resolve_mode, write_toolkit_themes};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn apply_all(st: &mut ThemeState, color_id: &str, mode: &str, corners: &str, nl: &str) {
    let pal = resolve_color(color_id).unwrap_or(&PALETTES[11]);
    let mode = if mode == "light" { "light" } else { "dark" };
    let corners = if corners == "rounded" { "rounded" } else { "square" };
    let nl = if nl == "on" { "on" } else { "off" };

    let (acc, hot, bg, bg_alt, bg_hov, border, fg, fg_dim, muted, sel_bg, lock_fg, lock_dim, lock_in, k_bg, k_fg, k_c0, k_c7, k_c8, k_c15, portal, gtk, tui_txt, tui_bg) = if mode == "light" {
        (
            pal.l_acc, pal.l_hot, "e6e8f0", "d8dae5", "cfd2e0", "9ea4bd", "1a1b26", "343b58", "636a8a", "c5cae0",
            "26, 27, 38", "52, 59, 88", "230, 232, 240", "#e6e8f0", "#1a1b26", "#d8dae5", "#1a1b26", "#636a8a", "#07080b",
            "prefer-light", "Adwaita", "black", "white",
        )
    } else {
        (
            pal.d_acc, pal.d_hot, "1a1b26", "24283b", "292e42", "414868", "c0caf5", "a9b1d6", "565f89", "283457",
            "205, 214, 244", "166, 173, 200", "26, 27, 38", "#1a1b26", "#c0caf5", "#1a1b26", "#c0caf5", "#565f89", "#ffffff",
            "prefer-dark", "Adwaita-dark", "white", "black",
        )
    };

    for d in [&st.state_dir, &st.state_dir.join("cast"), &st.home.join(".config/hypr/conf"), &st.home.join(".config/quickshell"), &st.home.join(".config/fuzzel"), &st.home.join(".config/kitty"), &st.home.join(".config/idlescreen")] {
        let _ = fs::create_dir_all(d);
    }
    for src_cast in [
        st.home.join("Projects/superfedora/supertheme/cast"),
        st.home.join(".local/share/supertheme/cast"),
        st.home.join(".local/share/superfedora/dotfiles/.config/superfedora/cast"),
        PathBuf::from("/etc/skel/.config/superfedora/cast"),
        PathBuf::from("/usr/share/superfedora/cast"),
    ] {
        if let Ok(entries) = fs::read_dir(&src_cast) {
            for e in entries.flatten() {
                let p = e.path();
                if let Some(name) = p.file_name() {
                    let dst = st.state_dir.join("cast").join(name);
                    if !dst.exists() { let _ = fs::copy(&p, &dst); }
                }
            }
            break;
        }
    }
    let _ = fs::write(st.state_dir.join("theme.env"), format!("THEME_COLOR=\"{}\"\nTHEME_MODE=\"{mode}\"\nTHEME_CORNERS=\"{corners}\"\nTHEME_NIGHTLIGHT=\"{nl}\"\n", pal.id));
    let _ = fs::write(st.state_dir.join("theme-corners"), format!("{corners}\n"));

    let (lock_r, hypr_r, qs_r, qs_rs, fuz_r) = if corners == "rounded" { ("10", "10", "10", "6", "10") } else { ("0", "0", "0", "0", "0") };
    let user_wall = st.home.join(format!("Pictures/Wallpapers/{mode}/{}.png", pal.wall_key));
    let sys_wall = PathBuf::from(format!("/usr/share/superfedora/wallpapers/{mode}/{}.png", pal.wall_key));
    let (wall_rel, wall_abs) = if user_wall.is_file() || !sys_wall.is_file() {
        (format!("~/Pictures/Wallpapers/{mode}/{}.png", pal.wall_key), user_wall)
    } else {
        (sys_wall.to_string_lossy().to_string(), sys_wall)
    };

    let _ = fs::write(st.home.join(".config/hypr/hyprpaper.conf"), format!("wallpaper {{\n    monitor =\n    path = {wall_rel}\n    fit_mode = cover\n}}\n\nsplash = false\n"));
    let _ = fs::write(st.home.join(".config/hypr/.current-wallpaper"), format!("{}\n", wall_abs.display()));
    if wall_abs.is_file() {
        let ws = wall_abs.to_string_lossy().to_string();
        run_quiet("hyprctl", &["hyprpaper", "unload", "all"]);
        run_quiet("hyprctl", &["hyprpaper", "preload", &ws]);
        run_quiet("hyprctl", &["hyprpaper", "wallpaper", &format!(",{ws}")]);
    }

    let lock_cfg = format!(
        "# Hyprlock Configuration — SuperFedora 16-Bit Theme ({} / {mode} / {corners})\ngeneral {{\n    hide_cursor = false\n    ignore_empty_input = true\n    immediate_render = true\n    fail_timeout = 1200\n}}\nauth {{\n    pam {{\n        enabled = true\n        module = password-auth\n    }}\n    fingerprint {{\n        enabled = true\n    }}\n}}\nanimations {{\n    enabled = false\n}}\nbackground {{\n    monitor =\n    path = {wall_rel}\n    blur_passes = 0\n    blur_size = 0\n}}\n{}{}{}{}{}{}\nimage {{\n    monitor =\n    path = ~/Pictures/Wallpapers/avatar.jpg\n    size = 88\n    rounding = {lock_r}\n    border_size = 2\n    border_color = rgb({acc})\n    position = 0, 148\n    halign = center\n    valign = bottom\n}}\n{}\ninput-field {{\n    monitor =\n    size = 290, 44\n    outline_thickness = 2\n    rounding = {lock_r}\n    dots_size = 0.26\n    dots_spacing = 0.20\n    dots_center = true\n    fade_on_empty = false\n    font_color = rgb({lock_fg})\n    inner_color = rgba({lock_in}, 0.94)\n    outer_color = rgb({acc})\n    check_color = rgb(9ece6a)\n    fail_color = rgb(f7768e)\n    fail_text = <b>Incorrect password ($ATTEMPTS)</b>\n    capslock_color = rgb(e0af68)\n    placeholder_text = Password\n    position = 0, 46\n    halign = center\n    valign = bottom\n}}\n",
        pal.name,
        lock_lbl("<b>\u{f0379} Screen Off</b>", &format!("rgba({lock_fg}, 1.0)"), "12", "-390, -24", "right", "top", "    onclick = ~/.config/hypr/scripts/toggle-touchpad.sh dpms off\n"),
        lock_lbl("<b>\u{f0904} Suspend</b>", &format!("rgba({lock_fg}, 1.0)"), "12", "-276, -24", "right", "top", "    onclick = systemctl suspend\n"),
        lock_lbl("<b>\u{f0709} Restart</b>", &format!("rgba({lock_fg}, 1.0)"), "12", "-162, -24", "right", "top", "    onclick = systemctl reboot\n"),
        lock_lbl("<b>\u{f0425} Power Off</b>", "rgb(f7768e)", "12", "-28, -24", "right", "top", "    onclick = systemctl poweroff\n"),
        lock_lbl("cmd[update:1000] ~/.local/bin/superlock status", &format!("rgb({acc})"), "12", "-28, 56", "right", "bottom", "    onclick = ~/.local/bin/superlock next\n"),
        lock_lbl("<b>\u{f0343} Switch DE / Log Out</b>", &format!("rgba({lock_dim}, 1.0)"), "12", "-28, 28", "right", "bottom", "    onclick = ~/.local/bin/superlock switch\n"),
        lock_lbl("<b>$USER</b>", &format!("rgba({lock_fg}, 1.0)"), "16", "0, 108", "center", "bottom", ""),
    );
    let _ = fs::write(st.home.join(".config/hypr/hyprlock.conf"), lock_cfg);

    let lnf_path = st.home.join(".config/hypr/conf/looknfeel.conf");
    if lnf_path.is_file() {
        let s = lnf_path.to_string_lossy().to_string();
        run_quiet("sed", &["-i",
            "-e", &format!("s/col\\.active_border = .*/col.active_border = rgb({acc})/"),
            "-e", &format!("s/col\\.inactive_border = .*/col.inactive_border = rgba({border}aa)/"),
            "-e", &format!("s/col\\.border_active = .*/col.border_active = rgb({acc})/"),
            "-e", &format!("s/col\\.border_inactive = .*/col.border_inactive = rgba({border}aa)/"),
            "-e", &format!("s/text_color = .*/text_color = rgb({fg})/"),
            "-e", &format!("s/col\\.active = .*/col.active = rgba({acc}cc)/"),
            "-e", &format!("s/col\\.inactive = .*/col.inactive = rgba({bg_alt}cc)/"),
            "-e", &format!("s/rounding = .*/rounding = {hypr_r}/"), &s,
        ]);
    }
    for (k, v) in [("general:col.active_border", format!("rgb({acc})")), ("general:col.inactive_border", format!("rgba({border}aa)")), ("decoration:rounding", hypr_r.into()), ("misc:allow_session_lock_restore", "1".into())] {
        run_quiet("hyprctl", &["keyword", k, &v]);
    }

    let nl_bool = if nl == "on" { "true" } else { "false" };
    let qs_json = format!(
        "{{\"theme\":\"{}\",\"name\":\"{}\",\"mode\":\"{mode}\",\"bg\":\"#{bg}\",\"bgAlt\":\"#{bg_alt}\",\"bgHover\":\"#{bg_hov}\",\"border\":\"#{border}\",\"fg\":\"#{fg}\",\"fgDim\":\"#{fg_dim}\",\"muted\":\"#{muted}\",\"blue\":\"#{acc}\",\"cyan\":\"#{hot}\",\"corners\":\"{corners}\",\"nightlight\":{nl_bool},\"radius\":{qs_r},\"radiusSm\":{qs_rs}}}\n",
        pal.id, pal.name
    );
    let _ = fs::write(st.state_dir.join("quickshell-theme.json"), qs_json);
    run_quiet("qs", &["ipc", "call", "theme", "reload"]);

    let fuzzel = format!(
        "# Fuzzel Configuration — SuperFedora 16-Bit Theme ({} / {mode} / {corners})\nfont=JetBrainsMono Nerd Font:size=12\ndpi-aware=auto\nprompt=\"> \"\nicon-theme=Adwaita\nterminal=kitty -e\nwidth=36\nlines=10\nhorizontal-pad=24\nvertical-pad=16\ninner-pad=10\n\n[colors]\nbackground={bg}f2\ntext={fg}ff\nmatch={acc}ff\nselection={sel_bg}ff\nselection-text={fg}ff\nselection-match={hot}ff\nborder={acc}ff\n\n[border]\nwidth=2\nradius={fuz_r}\n",
        pal.name
    );
    let _ = fs::write(st.home.join(".config/fuzzel/fuzzel.ini"), fuzzel);

    let kitty_p = st.home.join(".config/kitty/kitty.conf");
    if kitty_p.is_file() {
        let ks = kitty_p.to_string_lossy().to_string();
        run_quiet("sed", &["-i",
            "-e", &format!("s/^foreground .*/foreground            {k_fg}/"),
            "-e", &format!("s/^background .*/background            {k_bg}/"),
            "-e", &format!("s/^selection_background .*/selection_background   #{acc}/"),
            "-e", &format!("s/^cursor  .*/cursor                #{acc}/"),
            "-e", &format!("s/^cursor_text_color .*/cursor_text_color     {k_bg}/"),
            "-e", &format!("s/^color0 .*/color0                {k_c0}/"),
            "-e", &format!("s/^color7 .*/color7                {k_c7}/"),
            "-e", &format!("s/^color8 .*/color8                {k_c8}/"),
            "-e", &format!("s/^color15 .*/color15               {k_c15}/"), &ks,
        ]);
        run_quiet("pkill", &["-USR1", "-x", "kitty"]);
    }

    let _ = fs::write(st.state_dir.join("ghostty-theme.conf"), format!("background = #{bg}\nforeground = #{fg}\ncursor-color = #{acc}\nselection-background = #{sel_bg}\n"));
    let tmux_p = st.state_dir.join("tmux-theme.conf");
    let _ = fs::write(&tmux_p, format!("set -g status-style \"bg=#{bg_alt},fg=#{fg}\"\nset -g pane-active-border-style \"fg=#{acc}\"\nset -g pane-border-style \"fg=#{border}\"\n"));
    run_quiet("tmux", &["source-file", &tmux_p.to_string_lossy()]);
    let _ = fs::write(st.state_dir.join("nvim-theme.lua"), format!("vim.o.background = \"{mode}\"\n"));

    let idle_p = st.home.join(".config/idlescreen/config.toml");
    if !idle_p.exists() {
        let _ = fs::write(&idle_p, "[general]\nidle_timeout_secs = 120\nprimary_output = \"\"\n\n[saver]\nactive = \"superfedora\"\n\n[saver.ascii]\nmode = \"superfedora\"\ntargets = [\"superfedora\"]\ntext = \"\"\nfps = 0\ndwell_secs = 5.0\ndissolve_duration_secs = 0.699999988079071\n");
    }

    run_quiet("gsettings", &["set", "org.gnome.desktop.interface", "color-scheme", portal]);
    run_quiet("gsettings", &["set", "org.gnome.desktop.interface", "gtk-theme", gtk]);
    write_toolkit_themes(&st.home, &st.state_dir, mode, gtk, acc, hot, bg, bg_alt, bg_hov, border, fg, fg_dim, muted);

    if nl == "on" {
        let _ = Command::new("sh").args(["-c", "pgrep -x wlsunset >/dev/null 2>&1 || pgrep -x hyprsunset >/dev/null 2>&1 || (command -v hyprsunset >/dev/null 2>&1 && nohup hyprsunset -t 4200 >/dev/null 2>&1 &) || (command -v wlsunset >/dev/null 2>&1 && nohup wlsunset -t 4200 -T 6500 >/dev/null 2>&1 &)"]).status();
    } else {
        run_quiet("pkill", &["-x", "wlsunset"]);
        run_quiet("pkill", &["-x", "hyprsunset"]);
    }

    let _ = fs::write(st.state_dir.join("rgb-color"), format!("{acc}\n"));
    if st.state_dir.join("rgb-devices").metadata().map(|m| m.len() > 0).unwrap_or(false) {
        let _ = Command::new("openrgb").args(["--noautoconnect", "-m", "direct", "-c", acc]).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
    }

    let greetd = Path::new("/etc/greetd/config.toml");
    if greetd.exists() {
        let tui_theme = format!("border={};title={};prompt={};time={tui_txt};action={};button={};container={tui_bg};input={tui_txt}", pal.tui_c, pal.tui_c, pal.tui_c, pal.tui_c, pal.tui_c);
        let _ = fs::write(greetd, format!("# SuperFedora login screen: greetd + tuigreet on tty1.\n[terminal]\nvt = 1\n\n[default_session]\ncommand = \"tuigreet --time --asterisks --remember --remember-session --theme '{tui_theme}' --sessions /usr/share/wayland-sessions --cmd 'uwsm start -e -D Hyprland hyprland.desktop' --greeting 'SUPER FEDORA'\"\nuser = \"greetd\"\n"));
    }

    let mode_disp = if mode == "light" { "day" } else { "night" };
    println!("[OK] Active 16-bit theme: {} ({mode_disp}/{mode} mode, {corners} corners, nightlight: {nl})", pal.name);
    st.refresh();
}

fn lock_lbl(text: &str, col: &str, sz: &str, pos: &str, ha: &str, va: &str, extra: &str) -> String {
    format!("label {{\n    monitor =\n    text = {text}\n    color = {col}\n    font_size = {sz}\n    font_family = JetBrainsMono Nerd Font\n    position = {pos}\n    halign = {ha}\n    valign = {va}\n{extra}}}\n")
}

pub fn run_quiet(cmd: &str, args: &[&str]) {
    let _ = Command::new(cmd).args(args).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

pub fn list_wallpapers(st: &ThemeState) -> Vec<PathBuf> {
    let user_dir = st.home.join("Pictures/Wallpapers").join(&st.mode);
    let dir = if user_dir.is_dir() { user_dir } else { PathBuf::from("/usr/share/superfedora/wallpapers").join(&st.mode) };
    let mut walls: Vec<PathBuf> = fs::read_dir(&dir).ok().into_iter().flatten().flatten().map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()).map(|x| matches!(x, "png" | "jpg" | "jpeg" | "webp")).unwrap_or(false))
        .collect();
    walls.sort();
    walls
}

pub fn apply_wallpaper_file(st: &mut ThemeState, file: &Path) {
    let abs = fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
    let s = abs.to_string_lossy().to_string();
    run_quiet("hyprctl", &["hyprpaper", "preload", &s]);
    run_quiet("hyprctl", &["hyprpaper", "wallpaper", &format!(",{s}")]);
    let _ = fs::write(st.home.join(".config/hypr/.current-wallpaper"), format!("{s}\n"));
    let _ = fs::write(st.home.join(".config/hypr/hyprpaper.conf"), format!("wallpaper {{\n    monitor =\n    path = {s}\n    fit_mode = cover\n}}\n\nsplash = false\n"));
    st.refresh();
    println!("[OK] Applied wallpaper: {s}");
}

pub fn rotate_wallpaper(st: &mut ThemeState, step: isize) {
    let walls = list_wallpapers(st);
    if walls.is_empty() { return; }
    let cur_idx = walls.iter().position(|p| p.to_string_lossy() == st.current_wall).unwrap_or(0);
    let next_idx = (cur_idx as isize + step).rem_euclid(walls.len() as isize) as usize;
    apply_wallpaper_file(st, &walls[next_idx]);
}

pub fn set_rgb(st: &ThemeState, raw_hex: Option<&str>) {
    let default_hex = if st.mode == "light" { st.palette().l_acc } else { st.palette().d_acc };
    let mut hex = raw_hex.unwrap_or(default_hex).trim().trim_start_matches('#').to_string();
    if hex.eq_ignore_ascii_case("off") { hex = "000000".into(); }
    let _ = fs::create_dir_all(&st.state_dir);
    let _ = fs::write(st.state_dir.join("rgb-color"), format!("{hex}\n"));
    run_quiet("openrgb", &["--noautoconnect", "-m", "direct", "-c", &hex]);
    println!("[OK] RGB lighting set to #{hex}");
}

pub fn fuzzel_pick(st: &mut ThemeState) {
    let other_m = if st.mode == "light" { "night" } else { "day" };
    let other_c = if st.corners == "rounded" { "square" } else { "rounded" };
    let mut input = format!("[*] Toggle mode ({} -> {other_m})\n[*] Toggle corners ({} -> {other_c})\n", st.mode, st.corners);
    for p in PALETTES { input.push_str(p.id); input.push('\n'); }
    let Ok(mut child) = Command::new("fuzzel").args(["--dmenu", "--prompt", "16-Bit Theme > ", "--lines", "12", "--width", "42"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn() else { return };
    if let Some(mut stdin) = child.stdin.take() { let _ = stdin.write_all(input.as_bytes()); }
    let Ok(out) = child.wait_with_output() else { return };
    if !out.status.success() { return; }
    let choice = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let (c, m, cr, nl) = (st.color.clone(), st.mode.clone(), st.corners.clone(), st.nightlight.clone());
    if choice.starts_with("[*] Toggle mode") {
        apply_all(st, &c, resolve_mode("toggle", &m).unwrap_or("dark"), &cr, &nl);
    } else if choice.starts_with("[*] Toggle corners") {
        apply_all(st, &c, &m, other_c, &nl);
    } else if let Some(p) = resolve_color(&choice) {
        apply_all(st, p.id, &m, &cr, &nl);
    }
}

pub fn print_status_and_swatches(st: &ThemeState, swatches: bool) {
    let p = st.palette();
    let ids: Vec<&str> = PALETTES.iter().map(|x| x.id).collect();
    println!("Active Theme : {} ({}) [mode: {}, corners: {}, nightlight: {}]\nPalettes     : {}", p.name, st.color, st.mode, st.corners, st.nightlight, ids.join(" "));
    if swatches {
        println!("\n{}\x1b[1m━━ 16-BIT RAINBOW PALETTE STUDIO (Active: {})\x1b[0m", ansi_rgb(p.pri_rgb), st.color);
        for pal in PALETTES {
            let act = if pal.id == st.color { "\x1b[1;32m* ACTIVE\x1b[0m" } else { "        " };
            println!("  {}  \x1b[1m{:<14}\x1b[0m {:<10} {}████████████\x1b[0m {}████████\x1b[0m", act, pal.id, pal.name, ansi_rgb(pal.pri_rgb), ansi_rgb(pal.sec_rgb));
        }
    }
}
