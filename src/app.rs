// ─── Central Application State ───────────────────────────────────────────────
//
// The `App` struct owns **all** runtime state.  The UI reads from it; the
// event loop mutates it.  No other module holds mutable state.

use std::path::PathBuf;
use std::sync::mpsc;

use crate::config::OmpConfig;
use crate::detect::{DetectionReport, FontStatus, OmpInfo, OsInfo};
use crate::install::SetupStep;
use crate::shell::ShellInfo;
use crate::themes::{ThemeCategory, ThemeEntry};

// ── Enums ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Setup,
    Themes,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[Self::Dashboard, Self::Setup, Self::Themes];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Setup     => "Setup",
            Self::Themes    => "Themes",
        }
    }

    pub fn index(&self) -> usize {
        Self::ALL.iter().position(|t| t == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupFocus {
    Steps,
    Detail,
}

// ── Dashboard items ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardItem {
    QuickSetup,
    BrowseThemes,
    InstallFont,
    UpdateOmp,
    ResetDefault,
}

impl DashboardItem {
    pub const ALL: &'static [DashboardItem] = &[
        Self::QuickSetup,
        Self::BrowseThemes,
        Self::InstallFont,
        Self::UpdateOmp,
        Self::ResetDefault,
    ];

    pub fn icon(&self) -> &'static str {
        match self {
            Self::QuickSetup   => "⚡",
            Self::BrowseThemes => "🎨",
            Self::InstallFont  => "🔤",
            Self::UpdateOmp    => "⟳",
            Self::ResetDefault => "🔄",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::QuickSetup   => "Quick Setup",
            Self::BrowseThemes => "Browse Themes",
            Self::InstallFont  => "Install Fonts",
            Self::UpdateOmp    => "Update Oh My Posh",
            Self::ResetDefault => "Reset to Default",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::QuickSetup   => "Install everything and configure shells in one go",
            Self::BrowseThemes => "Find your perfect prompt theme from 100+ options",
            Self::InstallFont  => "Download and install all required Nerd Fonts for icons",
            Self::UpdateOmp    => "Check for and install the latest Oh My Posh version",
            Self::ResetDefault => "Remove Oh My Posh from all shells and restore defaults",
        }
    }

    pub fn key_hint(&self) -> &'static str {
        match self {
            Self::QuickSetup   => "S",
            Self::BrowseThemes => "T",
            Self::InstallFont  => "F",
            Self::UpdateOmp    => "U",
            Self::ResetDefault => "R",
        }
    }
}

// ── Confirm dialog ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    InstallOmp,
    InstallFont,
    ApplyTheme,
    ConfigureShell,
    UnconfigureShell,
    ConfigureAllShells,
    ResetToDefault,
}

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub action: ConfirmAction,
    pub confirm_selected: bool,
    /// Extra context — e.g. theme name, font name, shell index.
    pub context: String,
}

// ── Status message ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
    pub tick: u16,
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self { text: String::new(), is_error: false, tick: 0 }
    }
}

// ── Layout regions for mouse hit-testing ─────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct LayoutRegions {
    pub tab_bar: Option<(u16, u16, u16, u16)>,
    pub sidebar: Option<(u16, u16, u16, u16)>,
    pub list: Option<(u16, u16, u16, u16)>,
    pub detail: Option<(u16, u16, u16, u16)>,
    pub setup_area: Option<(u16, u16, u16, u16)>,
    pub setup_steps: Option<(u16, u16, u16, u16)>,
    pub setup_detail: Option<(u16, u16, u16, u16)>,
    pub dashboard_area: Option<(u16, u16, u16, u16)>,
    pub confirm_area: Option<(u16, u16, u16, u16)>,
}

// ── App ──────────────────────────────────────────────────────────────────────

pub struct App {
    // ── Global ───────────────────────────────────────────────────────────────
    pub running: bool,
    pub tab: Tab,
    pub focus: Focus,

