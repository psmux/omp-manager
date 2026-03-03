// ─── PowerShell-Inspired Color Palette ───────────────────────────────────────
//
// The entire UI uses a deep-blue console aesthetic inspired by PowerShell's
// default terminal profile ("Campbell PowerShell").

use ratatui::style::Color;

/// Deep navy - classic PowerShell console background.
pub const BG: Color = Color::Rgb(1, 36, 86);
/// Slightly lighter blue for panels, cards, input fields.
pub const BG_PANEL: Color = Color::Rgb(6, 46, 100);
/// Brighter blue for the selected / highlighted row.
pub const BG_HIGHLIGHT: Color = Color::Rgb(14, 69, 131);
/// Even brighter hover-like accent.
pub const BG_ACTIVE: Color = Color::Rgb(24, 89, 161);

/// PowerShell gold - primary accent for headers, active tab, badges.
pub const ACCENT: Color = Color::Rgb(255, 185, 0);
/// Bright cyan - secondary accent for links, info, buttons.
pub const ACCENT2: Color = Color::Rgb(59, 186, 243);

/// Main body text - near-white.
pub const TEXT: Color = Color::Rgb(229, 229, 229);
/// Muted text for descriptions, secondary info.
pub const TEXT_DIM: Color = Color::Rgb(128, 148, 174);
/// Very dim text used for borders, separators.
pub const TEXT_DARK: Color = Color::Rgb(38, 62, 100);

/// Success / installed / check-marks.
pub const GREEN: Color = Color::Rgb(22, 198, 12);
/// Errors, destructive actions.
pub const RED: Color = Color::Rgb(231, 72, 86);
/// Warnings, stars, highlights.
pub const YELLOW: Color = Color::Rgb(249, 241, 165);
/// Informational blue.
pub const BLUE: Color = Color::Rgb(59, 120, 255);
/// Purple accent (PowerShell 7 vibe).
pub const PURPLE: Color = Color::Rgb(180, 0, 158);
