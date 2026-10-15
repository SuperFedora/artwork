// Copyright 2026 SuperFedora Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::engine::{apply_all, list_wallpapers, rotate_wallpaper};
use crate::palettes::{PALETTES, ThemeState};
use crate::sfx::{handle_sfx_cli, play_sound};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io;
use std::time::Duration;

pub fn run_tui() -> i32 {
    let mut st = ThemeState::load();
    play_sound("startup");
    if enable_raw_mode().is_err() {
        return 1;
    }
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let Ok(mut terminal) = Terminal::new(CrosstermBackend::new(stdout)) else {
        let _ = disable_raw_mode();
        return 1;
    };

    let mut sel = PALETTES.iter().position(|p| p.id == st.color).unwrap_or(11);
    let mut status_msg = String::from("SuperTheme 16-Bit Studio Ready");

    loop {
        let _ = terminal.draw(|f| render_ui(f, &st, sel, &status_msg));

        if event::poll(Duration::from_millis(250)).unwrap_or(false)
            && let Ok(Event::Key(key)) = event::read()
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => sel = (sel + 1) % PALETTES.len(),
                KeyCode::Up | KeyCode::Char('k') => sel = (sel + PALETTES.len() - 1) % PALETTES.len(),
                KeyCode::Enter => {
                    let id = PALETTES[sel].id;
                    let m = st.mode.clone();
                    let c = st.corners.clone();
                    let nl = st.nightlight.clone();
                    apply_all(&mut st, id, &m, &c, &nl);
                    play_sound("confirm");
                    status_msg = format!("Applied palette: {id} ({m}, {c})");
                }
                KeyCode::Char(']') | KeyCode::Char('n') => {
                    sel = (sel + 1) % PALETTES.len();
                    let id = PALETTES[sel].id;
                    let m = st.mode.clone();
                    let c = st.corners.clone();
                    let nl = st.nightlight.clone();
                    apply_all(&mut st, id, &m, &c, &nl);
                    play_sound("click");
                    status_msg = format!("Rotated next palette: {id}");
                }
                KeyCode::Char('[') | KeyCode::Char('p') => {
                    sel = (sel + PALETTES.len() - 1) % PALETTES.len();
                    let id = PALETTES[sel].id;
                    let m = st.mode.clone();
                    let c = st.corners.clone();
                    let nl = st.nightlight.clone();
                    apply_all(&mut st, id, &m, &c, &nl);
                    play_sound("click");
                    status_msg = format!("Rotated prev palette: {id}");
                }
                KeyCode::Char('m') => {
                    let next_m = if st.mode == "light" { "dark" } else { "light" };
                    let id = st.color.clone();
                    let c = st.corners.clone();
                    let nl = st.nightlight.clone();
                    apply_all(&mut st, &id, next_m, &c, &nl);
                    play_sound("toggle");
                    status_msg = format!("Toggled mode: {next_m}");
                }
                KeyCode::Char('c') => {
                    let next_c = if st.corners == "rounded" { "square" } else { "rounded" };
                    let id = st.color.clone();
                    let m = st.mode.clone();
                    let nl = st.nightlight.clone();
                    apply_all(&mut st, &id, &m, next_c, &nl);
                    play_sound("toggle");
                    status_msg = format!("Toggled corners: {next_c}");
                }
                KeyCode::Char('l') => {
                    let next_nl = if st.nightlight == "on" { "off" } else { "on" };
                    let id = st.color.clone();
                    let m = st.mode.clone();
                    let c = st.corners.clone();
                    apply_all(&mut st, &id, &m, &c, next_nl);
                    play_sound("toggle");
                    status_msg = format!("Nightlight: {next_nl}");
                }
                KeyCode::Char('w') => {
                    rotate_wallpaper(&mut st, 1);
                    play_sound("click");
                    status_msg = format!("Rotated wallpaper: {}", st.current_wall);
                }
                KeyCode::Char('W') => {
                    rotate_wallpaper(&mut st, -1);
                    play_sound("click");
                    status_msg = format!("Rotated prev wallpaper: {}", st.current_wall);
                }
                KeyCode::Char('s') => {
                    let _ = handle_sfx_cli(&["toggle".into()]);
                    st.refresh();
                    status_msg = format!("16-Bit SFX: {}", if st.sfx_enabled { "ON" } else { "MUTED" });
                }
                KeyCode::Char('1') => { play_sound("startup"); status_msg = "Played SFX: startup".into(); }
                KeyCode::Char('2') => { play_sound("click"); status_msg = "Played SFX: click".into(); }
                KeyCode::Char('3') => { play_sound("confirm"); status_msg = "Played SFX: confirm".into(); }
                KeyCode::Char('4') => { play_sound("toggle"); status_msg = "Played SFX: toggle".into(); }
                KeyCode::Char('5') => { play_sound("volume"); status_msg = "Played SFX: volume".into(); }
                KeyCode::Char('6') => { play_sound("alert"); status_msg = "Played SFX: alert".into(); }
                _ => {}
            }
        }
    }

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
    0
}

