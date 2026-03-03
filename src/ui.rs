// ─── UI Rendering ────────────────────────────────────────────────────────────
//
// Pure rendering layer.  Reads `App` state, writes ratatui widgets.
// Also records layout regions for mouse hit-testing.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block as UiBlock, BorderType, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::app::*;
use crate::install::StepStatus;
use crate::theme::*;

// ── Scroll helper (matches TPP pattern) ──────────────────────────────────────

/// Ensure scroll_offset keeps the selected index visible.
/// `lines_per_item` is how many terminal rows each list row occupies.
fn ensure_scroll_visible(
    selected: usize,
    scroll_offset: &mut usize,
    visible_height: usize,
    lines_per_item: usize,
) {
    let items_visible = if lines_per_item > 0 {
        visible_height / lines_per_item
    } else {
        visible_height
    };
    if items_visible == 0 { return; }
    if selected >= *scroll_offset + items_visible {
        *scroll_offset = selected.saturating_sub(items_visible - 1);
    }
    if selected < *scroll_offset {
        *scroll_offset = selected;
    }
}

// ── Main entry point ─────────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    f.render_widget(UiBlock::default().style(Style::default().bg(BG)), area);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),   // header
            Constraint::Length(1),   // divider
            Constraint::Length(3),   // tabs
            Constraint::Min(10),     // body
            Constraint::Length(1),   // status
            Constraint::Length(1),   // footer
        ])
        .split(area);

    draw_header(f, outer[0], app);
    draw_divider(f, outer[1]);
    draw_tabs(f, outer[2], app);
    draw_body(f, outer[3], app);
    draw_status(f, outer[4], app);
    draw_footer(f, outer[5], app);

    // Confirm overlay
    if app.confirm.is_some() {
        draw_confirm(f, area, app);
    }
}

// ── Header ───────────────────────────────────────────────────────────────────

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let mut spans = vec![
        Span::styled(" ", Style::default().bg(BG)),
        Span::styled(" OMP ", Style::default().fg(BG).bg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("  Oh My Posh Manager", Style::default().fg(TEXT_DIM)),
    ];

    // Show OMP version on the right if installed
    if let Some(ref omp) = app.omp {
        let version_info = format!("v{}", omp.version);
        let used = 28usize;
        let pad = (area.width as usize).saturating_sub(used + version_info.len() + 2);
        spans.push(Span::styled(" ".repeat(pad), Style::default().bg(BG)));
        spans.push(Span::styled(version_info, Style::default().fg(ACCENT2)));
    }

    let header = Paragraph::new(vec![
        Line::from(""),
        Line::from(spans),
    ]).style(Style::default().bg(BG));
    f.render_widget(header, area);
}

// ── Divider ──────────────────────────────────────────────────────────────────

fn draw_divider(f: &mut Frame, area: Rect) {
    let divider = "─".repeat(area.width as usize);
    f.render_widget(
        Paragraph::new(Span::styled(divider, Style::default().fg(TEXT_DARK)))
            .style(Style::default().bg(BG)),
        area,
    );
}

// ── Tab bar ──────────────────────────────────────────────────────────────────

fn draw_tabs(f: &mut Frame, area: Rect, app: &mut App) {
    app.layout.tab_bar = Some((area.x, area.y, area.width, area.height));

    let tab_titles: Vec<Line> = Tab::ALL.iter().map(|t| {
        let style = if *t == app.tab {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT_DIM)
        };
        Line::from(Span::styled(format!(" {} ", t.label()), style))
    }).collect();

    let tabs = Tabs::new(tab_titles)
        .select(app.tab.index())
        .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        .divider(Span::styled(" │ ", Style::default().fg(TEXT_DARK)))
        .block(
            UiBlock::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(TEXT_DARK))
                .style(Style::default().bg(BG)),
        );

    f.render_widget(tabs, area);
}

// ── Body dispatch ────────────────────────────────────────────────────────────

fn draw_body(f: &mut Frame, area: Rect, app: &mut App) {
    match app.tab {
        Tab::Dashboard => draw_dashboard(f, area, app),
        Tab::Setup     => draw_setup(f, area, app),
        Tab::Themes    => draw_themes(f, area, app),
    }
}

// ── Dashboard ────────────────────────────────────────────────────────────────

