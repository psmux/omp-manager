// ─── Entry Point & Event Loop ────────────────────────────────────────────────
//
// Sets up the terminal, initialises the App from detection, runs the main
// event loop, and dispatches keyboard / mouse input.

mod app;
mod config;
mod detect;
mod fonts;
mod install;
mod preview;
mod segments;
mod shell;
mod theme;
mod themes;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::*;

// ── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Detect system
    let report = detect::detect_all();
    let shells = shell::detect_all_shells();

    // Initialise app
    let mut app = App::new(report, shells);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

// ── Event loop ───────────────────────────────────────────────────────────────

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    let mut needs_redraw = true;

    loop {
        // Draw only when state has changed (prevents flickering from constant redraws)
        if needs_redraw {
            terminal.draw(|f| ui::draw(f, app))?;
            needs_redraw = false;
        }

        // Poll events (~20 fps)
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    // Only handle key press events (not release/repeat on Windows)
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    if app.confirm.is_some() {
                        handle_confirm_input(app, key.code);
                    } else if app.theme_search_editing {
                        handle_search_input(app, key.code);
                    } else {
                        handle_normal_input(app, key.code, key.modifiers);
                    }
                    needs_redraw = true;
                }
                Event::Mouse(mouse) => {
                    handle_mouse(app, mouse.kind, mouse.column, mouse.row);
                    needs_redraw = true;
                }
                Event::Resize(_, _) => {
                    needs_redraw = true;
                }
                _ => {}
            }
        }

        // Tick status message countdown
        if app.status.tick > 0 {
            app.tick_status();
            needs_redraw = true;
        }

        // Apply any completed background theme downloads
        let prev_downloading = app.theme_downloading.len();
        app.poll_theme_downloads();
        if app.theme_downloading.len() != prev_downloading {
            needs_redraw = true;
        }

        if !app.running {
            return Ok(());
        }
    }
}

// ── Confirm dialog input ─────────────────────────────────────────────────────

fn handle_confirm_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            if let Some(ref mut d) = app.confirm {
                d.confirm_selected = !d.confirm_selected;
            }
        }
        KeyCode::Enter => {
            if let Some(dialog) = app.confirm.take() {
                if dialog.confirm_selected {
                    execute_confirm_action(app, dialog);
                }
            }
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(dialog) = app.confirm.take() {
                execute_confirm_action(app, dialog);
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => { app.confirm = None; }
        _ => {}
    }
}

fn execute_confirm_action(app: &mut App, dialog: ConfirmDialog) {
    match dialog.action {
        ConfirmAction::InstallOmp => {
            let result = install::install_omp();
            if result.success {
                app.set_status(&result.message);
                // Re-detect everything
                let report = detect::detect_all();
                app.omp = report.omp;
                if let Some(step) = app.setup_steps.get_mut(0) {
                    step.status = install::StepStatus::Done;
                }
                // Refresh theme list now that OMP themes dir is available
                app.refresh_themes();
                // Auto-advance to next setup step
                if app.tab == Tab::Setup && app.setup_current == 0 {
                    app.setup_current = 1;
                }
            } else {
                app.set_status_err(&result.message);
            }
        }
        ConfirmAction::InstallFont => {
            let font_name = &dialog.context;
            let result = fonts::install_font(font_name);
            if result.success {
                app.set_status(&result.message);
                app.fonts = detect::detect_fonts();
                if let Some(step) = app.setup_steps.get_mut(1) {
                    step.status = install::StepStatus::Done;
                }
                // Auto-advance to next setup step
                if app.tab == Tab::Setup && app.setup_current == 1 {
                    app.setup_current = 2;
                }
            } else {
                app.set_status_err(&result.message);
            }
        }
        ConfirmAction::ApplyTheme => {
            apply_theme(app, &dialog.context);
            // Auto-advance to next setup step
            if app.tab == Tab::Setup && app.setup_current == 2 {
                if app.active_theme_name.is_some() {
                    app.setup_current = 3;
                }
            }
        }
        ConfirmAction::ConfigureShell => {
            let idx: usize = dialog.context.parse().unwrap_or(0);
            if let Some(si) = app.shells.get(idx) {
                let theme_path = app.active_theme_name.as_ref().and_then(|name| {
                    app.omp.as_ref()
                        .and_then(|o| o.themes_path.as_ref())
                        .map(|tp| tp.join(format!("{name}.omp.json")).to_string_lossy().to_string())
                });
                let result = install::configure_shell(si, theme_path.as_deref());
                if result.success {
                    app.set_status(&result.message);
                    app.shells = shell::detect_all_shells();
                } else {
                    app.set_status_err(&result.message);
                }
            }
        }
        ConfirmAction::UnconfigureShell => {
            let idx: usize = dialog.context.parse().unwrap_or(0);
            if let Some(si) = app.shells.get(idx) {
                let result = install::unconfigure_shell(si);
                if result.success {
                    app.set_status(&result.message);
                    app.shells = shell::detect_all_shells();
                } else {
                    app.set_status_err(&result.message);
                }
            }
        }
        ConfirmAction::ConfigureAllShells => {
            configure_all_shells(app);
        }
        ConfirmAction::ResetToDefault => {
            reset_to_default(app);
        }
    }
}