    // ── Detection ────────────────────────────────────────────────────────────
    pub os_info: OsInfo,
    pub omp: Option<OmpInfo>,
    pub fonts: FontStatus,
    pub shells: Vec<ShellInfo>,

    // ── Dashboard ────────────────────────────────────────────────────────────
    pub dashboard_selected: usize,

    // ── Setup wizard ─────────────────────────────────────────────────────────
    pub setup_steps: Vec<SetupStep>,
    pub setup_current: usize,
    pub setup_font_selected: usize,
    pub setup_theme_selected: usize,
    pub setup_shell_toggles: Vec<bool>,
    pub setup_focus: SetupFocus,
    pub setup_shell_selected: usize,

    // ── Themes ───────────────────────────────────────────────────────────────
    pub theme_list: Vec<ThemeEntry>,
    pub theme_filtered: Vec<usize>,
    pub theme_selected: usize,
    pub theme_scroll: usize,
    pub theme_visible_height: usize,
    pub theme_search: String,
    pub theme_search_editing: bool,
    pub theme_category: ThemeCategory,
    pub theme_category_index: usize,
    pub active_theme_name: Option<String>,


    // ── Dialogs ──────────────────────────────────────────────────────────────
    pub confirm: Option<ConfirmDialog>,
    pub status: StatusMessage,

    // ── Layout ───────────────────────────────────────────────────────────────
    pub layout: LayoutRegions,

    // ── Background downloads ─────────────────────────────────────────────────
    /// Receives (theme_index, downloaded_path, parsed_config) from bg threads.
    pub theme_download_rx: mpsc::Receiver<(usize, PathBuf, OmpConfig)>,
    pub theme_download_tx: mpsc::Sender<(usize, PathBuf, OmpConfig)>,
    /// Set of theme indices currently being downloaded (avoid duplicate spawns).
    pub theme_downloading: std::collections::HashSet<usize>,
}

impl App {
    /// Construct the app from a detection report.
    pub fn new(report: DetectionReport, shells: Vec<ShellInfo>) -> Self {
        let omp_installed = report.omp.is_some();
        let has_font = report.fonts.has_nerd_font;

        let themes_dir = report.omp.as_ref().and_then(|o| o.themes_path.as_deref());
        let theme_list = crate::themes::discover_themes(themes_dir);
        let theme_filtered = (0..theme_list.len()).collect();

        let setup_steps = crate::install::create_setup_steps(omp_installed, has_font);

        // Determine current theme from first configured shell
        let active_theme_name = shells.iter()
            .find_map(|s| s.current_theme.as_ref())
            .and_then(|p| {
                std::path::Path::new(p)
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .map(|n| n.trim_end_matches(".omp").to_string())
            });

        let setup_shell_toggles = shells.iter().map(|s| s.available && !s.omp_configured).collect();

        let (tx, rx) = mpsc::channel();

        Self {
            running: true,
            tab: Tab::Dashboard,
            focus: Focus::List,
            os_info: report.os,
            omp: report.omp,
            fonts: report.fonts,
            shells,
            dashboard_selected: 0,
            setup_steps,
            setup_current: 0,
            setup_font_selected: 0,
            setup_theme_selected: 0,
            setup_shell_toggles,
            setup_focus: SetupFocus::Steps,
            setup_shell_selected: 0,
            theme_list,
            theme_filtered,
            theme_selected: 0,
            theme_scroll: 0,
            theme_visible_height: 20,
            theme_search: String::new(),
            theme_search_editing: false,
            theme_category: ThemeCategory::All,
            theme_category_index: 0,
            active_theme_name,
            confirm: None,
            status: StatusMessage::default(),
            layout: LayoutRegions::default(),
            theme_download_rx: rx,
            theme_download_tx: tx,
            theme_downloading: std::collections::HashSet::new(),
        }
    }

    // ── Status helpers ───────────────────────────────────────────────────────

    pub fn set_status(&mut self, msg: &str) {
        self.status = StatusMessage { text: msg.into(), is_error: false, tick: 120 };
    }

    pub fn set_status_err(&mut self, msg: &str) {
        self.status = StatusMessage { text: msg.into(), is_error: true, tick: 160 };
    }