fn draw_dashboard(f: &mut Frame, area: Rect, app: &mut App) {
    let outer = UiBlock::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(BG));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(40),
            Constraint::Length(2),
        ])
        .split(inner);

    let center = cols[1];

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),   // welcome banner
            Constraint::Length(1),   // spacer
            Constraint::Min(12),     // action cards
            Constraint::Length(1),   // spacer
            Constraint::Length(8),   // system info
            Constraint::Length(3),   // quick reference
        ])
        .split(center);

    // ── Welcome banner ────────────────────────────────
    let welcome_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Welcome to ", Style::default().fg(TEXT)),
            Span::styled("Oh My Posh Manager", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(" - your prompt customization tool", Style::default().fg(TEXT)),
        ]),
        Line::from(Span::styled(
            "  Install Oh My Posh, pick themes, configure fonts, and set up all your shells.",
            Style::default().fg(TEXT_DIM),
        )),
    ];
    f.render_widget(Paragraph::new(welcome_lines).style(Style::default().bg(BG)), rows[0]);

    // ── Action cards (TPP-style 2-line items with pointer) ────
    let cards_area = rows[2];
    app.layout.dashboard_area = Some((cards_area.x, cards_area.y + 1, cards_area.width, cards_area.height));

    let items: Vec<ListItem> = DashboardItem::ALL.iter().enumerate().map(|(i, item)| {
        let is_sel = i == app.dashboard_selected;
        let bg = if is_sel { BG_HIGHLIGHT } else { BG_PANEL };

        let icon_style = if is_sel {
            Style::default().fg(ACCENT2).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(ACCENT)
        };
        let label_style = if is_sel {
            Style::default().fg(ACCENT2).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
        };
        let desc_style = Style::default().fg(TEXT_DIM);

        let pointer = if is_sel { "▶ " } else { "  " };
        let pointer_style = if is_sel {
            Style::default().fg(ACCENT2).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT_DARK)
        };

        let key_hint = item.key_hint();

        let line1 = Line::from(vec![
            Span::styled(pointer, pointer_style),
            Span::styled(format!("{}  ", item.icon()), icon_style),
            Span::styled(item.label(), label_style),
            Span::styled(format!("  [{}]", key_hint), Style::default().fg(TEXT_DARK)),
        ]);
        let line2 = Line::from(vec![
            Span::styled("     ", Style::default()),
            Span::styled(item.description(), desc_style),
        ]);

        ListItem::new(vec![line1, line2]).style(Style::default().bg(bg))
    }).collect();

    let cards = List::new(items).block(
        UiBlock::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(TEXT_DARK))
            .title(Span::styled(" Quick Actions ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(BG)),
    );
    f.render_widget(cards, cards_area);

    // ── System info ───────────────────────────────────
    let mut info_lines: Vec<Line> = Vec::new();

    // Oh My Posh status
    if let Some(ref omp) = app.omp {
        info_lines.push(Line::from(vec![
            Span::styled("  Oh My Posh: ", Style::default().fg(TEXT_DIM)),
            Span::styled(format!("v{}", omp.version), Style::default().fg(GREEN)),
        ]));
    } else {
        info_lines.push(Line::from(vec![
            Span::styled("  Oh My Posh: ", Style::default().fg(TEXT_DIM)),
            Span::styled("Not installed - use Quick Setup", Style::default().fg(YELLOW)),
        ]));
    }

    // Font status
    if app.fonts.has_nerd_font {
        let font_name = app.fonts.installed_fonts.first().map(|s| s.as_str()).unwrap_or("detected");
        info_lines.push(Line::from(vec![
            Span::styled("  Nerd Font: ", Style::default().fg(TEXT_DIM)),
            Span::styled(font_name, Style::default().fg(GREEN)),
        ]));
    } else {
        info_lines.push(Line::from(vec![
            Span::styled("  Nerd Font: ", Style::default().fg(TEXT_DIM)),
            Span::styled("Not detected - install one in Setup", Style::default().fg(YELLOW)),
        ]));
    }

    // Active theme
    if let Some(ref theme) = app.active_theme_name {
        info_lines.push(Line::from(vec![
            Span::styled("  Theme: ", Style::default().fg(TEXT_DIM)),
            Span::styled(theme.as_str(), Style::default().fg(ACCENT2)),
        ]));
    }

    // OS
    info_lines.push(Line::from(vec![
        Span::styled("  System: ", Style::default().fg(TEXT_DIM)),
        Span::styled(format!("{} {}", app.os_info.name, app.os_info.version), Style::default().fg(TEXT)),
    ]));

    // Shell statuses
    let configured: Vec<_> = app.shells.iter().filter(|s| s.available && s.omp_configured).collect();
    let unconfigured: Vec<_> = app.shells.iter().filter(|s| s.available && !s.omp_configured).collect();
    if !configured.is_empty() {
        let names: String = configured.iter().map(|s| s.shell.label()).collect::<Vec<_>>().join(", ");
        info_lines.push(Line::from(vec![
            Span::styled("  Configured: ", Style::default().fg(TEXT_DIM)),
            Span::styled(names, Style::default().fg(GREEN)),
        ]));
    }
    if !unconfigured.is_empty() {
        let names: String = unconfigured.iter().map(|s| s.shell.label()).collect::<Vec<_>>().join(", ");
        info_lines.push(Line::from(vec![
            Span::styled("  Available: ", Style::default().fg(TEXT_DIM)),
            Span::styled(names, Style::default().fg(TEXT_DIM)),
        ]));
    }

    let info_block = Paragraph::new(info_lines)
        .block(
            UiBlock::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(TEXT_DARK))
                .title(Span::styled(" System Info ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
                .style(Style::default().bg(BG)),
        );
    f.render_widget(info_block, rows[4]);

    // ── Quick reference ───────────────────────────────
    let quick_ref = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑↓", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(" Navigate   ", Style::default().fg(TEXT_DIM)),
            Span::styled("Enter", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(" Select   ", Style::default().fg(TEXT_DIM)),
            Span::styled("Tab", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(" Switch Tab   ", Style::default().fg(TEXT_DIM)),
            Span::styled("q", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(" Quit", Style::default().fg(TEXT_DIM)),
        ]),
    ]).style(Style::default().bg(BG));
    f.render_widget(quick_ref, rows[5]);
}