// ── Search input ─────────────────────────────────────────────────────────────

fn handle_search_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Enter => {
            app.theme_search_editing = false;
        }
        KeyCode::Backspace => {
            app.theme_search.pop();
            app.apply_theme_filter();
        }
        KeyCode::Char(c) => {
            app.theme_search.push(c);
            app.apply_theme_filter();
        }
        _ => {}
    }
}

// ── Normal input ─────────────────────────────────────────────────────────────

fn handle_normal_input(app: &mut App, key: KeyCode, _mods: KeyModifiers) {
    // Global keys
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => { app.running = false; return; }
        KeyCode::Esc => {
            if app.theme_search_editing { app.theme_search_editing = false; return; }
        }
        KeyCode::Tab => { app.next_tab(); if app.tab == Tab::Themes { app.load_selected_theme_config(); } return; }
        KeyCode::BackTab => { app.prev_tab(); if app.tab == Tab::Themes { app.load_selected_theme_config(); } return; }
        KeyCode::Char('1') => { app.tab = Tab::Dashboard; return; }
        KeyCode::Char('2') => { app.tab = Tab::Setup; return; }
        KeyCode::Char('3') => { app.tab = Tab::Themes; app.load_selected_theme_config(); return; }
        _ => {}
    }

    // Tab-specific keys
    match app.tab {
        Tab::Dashboard => handle_dashboard_input(app, key),
        Tab::Setup     => handle_setup_input(app, key),
        Tab::Themes    => handle_themes_input(app, key),
    }
}

// ── Dashboard input ──────────────────────────────────────────────────────────

