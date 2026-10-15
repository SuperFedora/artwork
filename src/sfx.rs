// Copyright 2026 SuperFedora Contributors
// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::f64::consts::PI;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const SR: usize = 44100;

pub fn cache_dir() -> PathBuf {
    let base = env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| "/home/jeryd".into());
            PathBuf::from(home).join(".cache")
        });
    base.join("superfedora/sfx")
}

pub fn state_dir() -> PathBuf {
    let base = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| "/home/jeryd".into());
            PathBuf::from(home).join(".config")
        });
    base.join("superfedora")
}

pub fn is_enabled() -> bool {
    fs::read_to_string(state_dir().join("sfx.conf"))
        .map(|s| !s.contains("SFX_ENABLED=0"))
        .unwrap_or(true)
}

pub fn save_enabled(en: bool) {
    let dir = state_dir();
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(dir.join("sfx.conf"), format!("SFX_ENABLED={}\n", if en { 1 } else { 0 }));
    let _ = fs::write(dir.join("sfx"), if en { "on\n" } else { "off\n" });
    let _ = Command::new("qs")
        .args(["ipc", "call", "status", "refresh"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn synth_pcm(notes: &[(f64, f64)], vol: f64, duty: f64) -> Vec<i16> {
    let mut samples = Vec::new();
    for &(freq, dur) in notes {
        let n = ((SR as f64) * dur) as usize;
        let atk = ((SR as f64) * 0.004).max(1.0);
        for i in 0..n {
            let t = (i as f64) / (SR as f64);
            let env = ((i as f64) / atk).min(1.0) * (-3.5 * (i as f64) / (n.max(1) as f64)).exp();
            let val = if freq <= 0.0 {
                0.0
            } else {
                let phase = (t * freq).fract();
                let sq = if phase < duty { 1.0 } else { -1.0 };
                let sub = (2.0 * PI * (freq * 0.5) * t).sin() * 0.35;
                (sq * 0.65 + sub) * vol * env
            };
            let s = (val * 32767.0).clamp(-32767.0, 32767.0) as i16;
            samples.push(s);
        }
    }
    let echo_delay = ((SR as f64) * 0.055) as usize;
    let mut out = vec![0i16; samples.len() + echo_delay];
    out[..samples.len()].copy_from_slice(&samples);
    for (i, &s) in samples.iter().enumerate() {
        let echoed = (out[i + echo_delay] as i32) + ((s as f64 * 0.28) as i32);
        out[i + echo_delay] = echoed.clamp(-32767, 32767) as i16;
    }
    out
}

fn write_wav(path: &Path, pcm: &[i16]) {
    let Ok(mut f) = File::create(path) else { return };
    let data_len = (pcm.len() * 2) as u32;
    let file_len = 36 + data_len;
    let byte_rate = (SR as u32) * 2;
    let mut buf = Vec::with_capacity(44 + data_len as usize);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_len.to_le_bytes());
    buf.extend_from_slice(b"WAVEfmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&(SR as u32).to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&16u16.to_le_bytes());
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());
    for &sample in pcm {
        buf.extend_from_slice(&sample.to_le_bytes());
    }
    let _ = f.write_all(&buf);
}

pub fn ensure_bank_at(dir: &Path) {
    type SfxBank<'a> = (&'a str, &'a [(f64, f64)], f64, f64);
    let _ = fs::create_dir_all(dir);
    let banks: &[SfxBank<'_>] = &[
        ("startup", &[(987.77, 0.075), (1318.51, 0.32)], 0.22, 0.5),
        ("click", &[(1046.50, 0.018), (1567.98, 0.022)], 0.14, 0.25),
        ("confirm", &[(523.25, 0.045), (659.25, 0.045), (783.99, 0.045), (1046.50, 0.14)], 0.18, 0.5),
        ("toggle", &[(659.25, 0.035), (987.77, 0.065)], 0.16, 0.35),
        ("volume", &[(880.00, 0.025)], 0.13, 0.5),
        ("alert", &[(392.00, 0.07), (311.13, 0.14)], 0.20, 0.5),
    ];
    for &(name, notes, vol, duty) in banks {
        let p = dir.join(format!("{name}.wav"));
        if !p.exists() {
            write_wav(&p, &synth_pcm(notes, vol, duty));
        }
    }
}

pub fn ensure_bank() -> PathBuf {
    let dir = cache_dir();
    ensure_bank_at(&dir);
    dir
}

pub fn play_sound(raw_name: &str) {
    if !is_enabled() {
        return;
    }
    let name = match raw_name.trim().to_ascii_lowercase().as_str() {
        "boot" | "startup" => "startup",
        "tick" | "click" => "click",
        "ok" | "confirm" => "confirm",
        "toggle" => "toggle",
        "vol" | "volume" => "volume",
        "err" | "alert" => "alert",
        _ => "click",
    };
    let dir = ensure_bank();
    let wav = dir.join(format!("{name}.wav"));
    if !wav.exists() {
        return;
    }
    for player in ["pw-play", "paplay", "aplay"] {
        let mut c = Command::new(player);
        if player == "aplay" {
            c.arg("-q");
        }
        c.arg(&wav).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        if c.spawn().is_ok() {
            break;
        }
    }
}

pub fn handle_sfx_cli(args: &[String]) -> i32 {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("status");
    match cmd {
        "status" => {
            println!("{}", if is_enabled() { "on" } else { "off" });
            0
        }
        "on" | "enable" | "unmute" => {
            save_enabled(true);
            play_sound("confirm");
            println!("[OK] 16-bit sound effects enabled");
            0
        }
        "off" | "disable" | "mute" => {
            save_enabled(false);
            println!("[OK] 16-bit sound effects muted");
            0
        }
        "toggle" => {
            let next = !is_enabled();
            save_enabled(next);
            if next {
                play_sound("confirm");
                println!("[OK] 16-bit sound effects enabled");
            } else {
                println!("[OK] 16-bit sound effects muted");
            }
            0
        }
        "init" => {
            let dir = args.get(1).map(PathBuf::from).unwrap_or_else(cache_dir);
            ensure_bank_at(&dir);
            println!("[OK] Synthesized 16-bit WAV sound bank in {}", dir.display());
            0
        }
        "play" => {
            play_sound(args.get(1).map(|s| s.as_str()).unwrap_or("click"));
            0
        }
        "startup" | "boot" | "click" | "tick" | "confirm" | "ok" | "volume" | "vol" | "alert" | "err" => {
            play_sound(cmd);
            0
        }
        "-h" | "--help" | "help" => {
            println!("Usage: supertheme sfx [status|on|off|toggle|init|play <chime>]");
            0
        }
        _ => {
            eprintln!("unknown sfx command: {cmd}");
            2
        }
    }
}