// ── Setup tab ────────────────────────────────────────────────────────────────

fn draw_setup(f: &mut Frame, area: Rect, app: &mut App) {
    app.layout.setup_area = Some((area.x, area.y, area.width, area.height));

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(5),
            Constraint::Length(30),
            Constraint::Min(1),
            Constraint::Percentage(5),
        ])
        .split(area);

    // Record layout regions for mouse hit-testing
    app.layout.setup_steps = Some((cols[1].x, cols[1].y, cols[1].width, cols[1].height));
    app.layout.setup_detail = Some((cols[2].x, cols[2].y, cols[2].width, cols[2].height));

    draw_setup_steps(f, cols[1], app);
    draw_setup_detail(f, cols[2], app);
}

fn draw_setup_steps(f: &mut Frame, area: Rect, app: &App) {
    let border_color = if app.setup_focus == SetupFocus::Steps { ACCENT } else { TEXT_DARK };
    let block = UiBlock::default()
        .title(Span::styled(" Setup Wizard ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BG_PANEL));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = app.setup_steps.iter().enumerate().map(|(i, step)| {
        let is_current = i == app.setup_current;
        let icon = step.status.icon();
        let icon_color = match step.status {
            StepStatus::Done       => GREEN,
            StepStatus::Failed     => RED,
            StepStatus::InProgress => ACCENT2,
            StepStatus::Skipped    => TEXT_DIM,
            StepStatus::Pending    => TEXT_DIM,
        };

        let pointer = if is_current { "▶" } else { " " };
        let pointer_style = if is_current {
            Style::default().fg(ACCENT2).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT_DARK)
        };

        let label_style = if is_current {
            Style::default().fg(TEXT).bg(BG_HIGHLIGHT)
        } else {
            Style::default().fg(TEXT_DIM)
        };

        let line1 = Line::from(vec![
            Span::styled(format!("{pointer} "), pointer_style),
            Span::styled(format!("{icon} "), Style::default().fg(icon_color)),
            Span::styled(&step.label, label_style),
        ]);
        let line2 = Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(step.status.label(), Style::default().fg(icon_color)),
        ]);

        let bg = if is_current { BG_HIGHLIGHT } else { BG_PANEL };
        ListItem::new(vec![line1, line2]).style(Style::default().bg(bg))
    }).collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