fn handle_dashboard_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up => {
            if app.dashboard_selected > 0 { app.dashboard_selected -= 1; }
        }
        KeyCode::Down => {
            if app.dashboard_selected < DashboardItem::ALL.len().saturating_sub(1) {
                app.dashboard_selected += 1;
            }
        }
        KeyCode::Enter => {
            match DashboardItem::ALL.get(app.dashboard_selected) {
                Some(DashboardItem::QuickSetup)   => { app.tab = Tab::Setup; }
                Some(DashboardItem::BrowseThemes) => { app.tab = Tab::Themes; app.load_selected_theme_config(); }
                Some(DashboardItem::InstallFont)  => { app.tab = Tab::Setup; app.setup_current = 1; }
                Some(DashboardItem::UpdateOmp) => {
                    if app.omp.is_some() {
                        let result = install::update_omp();
                        if result.success {
                            app.set_status(&result.message);
                            let report = detect::detect_all();
                            app.omp = report.omp;
                        } else {
                            app.set_status_err(&result.message);
                        }
                    } else {
                        app.set_status_err("Install Oh My Posh first (use Quick Setup)");
                    }
                }
                Some(DashboardItem::ResetDefault) => {
                    app.confirm = Some(ConfirmDialog {
                        title: "Reset to Default".into(),
                        message: "Remove Oh My Posh from ALL shell profiles and clear active theme?\n\nYour shells will return to their default prompt.".into(),
                        action: ConfirmAction::ResetToDefault,
                        confirm_selected: false,
                        context: String::new(),
                    });
                }
                None => {}
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => { app.tab = Tab::Setup; }
        KeyCode::Char('t') | KeyCode::Char('T') => { app.tab = Tab::Themes; app.load_selected_theme_config(); }
        KeyCode::Char('f') | KeyCode::Char('F') => { app.tab = Tab::Setup; app.setup_current = 1; }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            if app.omp.is_some() {
                let result = install::update_omp();
                if result.success {
                    app.set_status(&result.message);
                    let report = detect::detect_all();
                    app.omp = report.omp;
                } else {
                    app.set_status_err(&result.message);
                }
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.confirm = Some(ConfirmDialog {
                title: "Reset to Default".into(),
                message: "Remove Oh My Posh from ALL shell profiles and clear active theme?\n\nYour shells will return to their default prompt.".into(),
                action: ConfirmAction::ResetToDefault,
                confirm_selected: false,
                context: String::new(),
            });
        }
        _ => {}
    }
}

// ── Setup input ──────────────────────────────────────────────────────────────

fn handle_setup_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Left => {
            app.setup_focus = SetupFocus::Steps;
        }
        KeyCode::Right => {
            app.setup_focus = SetupFocus::Detail;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            match app.setup_focus {
                SetupFocus::Steps => {
                    if app.setup_current > 0 { app.setup_current -= 1; }
                }
                SetupFocus::Detail => match app.setup_current {
                    1 => { if app.setup_font_selected > 0 { app.setup_font_selected -= 1; } }
                    2 => { if app.setup_theme_selected > 0 { app.setup_theme_selected -= 1; } }
                    3 => {
                        let avail: Vec<usize> = app.shells.iter().enumerate()
                            .filter(|(_, s)| s.available)
                            .map(|(i, _)| i)
                            .collect();
                        if let Some(pos) = avail.iter().position(|&i| i == app.setup_shell_selected) {
                            if pos > 0 { app.setup_shell_selected = avail[pos - 1]; }
                        }
                    }
                    _ => {}
                },
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            match app.setup_focus {
                SetupFocus::Steps => {
                    if app.setup_current < app.setup_steps.len().saturating_sub(1) {
                        app.setup_current += 1;
                    }
                }
                SetupFocus::Detail => match app.setup_current {
                    1 => {
                        let max = fonts::FONT_CATALOG.len().min(8).saturating_sub(1);
                        if app.setup_font_selected < max { app.setup_font_selected += 1; }
                    }
                    2 => {
                        if app.setup_theme_selected < 7 { app.setup_theme_selected += 1; }
                    }
                    3 => {
                        let avail: Vec<usize> = app.shells.iter().enumerate()
                            .filter(|(_, s)| s.available)
                            .map(|(i, _)| i)
                            .collect();
                        if let Some(pos) = avail.iter().position(|&i| i == app.setup_shell_selected) {
                            if pos < avail.len().saturating_sub(1) {
                                app.setup_shell_selected = avail[pos + 1];
                            }
                        } else if let Some(&first) = avail.first() {
                            app.setup_shell_selected = first;
                        }
                    }
                    _ => {}
                },
            }
        }
        KeyCode::Char(' ') => {
            // Toggle individual shell in step 3
            if app.setup_current == 3 {
                let idx = app.setup_shell_selected;
                if app.shells.get(idx).map(|s| s.available && !s.omp_configured).unwrap_or(false) {
                    if let Some(t) = app.setup_shell_toggles.get_mut(idx) {
                        *t = !*t;
                    }
                }
            }
        }
        KeyCode::Enter => {
            execute_setup_step(app);
        }
        _ => {}
    }
}