fn render_ui(f: &mut Frame, st: &ThemeState, selected: usize, status_msg: &str) {
    let pri = st.primary_color();
    let sec = st.secondary_color();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(14), Constraint::Length(3)])
        .split(f.area());

    let hdr = Paragraph::new(Line::from(vec![
        Span::styled(" [::] SUPERTHEME v0.2.0 ", Style::default().fg(Color::Black).bg(pri).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("  Active: {} ({})  |  Corners: {}  |  SFX: {} ", st.color, st.mode, st.corners, if st.sfx_enabled { "ON" } else { "OFF" }),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(pri)));
    f.render_widget(hdr, chunks[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(56), Constraint::Percentage(44)])
        .split(chunks[1]);

    let items: Vec<ListItem> = PALETTES
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_sel = i == selected;
            let is_act = p.id == st.color;
            let pc = Color::Rgb(p.pri_rgb.0, p.pri_rgb.1, p.pri_rgb.2);
            let sc = Color::Rgb(p.sec_rgb.0, p.sec_rgb.1, p.sec_rgb.2);
            ListItem::new(Line::from(vec![
                Span::styled(if is_sel { " > " } else { "   " }, Style::default().fg(pri).add_modifier(Modifier::BOLD)),
                Span::styled(if is_act { "[ACTIVE] " } else { "         " }, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:<13} {:<9} ", p.id, p.name),
                    Style::default()
                        .fg(if is_sel { Color::Black } else { Color::White })
                        .bg(if is_sel { pc } else { Color::Reset })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("██████████", Style::default().fg(pc)),
                Span::raw(" "),
                Span::styled("██████", Style::default().fg(sc)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(pri))
            .title(Span::styled(" 16-BIT RAINBOW PALETTES (Enter=Apply, [/]=Rotate) ", Style::default().fg(pri).add_modifier(Modifier::BOLD))),
    );
    f.render_widget(list, cols[0]);

    render_right_panel(f, cols[1], st, selected);

    let ftr = Paragraph::new(Line::from(vec![
        Span::styled(" KEYS ", Style::default().fg(Color::Black).bg(sec).add_modifier(Modifier::BOLD)),
        Span::raw(" [Enter] Apply  [n/p] Rotate  [m] Light/Dark  [c] Corners  [w] Wallpaper  [l] Nightlight  [s] SFX  [1-6] Chimes  [q] Quit  | "),
        Span::styled(status_msg, Style::default().fg(pri).add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(sec)));
    f.render_widget(ftr, chunks[2]);
}

fn render_right_panel(f: &mut Frame, area: Rect, st: &ThemeState, selected: usize) {
    let pri = st.primary_color();
    let sec = st.secondary_color();
    let sel_p = &PALETTES[selected];
    let walls = list_wallpapers(st);
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("  Selected Palette : ", Style::default().fg(sec).add_modifier(Modifier::BOLD)), Span::styled(format!("{} ({})", sel_p.name, sel_p.id), Style::default().fg(pri).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("  Default Wall Key : ", Style::default().fg(sec).add_modifier(Modifier::BOLD)), Span::raw(sel_p.wall_key)]),
        Line::from(vec![Span::styled("  Day / Night Mode : ", Style::default().fg(sec).add_modifier(Modifier::BOLD)), Span::raw(format!("{}  ([m] toggle)", st.mode))]),
        Line::from(vec![Span::styled("  Corner Geometry  : ", Style::default().fg(sec).add_modifier(Modifier::BOLD)), Span::raw(format!("{}  ([c] toggle)", st.corners))]),
        Line::from(vec![Span::styled("  Warm Nightlight  : ", Style::default().fg(sec).add_modifier(Modifier::BOLD)), Span::raw(format!("{}  ([l] toggle)", st.nightlight))]),
        Line::from(vec![Span::styled("  Active Wallpaper : ", Style::default().fg(sec).add_modifier(Modifier::BOLD)), Span::raw(format!("{} ({} in mode dir, [w] rotate)", st.current_wall.rsplit('/').next().unwrap_or("default"), walls.len()))]),
        Line::from(vec![Span::styled("  16-Bit SFX Synth : ", Style::default().fg(sec).add_modifier(Modifier::BOLD)), Span::raw(format!("{}  ([s] toggle, [1-6] play chime)", if st.sfx_enabled { "ENABLED" } else { "MUTED" }))]),
        Line::from(""),
        Line::from(vec![Span::raw("  Dark Accent/Hot  : "), Span::styled(format!("#{}  #{}", sel_p.d_acc, sel_p.d_hot), Style::default().fg(Color::Rgb(sel_p.pri_rgb.0, sel_p.pri_rgb.1, sel_p.pri_rgb.2)))]),
        Line::from(vec![Span::raw("  Light Accent/Hot : "), Span::styled(format!("#{}  #{}", sel_p.l_acc, sel_p.l_hot), Style::default().fg(Color::Rgb(sel_p.sec_rgb.0, sel_p.sec_rgb.1, sel_p.sec_rgb.2)))]),
    ];
    let blk = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(sec))
        .title(Span::styled(" THEME, WALLPAPER & SFX CONTROLS ", Style::default().fg(sec).add_modifier(Modifier::BOLD)));
    f.render_widget(Paragraph::new(lines).block(blk), area);
}