fn draw_setup_detail(f: &mut Frame, area: Rect, app: &App) {
    let border_color = if app.setup_focus == SetupFocus::Detail { ACCENT } else { TEXT_DARK };
    let block = UiBlock::default()
        .title(Span::styled(" Details ", Style::default().fg(ACCENT2).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BG_PANEL));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let step = match app.setup_steps.get(app.setup_current) {
        Some(s) => s,
        None => return,
    };

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!("Step {} of {}", app.setup_current + 1, app.setup_steps.len()),
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(&step.label, Style::default().fg(TEXT).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(Span::styled(&step.description, Style::default().fg(TEXT_DIM))));
    lines.push(Line::from(""));

    match app.setup_current {
        0 => {
            if step.status == StepStatus::Done {
                if let Some(ref omp) = app.omp {
                    lines.push(Line::from(vec![
                        Span::styled(" ✓ ", Style::default().fg(GREEN)),
                        Span::styled(format!("Installed - v{}", omp.version), Style::default().fg(TEXT)),
                    ]));
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "  Press ↓ to continue to next step",
                        Style::default().fg(TEXT_DIM),
                    )));
                }
            } else {
                lines.push(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(" Enter ", Style::default().fg(BG).bg(GREEN).add_modifier(Modifier::BOLD)),
                    Span::styled(" Install Oh My Posh", Style::default().fg(TEXT)),
                ]));
                lines.push(Line::from(""));
                #[cfg(target_os = "windows")]
                lines.push(Line::from(Span::styled("  Method: winget (or PowerShell fallback)", Style::default().fg(TEXT_DIM))));
                #[cfg(target_os = "macos")]
                lines.push(Line::from(Span::styled("  Method: Homebrew", Style::default().fg(TEXT_DIM))));
                #[cfg(target_os = "linux")]
                lines.push(Line::from(Span::styled("  Method: Official install script", Style::default().fg(TEXT_DIM))));
            }
        }
        1 => {
            if step.status == StepStatus::Done {
                lines.push(Line::from(vec![
                    Span::styled(" ✓ ", Style::default().fg(GREEN)),
                    Span::styled("Nerd Font installed", Style::default().fg(TEXT)),
                ]));
            } else {
                lines.push(Line::from(Span::styled(
                    "Select a font with ↑/↓ and press Enter to install:",
                    Style::default().fg(ACCENT2),
                )));
                lines.push(Line::from(""));
                for (i, font) in crate::fonts::FONT_CATALOG.iter().take(8).enumerate() {
                    let is_sel = i == app.setup_font_selected;
                    let pointer = if is_sel { "▶" } else { " " };
                    let rec = if font.recommended { " ★ recommended" } else { "" };
                    let style = if is_sel {
                        Style::default().fg(TEXT).bg(BG_HIGHLIGHT)
                    } else {
                        Style::default().fg(TEXT_DIM)
                    };
                    lines.push(Line::from(vec![
                        Span::styled(format!(" {pointer} "), if is_sel { Style::default().fg(ACCENT2) } else { Style::default().fg(TEXT_DARK) }),
                        Span::styled(format!("{}", font.display), style),
                        Span::styled(rec, Style::default().fg(YELLOW)),
                    ]));
                    lines.push(Line::from(Span::styled(
                        format!("     {}", font.description),
                        Style::default().fg(TEXT_DARK),
                    )));
                }
            }
        }
        2 => {
            lines.push(Line::from(Span::styled(
                "Quick picks - or go to the Themes tab for the full list:",
                Style::default().fg(ACCENT2),
            )));
            lines.push(Line::from(""));
            let quick_themes = ["paradox", "catppuccin_mocha", "dracula", "powerlevel10k_rainbow", "agnoster", "pure", "spaceship", "tokyonight_storm"];
            for (i, name) in quick_themes.iter().enumerate() {
                let is_sel = i == app.setup_theme_selected;
                let pointer = if is_sel { "▶" } else { " " };
                let style = if is_sel {
                    Style::default().fg(TEXT).bg(BG_HIGHLIGHT)
                } else {
                    Style::default().fg(TEXT_DIM)
                };
                lines.push(Line::from(vec![
                    Span::styled(format!(" {pointer} "), if is_sel { Style::default().fg(ACCENT2) } else { Style::default().fg(TEXT_DARK) }),
                    Span::styled(*name, style),
                ]));
            }
        }
        3 => {
            lines.push(Line::from(Span::styled(
                "Select with ↑/↓, Space to toggle, Enter to apply:",
                Style::default().fg(ACCENT2),
            )));
            lines.push(Line::from(""));
            for (i, si) in app.shells.iter().enumerate() {
                if !si.available { continue; }
                let is_sel = i == app.setup_shell_selected && app.setup_focus == SetupFocus::Detail;
                let toggled = app.setup_shell_toggles.get(i).copied().unwrap_or(false);
                let check = if si.omp_configured { "✓" } else if toggled { "☑" } else { "☐" };
                let pointer = if is_sel { "▶" } else { " " };
                let color = if si.omp_configured { GREEN } else if toggled { ACCENT2 } else { TEXT_DIM };
                let bg_style = if is_sel { BG_HIGHLIGHT } else { BG_PANEL };
                let label = if si.omp_configured {
                    format!("{pointer} {check}  {} (already configured)", si.shell.label())
                } else {
                    format!("{pointer} {check}  {}", si.shell.label())
                };
                lines.push(Line::from(Span::styled(label, Style::default().fg(color).bg(bg_style))));
            }
        }
        _ => {}
    }

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