fn execute_setup_step(app: &mut App) {
    match app.setup_current {
        0 => {
            // Install OMP
            if app.omp.is_none() {
                app.confirm = Some(ConfirmDialog {
                    title: "Install Oh My Posh".into(),
                    message: "This will download and install Oh My Posh on your system.".into(),
                    action: ConfirmAction::InstallOmp,
                    confirm_selected: true,
                    context: String::new(),
                });
            } else {
                app.set_status("Oh My Posh is already installed!");
                if app.setup_current < app.setup_steps.len() - 1 {
                    app.setup_current += 1;
                }
            }
        }
        1 => {
            // Install font
            if let Some(font) = fonts::FONT_CATALOG.get(app.setup_font_selected) {
                app.confirm = Some(ConfirmDialog {
                    title: "Install Nerd Font".into(),
                    message: format!("Install {} on your system?", font.display),
                    action: ConfirmAction::InstallFont,
                    confirm_selected: true,
                    context: font.name.to_string(),
                });
            }
        }
        2 => {
            // Apply a quick theme
            let quick_themes = ["paradox", "catppuccin_mocha", "dracula", "powerlevel10k_rainbow", "agnoster", "pure", "spaceship", "tokyonight_storm"];
            if let Some(name) = quick_themes.get(app.setup_theme_selected) {
                app.confirm = Some(ConfirmDialog {
                    title: "Apply Theme".into(),
                    message: format!("Set '{name}' as your active prompt theme?"),
                    action: ConfirmAction::ApplyTheme,
                    confirm_selected: true,
                    context: name.to_string(),
                });
            }
        }
        3 => {
            // Configure all selected shells
            app.confirm = Some(ConfirmDialog {
                title: "Configure Shells".into(),
                message: "Add Oh My Posh to the selected shells' profiles?".into(),
                action: ConfirmAction::ConfigureAllShells,
                confirm_selected: true,
                context: String::new(),
            });
        }
        _ => {}
    }
}

// ── Themes input ─────────────────────────────────────────────────────────────

fn handle_themes_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('/') => {
            app.theme_search_editing = true;
        }
        KeyCode::Up => {
            match app.focus {
                Focus::Sidebar => {
                    if app.theme_category_index > 0 {
                        app.theme_category_index -= 1;
                        app.theme_category = themes::ThemeCategory::ALL[app.theme_category_index];
                        app.apply_theme_filter();
                    }
                }
                Focus::List => {
                    if app.theme_selected > 0 {
                        app.theme_selected -= 1;
                        app.load_selected_theme_config();
                    }
                    ensure_theme_scroll(app);
                }
                _ => {}
            }
        }
        KeyCode::Down => {
            match app.focus {
                Focus::Sidebar => {
                    if app.theme_category_index < themes::ThemeCategory::ALL.len() - 1 {
                        app.theme_category_index += 1;
                        app.theme_category = themes::ThemeCategory::ALL[app.theme_category_index];
                        app.apply_theme_filter();
                    }
                }
                Focus::List => {
                    if app.theme_selected < app.theme_filtered.len().saturating_sub(1) {
                        app.theme_selected += 1;
                        app.load_selected_theme_config();
                    }
                    ensure_theme_scroll(app);
                }
                _ => {}
            }
        }
        KeyCode::Left => {
            app.focus = Focus::Sidebar;
        }
        KeyCode::Right => {
            if app.focus == Focus::Sidebar {
                app.focus = Focus::List;
            } else {
                app.focus = Focus::Detail;
            }
        }
        KeyCode::Enter => {
            if let Some(theme) = app.selected_theme() {
                let name = theme.name.clone();
                app.confirm = Some(ConfirmDialog {
                    title: "Apply Theme".into(),
                    message: format!("Set '{name}' as your active prompt theme?"),
                    action: ConfirmAction::ApplyTheme,
                    confirm_selected: true,
                    context: name,
                });
            }
        }
        _ => {}
    }
}

