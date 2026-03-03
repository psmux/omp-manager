// ─── Theme Preview Rendering ─────────────────────────────────────────────────
//
// Generates ratatui Span sequences that visualise what a theme's prompt would
// look like, by reading the parsed OmpConfig and producing colored text with
// powerline separators.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::config::OmpConfig;
use crate::segments::lookup_segment;

/// A single styled chunk of the preview prompt.
#[derive(Debug, Clone)]
pub struct PreviewChunk {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
}

/// Generate a full preview of a theme config as a list of Lines.
pub fn render_preview<'a>(config: &OmpConfig, panel_bg: Color) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    if config.blocks.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (empty config - no blocks)",
            Style::default().fg(Color::DarkGray),
        )));
        return lines;
    }

    for (bi, block) in config.blocks.iter().enumerate() {
        if block.block_type == "newline" {
            lines.push(Line::from(""));
            continue;
        }

        let mut spans: Vec<Span<'a>> = Vec::new();
        let mut prev_bg = panel_bg;

        for (si, seg) in block.segments.iter().enumerate() {
            let bg = parse_color(seg.background.as_deref()).unwrap_or(PALETTE[si % PALETTE.len()]);
            let fg = parse_color(seg.foreground.as_deref()).unwrap_or(Color::White);

            // Powerline separator between segments
            if si > 0 && seg.style == "powerline" {
                spans.push(Span::styled(
                    "\u{e0b0}".to_string(),
                    Style::default().fg(prev_bg).bg(bg),
                ));
            }

            // Segment content - use example text from catalog or segment type name
            let label = lookup_segment(&seg.seg_type)
                .map(|info| format!(" {} {} ", info.icon, info.example))
                .unwrap_or_else(|| format!(" {} ", seg.seg_type));

            spans.push(Span::styled(
                label,
                Style::default().fg(fg).bg(bg),
            ));
            prev_bg = bg;
        }

        // Trailing powerline arrow
        if !block.segments.is_empty() {
            spans.push(Span::styled(
                "\u{e0b0}".to_string(),
                Style::default().fg(prev_bg).bg(panel_bg),
            ));
        }

        // Alignment label
        let align_label = if block.alignment == "right" { "  (right)" } else { "" };
        if !align_label.is_empty() {
            spans.push(Span::styled(
                align_label.to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let block_label = format!("Block {} ({} {}):", bi + 1, block.block_type, block.alignment);
        lines.push(Line::from(Span::styled(
            block_label,
            Style::default().fg(Color::Rgb(128, 148, 174)),
        )));
        lines.push(Line::from(spans));
        lines.push(Line::from(""));
    }

    // Transient prompt preview
    if let Some(ref tp) = config.transient_prompt {
        let tmpl = tp.template.as_deref().unwrap_or("❯ ");
        lines.push(Line::from(Span::styled(
            format!("Transient: {tmpl}"),
            Style::default().fg(Color::Rgb(128, 148, 174)),
        )));
    }

    // Segment summary
    let types = config.segment_types();
    if !types.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Segments: {}", types.join(", ")),
            Style::default().fg(Color::Rgb(128, 148, 174)),
        )));
    }

    lines
}

/// Preview when theme file isn't available - just show the name and info.
pub fn render_placeholder<'a>(name: &str, description: &str) -> Vec<Line<'a>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {name}"),
            Style::default().fg(Color::White).add_modifier(ratatui::style::Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {description}"),
            Style::default().fg(Color::Rgb(160, 160, 160)),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Install Oh My Posh to see a live preview".to_string(),
            Style::default().fg(Color::DarkGray),
        )),
    ]
}

// ── Color parsing ────────────────────────────────────────────────────────────

/// Parse a hex color string like `"#61AFEF"` into a ratatui Color.
fn parse_color(hex: Option<&str>) -> Option<Color> {
    let hex = hex?;
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else {
        // Try named colors
        match hex.to_lowercase().as_str() {
            "black"   => Some(Color::Black),
            "red"     => Some(Color::Red),
            "green"   => Some(Color::Green),
            "yellow"  => Some(Color::Yellow),
            "blue"    => Some(Color::Blue),
            "magenta" => Some(Color::Magenta),
            "cyan"    => Some(Color::Cyan),
            "white"   => Some(Color::White),
            _ => None,
        }
    }
}

/// Fallback palette when theme doesn't specify colors.
const PALETTE: &[Color] = &[
    Color::Rgb(97, 175, 239),   // Blue
    Color::Rgb(152, 195, 121),  // Green
    Color::Rgb(229, 192, 123),  // Yellow
    Color::Rgb(198, 120, 221),  // Purple
    Color::Rgb(224, 108, 117),  // Red
    Color::Rgb(86, 182, 194),   // Cyan
];