// ── Themes tab ───────────────────────────────────────────────────────────────

fn draw_themes(f: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),    // category sidebar
            Constraint::Percentage(35), // theme list
            Constraint::Min(30),       // preview
        ])
        .split(area);

    app.layout.sidebar = Some((cols[0].x, cols[0].y, cols[0].width, cols[0].height));
    app.layout.list = Some((cols[1].x, cols[1].y, cols[1].width, cols[1].height));
    app.layout.detail = Some((cols[2].x, cols[2].y, cols[2].width, cols[2].height));

    draw_theme_categories(f, cols[0], app);

    // Calculate visible height for scroll, then adjust scroll
    let list_inner_h = cols[1].height.saturating_sub(4) as usize; // borders + search bar
    app.theme_visible_height = list_inner_h;
    ensure_scroll_visible(app.theme_selected, &mut app.theme_scroll, list_inner_h, 2);

    draw_theme_list(f, cols[1], app);
    draw_theme_detail(f, cols[2], app);
}

fn draw_theme_categories(f: &mut Frame, area: Rect, app: &App) {
    let block = UiBlock::default()
        .title(Span::styled(" CATEGORIES ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(TEXT_DARK))
        .style(Style::default().bg(BG_PANEL));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = crate::themes::ThemeCategory::ALL.iter().enumerate().map(|(i, cat)| {
        let is_sel = i == app.theme_category_index;
        let style = if is_sel {
            Style::default().fg(ACCENT).bg(BG_HIGHLIGHT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT_DIM)
        };
        let pointer = if is_sel { "▶" } else { " " };

        ListItem::new(Line::from(vec![
            Span::styled(format!(" {pointer} "), if is_sel { Style::default().fg(ACCENT2) } else { Style::default().fg(TEXT_DARK) }),
            Span::styled(format!("{} ", cat.label()), style),
        ]))
    }).collect();

    f.render_widget(List::new(items).style(Style::default().bg(BG_PANEL)), inner);
}

fn draw_theme_list(f: &mut Frame, area: Rect, app: &App) {
    let block = UiBlock::default()
        .title(Span::styled(
            format!(" Themes ({}) ", app.theme_filtered.len()),
            Style::default().fg(ACCENT),
        ))
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(TEXT_DARK))
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Search bar + list
    let search_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(inner);

    // Search bar with border
    let search_style = if app.theme_search_editing {
        Style::default().fg(ACCENT)
    } else {
        Style::default().fg(TEXT_DIM)
    };
    let search_text = if app.theme_search.is_empty() {
        "  / Search themes...".to_string()
    } else {
        format!("  / {}", app.theme_search)
    };
    let search_bar = Paragraph::new(search_text)
        .style(search_style)
        .block(
            UiBlock::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(TEXT_DARK))
                .style(Style::default().bg(BG_PANEL)),
        );
    f.render_widget(search_bar, search_rows[0]);

    // Theme list (2 lines per item, TPP-style)
    let list_area = search_rows[1];
    let visible_height = list_area.height as usize;

    let items: Vec<ListItem> = app.theme_filtered.iter()
        .skip(app.theme_scroll)
        .take(visible_height / 2) // 2 lines per item
        .enumerate()
        .map(|(vi, &idx)| {
            let theme = &app.theme_list[idx];
            let is_selected = vi + app.theme_scroll == app.theme_selected;
            let is_active = app.active_theme_name.as_deref() == Some(&theme.name);

            let name_style = if is_selected {
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
            };

            let on_disk = if theme.path.is_some() { "" } else { " (remote)" };
            let mut name_spans = vec![
                Span::styled(format!(" {}", theme.name), name_style),
                Span::styled(on_disk, Style::default().fg(TEXT_DARK)),
            ];
            if is_active {
                name_spans.push(Span::styled(" ★", Style::default().fg(YELLOW)));
            }

            let line1 = Line::from(name_spans);

            // Description line (truncated to fit)
            let desc = &theme.description;
            let max_desc = (list_area.width as usize).saturating_sub(4);
            let desc_text = if desc.len() > max_desc {
                format!(" {:.width$}…", desc, width = max_desc.saturating_sub(1))
            } else {
                format!(" {}", desc)
            };
            let line2 = Line::from(Span::styled(desc_text, Style::default().fg(TEXT_DIM)));

            let bg = if is_selected { BG_HIGHLIGHT } else { BG };
            ListItem::new(vec![line1, line2]).style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(items).style(Style::default().bg(BG)), list_area);
}

fn draw_theme_detail(f: &mut Frame, area: Rect, app: &App) {
    let block = UiBlock::default()
        .title(Span::styled(" Preview ", Style::default().fg(ACCENT2).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(TEXT_DARK))
        .style(Style::default().bg(BG_PANEL));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(theme) = app.selected_theme() else {
        f.render_widget(
            Paragraph::new(Span::styled("  ← Select a theme to view details", Style::default().fg(TEXT_DIM)))
                .style(Style::default().bg(BG_PANEL)),
            inner,
        );
        return;
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // name
            Constraint::Length(2),  // description
            Constraint::Length(1),  // separator
            Constraint::Min(4),    // preview
            Constraint::Length(1),  // separator
            Constraint::Length(2),  // action
        ])
        .split(inner);

    // Theme name
    let is_active = app.active_theme_name.as_deref() == Some(&theme.name);
    let mut name_spans = vec![
        Span::styled(format!("  {}", theme.name), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ];
    if is_active {
        name_spans.push(Span::styled("  ★ ACTIVE", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)));
    }
    f.render_widget(Paragraph::new(Line::from(name_spans)).style(Style::default().bg(BG_PANEL)), layout[0]);

    // Description
    f.render_widget(
        Paragraph::new(Span::styled(
            format!("  {}", theme.description),
            Style::default().fg(TEXT_DIM),
        )).wrap(Wrap { trim: false })
          .style(Style::default().bg(BG_PANEL)),
        layout[1],
    );

    // Separator
    let sep = "─".repeat(inner.width as usize);
    f.render_widget(Paragraph::new(Span::styled(&sep, Style::default().fg(TEXT_DARK))).style(Style::default().bg(BG_PANEL)), layout[2]);

    // Preview
    if let Some(ref cfg) = theme.config {
        let mut preview_lines = vec![
            Line::from(Span::styled("  Prompt Preview:", Style::default().fg(ACCENT2))),
            Line::from(""),
        ];
        let rendered = crate::preview::render_preview(cfg, BG_PANEL);
        for pl in rendered {
            preview_lines.push(pl);
        }
        f.render_widget(Paragraph::new(preview_lines).wrap(Wrap { trim: false }).style(Style::default().bg(BG_PANEL)), layout[3]);
    } else {
        // Config not loaded yet - either loading from disk or downloading
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {}", theme.name),
                    Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {}", theme.description),
                    Style::default().fg(TEXT_DIM),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  ⏳ Downloading preview…",
                    Style::default().fg(ACCENT2),
                )),
                Line::from(Span::styled(
                    "  Navigate away and back, or press Enter to apply.",
                    Style::default().fg(TEXT_DARK),
                )),
            ]).wrap(Wrap { trim: false })
              .style(Style::default().bg(BG_PANEL)),
            layout[3],
        );
    }

    // Separator
    f.render_widget(Paragraph::new(Span::styled(&sep, Style::default().fg(TEXT_DARK))).style(Style::default().bg(BG_PANEL)), layout[4]);

    // Action buttons (clear, visible, TPP-style)
    if is_active {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(" ★ Currently Active ", Style::default().fg(BG).bg(YELLOW).add_modifier(Modifier::BOLD)),
            ])).style(Style::default().bg(BG_PANEL)),
            layout[5],
        );
    } else {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(" ⬇ Apply Theme ", Style::default().fg(BG).bg(GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("  Enter", Style::default().fg(TEXT_DARK)),
            ])).style(Style::default().bg(BG_PANEL)),
            layout[5],
        );
    }
}