fn ensure_theme_scroll(app: &mut App) {
    // 2 lines per item in the new rendering
    let visible_items = app.theme_visible_height / 2;
    if visible_items == 0 { return; }
    if app.theme_selected >= app.theme_scroll + visible_items {
        app.theme_scroll = app.theme_selected.saturating_sub(visible_items - 1);
    }
    if app.theme_selected < app.theme_scroll {
        app.theme_scroll = app.theme_selected;
    }
}

// ── Mouse input ──────────────────────────────────────────────────────────────

fn handle_mouse(app: &mut App, kind: MouseEventKind, x: u16, y: u16) {
    match kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // ── Confirmation dialog: handle button clicks ──────────────
            if app.confirm.is_some() {
                if let Some((cx, cy, cw, ch)) = app.layout.confirm_area {
                    let btn_y = cy + ch.saturating_sub(2);
                    if y >= btn_y.saturating_sub(1) && y <= btn_y + 1 {
                        let mid = cx + cw / 2;
                        if let Some(ref mut d) = app.confirm {
                            if x < mid {
                                d.confirm_selected = false;
                            } else {
                                d.confirm_selected = true;
                            }
                        }
                    }
                }
                return;
            }

            // ── Tab bar clicks ─────────────────────────────────────────
            if hit_test(x, y, &app.layout.tab_bar) {
                if let Some((tx, _, _tw, _)) = app.layout.tab_bar {
                    // Calculate actual rendered tab positions instead of equal-width division.
                    // Labels are " Dashboard ", " Setup ", " Themes " with " │ " dividers.
                    let divider_w: u16 = 3;
                    let mut tab_starts: Vec<u16> = Vec::new();
                    let mut cursor = tx;
                    for (i, t) in Tab::ALL.iter().enumerate() {
                        tab_starts.push(cursor);
                        cursor += (t.label().len() as u16) + 2; // " label "
                        if i < Tab::ALL.len() - 1 {
                            cursor += divider_w;
                        }
                    }
                    // Find the rightmost tab whose start position is <= x
                    // (only within the rendered tab region — ignore clicks past all labels)
                    if x < cursor {
                        let mut tab_idx = 0;
                        for (i, &s) in tab_starts.iter().enumerate() {
                            if x >= s { tab_idx = i; }
                        }
                        if let Some(&tab) = Tab::ALL.get(tab_idx) {
                            app.tab = tab;
                            if tab == Tab::Themes { app.load_selected_theme_config(); }
                        }
                    }
                }
                return;
            }

            // ── Per-tab click handling ─────────────────────────────────
            match app.tab {
                Tab::Dashboard => {
                    if hit_test(x, y, &app.layout.dashboard_area) {
                        if let Some((_, dy, _, _)) = app.layout.dashboard_area {
                            let row = (y.saturating_sub(dy)) as usize / 2;
                            if row < DashboardItem::ALL.len() {
                                app.dashboard_selected = row;
                                handle_dashboard_input(app, KeyCode::Enter);
                            }
                        }
                    }
                }
                Tab::Setup => {
                    // Steps panel click
                    if hit_test(x, y, &app.layout.setup_steps) {
                        if let Some((_, sy, _, _)) = app.layout.setup_steps {
                            let row = (y.saturating_sub(sy + 1)) as usize / 2;
                            if row < app.setup_steps.len() {
                                app.setup_current = row;
                                app.setup_focus = SetupFocus::Steps;
                            }
                        }
                    }
                    // Detail panel click
                    else if hit_test(x, y, &app.layout.setup_detail) {
                        app.setup_focus = SetupFocus::Detail;
                        if let Some((_, dy, _, _)) = app.layout.setup_detail {
                            match app.setup_current {
                                1 => {
                                    let row = (y.saturating_sub(dy + 8)) as usize / 2;
                                    let max = fonts::FONT_CATALOG.len().min(8);
                                    if row < max {
                                        app.setup_font_selected = row;
                                    }
                                }
                                2 => {
                                    let row = (y.saturating_sub(dy + 7)) as usize;
                                    if row < 8 {
                                        app.setup_theme_selected = row;
                                    }
                                }
                                3 => {
                                    let row = (y.saturating_sub(dy + 7)) as usize;
                                    let avail: Vec<usize> = app.shells.iter().enumerate()
                                        .filter(|(_, s)| s.available)
                                        .map(|(i, _)| i)
                                        .collect();
                                    if let Some(&idx) = avail.get(row) {
                                        app.setup_shell_selected = idx;
                                        if !app.shells[idx].omp_configured {
                                            if let Some(t) = app.setup_shell_toggles.get_mut(idx) {
                                                *t = !*t;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Tab::Themes => {
                    // Sidebar click
                    if hit_test(x, y, &app.layout.sidebar) {
                        if let Some((_, sy, _, _)) = app.layout.sidebar {
                            let row = (y.saturating_sub(sy + 1)) as usize;
                            if row < themes::ThemeCategory::ALL.len() {
                                app.theme_category_index = row;
                                app.theme_category = themes::ThemeCategory::ALL[row];
                                app.apply_theme_filter();
                                app.focus = Focus::Sidebar;
                            }
                        }
                    }
                    // List click
                    else if hit_test(x, y, &app.layout.list) {
                        if let Some((_, ly, _, _)) = app.layout.list {
                            if y < ly + 3 {
                                // Click on search bar area → activate search
                                app.theme_search_editing = true;
                                app.focus = Focus::List;
                            } else {
                                let row = (y.saturating_sub(ly + 3)) as usize;
                                let idx = app.theme_scroll + row / 2;
                                if idx < app.theme_filtered.len() {
                                    app.theme_selected = idx;
                                    app.load_selected_theme_config();
                                    app.focus = Focus::List;
                                }
                            }
                        }
                    }
                    // Detail panel – action button area (bottom)
                    else if hit_test(x, y, &app.layout.detail) {
                        if let Some((_, dy, _, dh)) = app.layout.detail {
                            let btn_row = dy + dh.saturating_sub(3);
                            if y >= btn_row {
                                if let Some(theme) = app.selected_theme() {
                                    let is_active = app.active_theme_name.as_deref() == Some(&theme.name);
                                    if !is_active {
                                        let name = theme.name.clone();
                                        app.confirm = Some(ConfirmDialog {
                                            title: "Apply Theme".into(),
                                            message: format!("Set '{name}' as your active prompt theme?"),
                                            action: ConfirmAction::ApplyTheme,
                                            confirm_selected: true,
                                            context: name,
                                        });
                                    }
                                }
                            }
                            app.focus = Focus::Detail;
                        }
                    }
                }
            }
        }

        MouseEventKind::Down(MouseButton::Right) => {
            // Right-click in themes list → apply
            if app.tab == Tab::Themes && hit_test(x, y, &app.layout.list) {
                if let Some(theme) = app.selected_theme() {
                    let is_active = app.active_theme_name.as_deref() == Some(&theme.name);
                    if !is_active {
                        let name = theme.name.clone();
                        app.confirm = Some(ConfirmDialog {
                            title: "Apply Theme".into(),
                            message: format!("Set '{name}' as your active prompt theme?"),
                            action: ConfirmAction::ApplyTheme,
                            confirm_selected: true,
                            context: name,
                        });
                    }
                }
            }
        }

        MouseEventKind::ScrollUp => {
            if app.confirm.is_some() { return; }
            match app.tab {
                Tab::Dashboard => {
                    if app.dashboard_selected > 0 { app.dashboard_selected -= 1; }
                }
                Tab::Setup => {
                    if hit_test(x, y, &app.layout.setup_steps) {
                        if app.setup_current > 0 { app.setup_current -= 1; }
                    } else {
                        match app.setup_current {
                            1 => { if app.setup_font_selected > 0 { app.setup_font_selected -= 1; } }
                            2 => { if app.setup_theme_selected > 0 { app.setup_theme_selected -= 1; } }
                            3 => {
                                let avail: Vec<usize> = app.shells.iter().enumerate()
                                    .filter(|(_, s)| s.available)
                                    .map(|(i, _)| i)
                                    .collect();
                                if let Some(pos) = avail.iter().position(|&i| i == app.setup_shell_selected) {
                                    if pos > 0 { app.setup_shell_selected = avail[pos - 1]; }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Tab::Themes => {
                    if hit_test(x, y, &app.layout.sidebar) {
                        if app.theme_category_index > 0 {
                            app.theme_category_index -= 1;
                            app.theme_category = themes::ThemeCategory::ALL[app.theme_category_index];
                            app.apply_theme_filter();
                        }
                    } else if hit_test(x, y, &app.layout.list) || hit_test(x, y, &app.layout.detail) {
                        if app.theme_selected > 0 {
                            app.theme_selected -= 1;
                            app.load_selected_theme_config();
                            ensure_theme_scroll(app);
                        }
                    }
                }
            }
        }

        MouseEventKind::ScrollDown => {
            if app.confirm.is_some() { return; }
            match app.tab {
                Tab::Dashboard => {
                    if app.dashboard_selected < DashboardItem::ALL.len().saturating_sub(1) {
                        app.dashboard_selected += 1;
                    }
                }
                Tab::Setup => {
                    if hit_test(x, y, &app.layout.setup_steps) {
                        if app.setup_current < app.setup_steps.len().saturating_sub(1) {
                            app.setup_current += 1;
                        }
                    } else {
                        match app.setup_current {
                            1 => {
                                let max = fonts::FONT_CATALOG.len().min(8).saturating_sub(1);
                                if app.setup_font_selected < max { app.setup_font_selected += 1; }
                            }
                            2 => { if app.setup_theme_selected < 7 { app.setup_theme_selected += 1; } }
                            3 => {
                                let avail: Vec<usize> = app.shells.iter().enumerate()
                                    .filter(|(_, s)| s.available)
                                    .map(|(i, _)| i)
                                    .collect();
                                if let Some(pos) = avail.iter().position(|&i| i == app.setup_shell_selected) {
                                    if pos < avail.len().saturating_sub(1) {
                                        app.setup_shell_selected = avail[pos + 1];
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Tab::Themes => {
                    if hit_test(x, y, &app.layout.sidebar) {
                        if app.theme_category_index < themes::ThemeCategory::ALL.len().saturating_sub(1) {
                            app.theme_category_index += 1;
                            app.theme_category = themes::ThemeCategory::ALL[app.theme_category_index];
                            app.apply_theme_filter();
                        }
                    } else if hit_test(x, y, &app.layout.list) || hit_test(x, y, &app.layout.detail) {
                        if app.theme_selected < app.theme_filtered.len().saturating_sub(1) {
                            app.theme_selected += 1;
                            app.load_selected_theme_config();
                            ensure_theme_scroll(app);
                        }
                    }
                }
            }
        }

        _ => {}
    }
}

fn hit_test(x: u16, y: u16, region: &Option<(u16, u16, u16, u16)>) -> bool {
    if let Some((rx, ry, rw, rh)) = region {
        x >= *rx && x < rx + rw && y >= *ry && y < ry + rh
    } else {
        false
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn apply_theme(app: &mut App, theme_name: &str) {
    let themes_dir = app.omp.as_ref().and_then(|o| o.themes_path.as_deref());
    let entry = app.theme_list.iter().find(|t| t.name == theme_name);
    let path = entry.and_then(|e| themes::get_theme_file_path(e, themes_dir));

    if let Some(ref path) = path {
        // Update the theme entry's cached path so future lookups don't re-download
        if let Some(entry) = app.theme_list.iter_mut().find(|t| t.name == theme_name) {
            if entry.path.is_none() {
                entry.path = Some(path.clone());
            }
        }

        let path_str = path.to_string_lossy().to_string();
        // Update all configured shells
        let mut success_count = 0;
        let shell_list = app.shells.clone();
        for si in &shell_list {
            if si.omp_configured {
                let result = install::configure_shell(si, Some(&path_str));
                if result.success { success_count += 1; }
            }
        }
        app.active_theme_name = Some(theme_name.to_string());
        if success_count > 0 {
            app.set_status(&format!("Theme '{theme_name}' applied to {success_count} shell(s) — restart shells to see changes"));
        } else {
            app.set_status(&format!("Theme '{theme_name}' selected — configure shells in Setup tab to activate"));
        }
        // Mark setup step
        if let Some(step) = app.setup_steps.get_mut(2) {
            step.status = install::StepStatus::Done;
        }
        // Refresh shells
        app.shells = shell::detect_all_shells();
    } else {
        app.set_status_err(&format!("Could not find or download theme '{theme_name}' — check your internet connection"));
    }
}

fn configure_all_shells(app: &mut App) {
    // Find the theme file path — check OMP themes dir and our cache
    let theme_path = app.active_theme_name.as_ref().and_then(|name| {
        let themes_dir = app.omp.as_ref().and_then(|o| o.themes_path.as_deref());
        // Check if theme entry has a cached path
        let from_entry = app.theme_list.iter()
            .find(|t| t.name == *name)
            .and_then(|t| t.path.as_ref())
            .map(|p| p.to_string_lossy().to_string());
        if from_entry.is_some() { return from_entry; }
        // Fallback: construct from themes_dir
        themes_dir.map(|tp| tp.join(format!("{name}.omp.json")).to_string_lossy().to_string())
    });
    let mut configured = 0;
    let shell_list = app.shells.clone();
    for (i, si) in shell_list.iter().enumerate() {
        let toggled = app.setup_shell_toggles.get(i).copied().unwrap_or(false);
        if si.available && toggled && !si.omp_configured {
            let result = install::configure_shell(si, theme_path.as_deref());
            if result.success { configured += 1; }
        }
    }
    if configured > 0 {
        app.set_status(&format!(
            "✓ Setup complete! Configured {configured} shell(s) — restart them to see your new prompt"
        ));
        if let Some(step) = app.setup_steps.get_mut(3) {
            step.status = install::StepStatus::Done;
        }
    } else {
        app.set_status("No shells were toggled for configuration");
    }
    app.shells = shell::detect_all_shells();
}
fn reset_to_default(app: &mut App) {
    let shell_list = app.shells.clone();
    let mut removed = 0;

    for si in &shell_list {
        if si.omp_configured {
            let result = install::unconfigure_shell(si);
            if result.success { removed += 1; }
        }
    }

    // Clear active theme
    app.active_theme_name = None;

    // Reset setup step statuses for theme & shells
    if let Some(step) = app.setup_steps.get_mut(2) {
        step.status = install::StepStatus::Pending;
    }
    if let Some(step) = app.setup_steps.get_mut(3) {
        step.status = install::StepStatus::Pending;
    }

    // Refresh shells
    app.shells = shell::detect_all_shells();

    if removed > 0 {
        app.set_status(&format!(
            "Reset complete — removed Oh My Posh from {removed} shell(s). Restart your shells to see the default prompt."
        ));
    } else {
        app.set_status("No configured shells found — already at default");
    }
}