    pub fn tick_status(&mut self) {
        if self.status.tick > 0 {
            self.status.tick -= 1;
        }
    }

    // ── Theme helpers ────────────────────────────────────────────────────────

    /// Re-discover themes from disk (e.g. after OMP install).
    pub fn refresh_themes(&mut self) {
        let themes_dir = self.omp.as_ref().and_then(|o| o.themes_path.as_deref());
        self.theme_list = crate::themes::discover_themes(themes_dir);
        self.apply_theme_filter();
    }

    pub fn apply_theme_filter(&mut self) {
        if self.theme_search.is_empty() {
            self.theme_filtered = crate::themes::filter_themes(&self.theme_list, self.theme_category);
        } else {
            let by_cat = crate::themes::filter_themes(&self.theme_list, self.theme_category);
            let by_search = crate::themes::search_themes(&self.theme_list, &self.theme_search);
            self.theme_filtered = by_cat.into_iter().filter(|i| by_search.contains(i)).collect();
        }
        self.theme_selected = 0;
        self.theme_scroll = 0;
    }

    /// Currently selected theme (if any).
    pub fn selected_theme(&self) -> Option<&ThemeEntry> {
        self.theme_filtered.get(self.theme_selected)
            .and_then(|&i| self.theme_list.get(i))
    }

    /// Load the config for the currently selected theme (lazy).
    /// If the theme JSON isn't on disk, spawns a background download for preview.
    pub fn load_selected_theme_config(&mut self) {
        if let Some(&idx) = self.theme_filtered.get(self.theme_selected) {
            let entry = &self.theme_list[idx];
            if entry.config.is_some() {
                return; // already loaded
            }

            // If we have a local path, load from disk (fast, synchronous)
            if let Some(ref path) = entry.path {
                if let Ok(cfg) = crate::config::load_config(path) {
                    self.theme_list[idx].config = Some(cfg);
                    return;
                }
            }

            // No local file — spawn background download (non-blocking)
            if self.theme_downloading.contains(&idx) {
                return; // already downloading
            }
            let themes_dir = self.omp.as_ref().and_then(|o| o.themes_path.as_deref());
            if let Some(cache) = crate::themes::themes_cache_dir(themes_dir) {
                let name = entry.name.clone();
                let tx = self.theme_download_tx.clone();
                self.theme_downloading.insert(idx);
                std::thread::spawn(move || {
                    if let Ok(path) = crate::themes::download_theme(&name, &cache) {
                        if let Ok(cfg) = crate::config::load_config(&path) {
                            let _ = tx.send((idx, path, cfg));
                        }
                    }
                });
            }
        }
    }

    /// Poll for completed background theme downloads and apply them.
    pub fn poll_theme_downloads(&mut self) {
        while let Ok((idx, path, cfg)) = self.theme_download_rx.try_recv() {
            if idx < self.theme_list.len() {
                self.theme_list[idx].config = Some(cfg);
                self.theme_list[idx].path = Some(path);
            }
            self.theme_downloading.remove(&idx);
        }
    }

    // ── Navigation ───────────────────────────────────────────────────────────

    pub fn next_tab(&mut self) {
        let idx = self.tab.index();
        let next = (idx + 1) % Tab::ALL.len();
        self.tab = Tab::ALL[next];
    }

    pub fn prev_tab(&mut self) {
        let idx = self.tab.index();
        let prev = if idx == 0 { Tab::ALL.len() - 1 } else { idx - 1 };
        self.tab = Tab::ALL[prev];
    }

    /// Generic scroll helper — clamp within `0..len`.
    pub fn clamp_selection(sel: &mut usize, scroll: &mut usize, len: usize, visible: usize) {
        if len == 0 { *sel = 0; *scroll = 0; return; }
        if *sel >= len { *sel = len - 1; }
        if *sel < *scroll { *scroll = *sel; }
        if *sel >= *scroll + visible { *scroll = *sel - visible + 1; }
    }
}