// ── Status bar ───────────────────────────────────────────────────────────────

fn draw_status(f: &mut Frame, area: Rect, app: &App) {
    if app.status.tick == 0 || app.status.text.is_empty() {
        return;
    }
    let bg_color = if app.status.is_error { RED } else { GREEN };
    f.render_widget(
        Paragraph::new(Span::styled(
            format!(" {} ", app.status.text),
            Style::default().fg(BG).bg(bg_color).add_modifier(Modifier::BOLD),
        )),
        area,
    );
}

// ── Footer ───────────────────────────────────────────────────────────────────

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let keys: Vec<(&str, &str)> = match app.tab {
        Tab::Dashboard => vec![
            ("↑↓", "Navigate"), ("Enter", "Select"),
            ("Tab", "Next Tab"), ("q", "Quit"),
        ],
        Tab::Setup => vec![
            ("←→", "Panel"), ("↑↓", "Navigate"), ("Enter", "Execute"),
            ("Space", "Toggle"), ("Tab", "Next Tab"), ("q", "Quit"),
        ],
        Tab::Themes => vec![
            ("↑↓", "Navigate"), ("←→", "Panel"), ("/", "Search"),
            ("Enter", "Apply"), ("Tab", "Next Tab"), ("q", "Quit"),
        ],
    };

    let spans: Vec<Span> = keys.iter().flat_map(|(key, label)| {
        vec![
            Span::styled(format!(" {key} "), Style::default().fg(BG).bg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {label} "), Style::default().fg(TEXT_DIM).bg(BG)),
        ]
    }).collect();

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Confirm overlay ──────────────────────────────────────────────────────────

fn draw_confirm(f: &mut Frame, area: Rect, app: &mut App) {
    if app.confirm.is_none() { return; }

    let width = 54.min(area.width.saturating_sub(4));
    let height = 9.min(area.height.saturating_sub(4));
    let cx = (area.width.saturating_sub(width)) / 2;
    let cy = (area.height.saturating_sub(height)) / 2;
    let rect = Rect::new(cx, cy, width, height);

    // Record layout for mouse hit-testing
    app.layout.confirm_area = Some((rect.x, rect.y, rect.width, rect.height));

    let dialog = app.confirm.as_ref().unwrap();

    f.render_widget(Clear, rect);

    let block = UiBlock::default()
        .title(Span::styled(
            format!(" {} ", dialog.title),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(BG_PANEL));
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    // Message
    f.render_widget(
        Paragraph::new(Span::styled(&dialog.message, Style::default().fg(TEXT)))
            .wrap(Wrap { trim: false }),
        rows[0],
    );

    // Hint
    f.render_widget(
        Paragraph::new(Span::styled(
            "  ←→/Tab toggle · Enter confirm · Esc/n cancel · y yes",
            Style::default().fg(TEXT_DARK),
        )),
        rows[1],
    );

    // Buttons
    let cancel_style = if !dialog.confirm_selected {
        Style::default().fg(BG).bg(RED).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(TEXT_DIM)
    };
    let ok_style = if dialog.confirm_selected {
        Style::default().fg(BG).bg(GREEN).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(TEXT_DIM)
    };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(" Cancel ", cancel_style),
            Span::styled("    ", Style::default()),
            Span::styled(" Confirm ", ok_style),
        ])),
        rows[2],
    );
}
