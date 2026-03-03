// ─── Theme Catalog & Management ──────────────────────────────────────────────
//
// Curated list of every built-in Oh My Posh theme, plus runtime discovery of
// themes from disk.  Handles listing, filtering, searching, and applying.

use std::path::{Path, PathBuf};

use crate::config::OmpConfig;

// ── Theme category ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeCategory {
    All,
    Popular,
    Minimal,
    Powerline,
    Colorful,
    Dark,
    Light,
}

impl ThemeCategory {
    pub const ALL: &'static [ThemeCategory] = &[
        Self::All,
        Self::Popular,
        Self::Minimal,
        Self::Powerline,
        Self::Colorful,
        Self::Dark,
        Self::Light,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::All       => "All Themes",
            Self::Popular   => "⭐ Popular",
            Self::Minimal   => "◽ Minimal",
            Self::Powerline => "▶ Powerline",
            Self::Colorful  => "🎨 Colorful",
            Self::Dark      => "🌙 Dark",
            Self::Light     => "☀ Light",
        }
    }
}

// ── Theme entry ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ThemeEntry {
    /// Theme filename without the `.omp.json` suffix.
    pub name: String,
    /// Brief description.
    pub description: String,
    /// Categories this theme belongs to.
    pub categories: Vec<ThemeCategory>,
    /// Absolute path to the `.omp.json` file (if found on disk).
    pub path: Option<PathBuf>,
    /// Parsed config (loaded lazily on selection).
    pub config: Option<OmpConfig>,
}

// ── Built-in catalog ─────────────────────────────────────────────────────────
// (name, description, categories)

type ThemeDef = (&'static str, &'static str, &'static [ThemeCategory]);

#[rustfmt::skip]
static BUILTIN_THEMES: &[ThemeDef] = &[
    ("agnoster",              "Classic powerline-style prompt",                       &[ThemeCategory::Powerline, ThemeCategory::Popular]),
    ("agnosterplus",          "Extended agnoster with extra segments",                &[ThemeCategory::Powerline, ThemeCategory::Popular]),
    ("aliens",                "Alien-themed creative prompt",                         &[ThemeCategory::Colorful]),
    ("amro",                  "Clean two-line prompt",                                &[ThemeCategory::Minimal]),
    ("atomic",                "Atomic design-inspired prompt",                        &[ThemeCategory::Colorful]),
    ("atomicBit",             "Atomic variant with bit-icons",                        &[ThemeCategory::Colorful]),
    ("avit",                  "Minimal avit-style prompt",                            &[ThemeCategory::Minimal]),
    ("blueish",               "Cool blue tones prompt",                               &[ThemeCategory::Dark, ThemeCategory::Colorful]),
    ("blue-owl",              "Elegant blue owl theme",                               &[ThemeCategory::Dark]),
    ("bubbles",               "Rounded bubble segments",                              &[ThemeCategory::Colorful, ThemeCategory::Popular]),
    ("bubblesextra",          "Extended bubbles with more info",                      &[ThemeCategory::Colorful]),
    ("bubblesline",           "Bubbles with line decorations",                        &[ThemeCategory::Colorful]),
    ("capr4n",                "Capr4n creative theme",                                &[ThemeCategory::Colorful]),
    ("catppuccin",            "Catppuccin color palette",                             &[ThemeCategory::Popular, ThemeCategory::Colorful]),
    ("catppuccin_frappe",     "Catppuccin Frappé variant",                            &[ThemeCategory::Colorful]),
    ("catppuccin_latte",      "Catppuccin Latte – light variant",                     &[ThemeCategory::Light, ThemeCategory::Colorful]),
    ("catppuccin_macchiato",  "Catppuccin Macchiato variant",                         &[ThemeCategory::Colorful]),
    ("catppuccin_mocha",      "Catppuccin Mocha – rich dark",                         &[ThemeCategory::Dark, ThemeCategory::Popular]),
    ("cert",                  "Certificate-style prompt",                             &[ThemeCategory::Minimal]),
    ("clean-detailed",        "Clean with detailed system info",                      &[ThemeCategory::Minimal]),
    ("cloud-context",         "Shows cloud provider context",                         &[ThemeCategory::Powerline]),
    ("cobalt2",               "Cobalt2-inspired vibrant theme",                       &[ThemeCategory::Colorful, ThemeCategory::Popular]),
    ("craver",                "Craver creative prompt",                               &[ThemeCategory::Colorful]),
    ("darkblood",             "Dark blood-red theme",                                 &[ThemeCategory::Dark]),
    ("default",               "Oh My Posh default theme",                             &[ThemeCategory::Minimal, ThemeCategory::Popular]),
    ("di4am0nd",              "Diamond-style prompt segments",                        &[ThemeCategory::Colorful]),
    ("dracula",               "Dracula color scheme",                                 &[ThemeCategory::Dark, ThemeCategory::Popular]),
    ("easy-term",             "Easy-to-read terminal prompt",                         &[ThemeCategory::Minimal]),
    ("emodipt",               "Emoji-powered prompt",                                 &[ThemeCategory::Colorful]),
    ("emodipt-extend",        "Extended emoji prompt",                                &[ThemeCategory::Colorful]),
    ("fish",                  "Fish shell inspired prompt",                           &[ThemeCategory::Minimal]),
    ("free-ukraine",          "Blue and gold Ukraine theme",                          &[ThemeCategory::Colorful]),
    ("froczh",                "Froczh creative theme",                                &[ThemeCategory::Colorful]),
    ("glowsticks",            "Neon glow-stick prompt",                               &[ThemeCategory::Colorful]),
    ("gmay",                  "Gmay clean prompt",                                    &[ThemeCategory::Minimal]),
    ("gruvbox",               "Gruvbox retro palette",                                &[ThemeCategory::Dark, ThemeCategory::Popular]),
    ("half-life",             "Half-Life game inspired",                              &[ThemeCategory::Dark]),
    ("honukai",               "Honukai colorful theme",                               &[ThemeCategory::Colorful]),
    ("hotstick.minimal",      "Hot stick minimal variant",                            &[ThemeCategory::Minimal]),
    ("hul10",                 "Hul10 developer prompt",                               &[ThemeCategory::Powerline]),
    ("huvix",                 "Huvix modern prompt",                                  &[ThemeCategory::Colorful]),
    ("if_tea",                "If-tea creative theme",                                &[ThemeCategory::Colorful]),
    ("illusi0n",              "Illusion gradient prompt",                             &[ThemeCategory::Colorful]),
    ("iterm2",                "iTerm2-style prompt",                                  &[ThemeCategory::Minimal]),
    ("jandedobbeleer",        "Jan De Dobbeleer's personal theme",                    &[ThemeCategory::Powerline, ThemeCategory::Popular]),
    ("jblab_2021",            "JBLab 2021 prompt",                                    &[ThemeCategory::Powerline]),
    ("jonnychipz",            "Jonnychipz creative prompt",                           &[ThemeCategory::Colorful]),
    ("json",                  "JSON-format display prompt",                            &[ThemeCategory::Minimal]),
    ("jtracey93",             "jtracey93 developer theme",                            &[ThemeCategory::Powerline]),
    ("jv_sitecoredude",       "Sitecore developer theme",                             &[ThemeCategory::Powerline]),
    ("kali",                  "Kali Linux inspired prompt",                           &[ThemeCategory::Dark, ThemeCategory::Popular]),
    ("kushal",                "Kushal clean theme",                                   &[ThemeCategory::Minimal]),
    ("lambdageneration",      "Lambda-symbol prompt",                                 &[ThemeCategory::Minimal]),
    ("larserikfinholt",       "Lars Erik Finholt theme",                              &[ThemeCategory::Colorful]),
    ("M365Princess",          "Microsoft 365 princess theme",                         &[ThemeCategory::Colorful, ThemeCategory::Light]),
    ("marcduiker",            "Marc Duiker's Azure theme",                            &[ThemeCategory::Powerline]),
    ("markbull",              "Markbull developer prompt",                            &[ThemeCategory::Powerline]),
    ("material",              "Material design colors",                               &[ThemeCategory::Colorful, ThemeCategory::Popular]),
    ("microverse-power",      "Micro-verse power prompt",                             &[ThemeCategory::Powerline]),
    ("mojada",                "Mojada dark theme",                                    &[ThemeCategory::Dark]),
    ("montys",                "Monty's creative prompt",                              &[ThemeCategory::Colorful]),
    ("mt",                    "MT minimal two-line",                                  &[ThemeCategory::Minimal]),
    ("negligible",            "Ultra-minimal prompt",                                 &[ThemeCategory::Minimal]),
    ("neko",                  "Neko cat-themed prompt",                               &[ThemeCategory::Colorful]),
    ("night-owl",             "Night Owl VS Code theme colors",                       &[ThemeCategory::Dark, ThemeCategory::Popular]),
    ("nordtron",              "Nord color scheme",                                    &[ThemeCategory::Dark, ThemeCategory::Popular]),
    ("nu4a",                  "Nu4a modern prompt",                                   &[ThemeCategory::Minimal]),
    ("onehalf.minimal",       "One Half minimal variant",                             &[ThemeCategory::Minimal]),
    ("paradox",               "Paradox powerline prompt",                             &[ThemeCategory::Powerline, ThemeCategory::Popular]),
    ("patriksvensson",        "Patrik Svensson's theme",                              &[ThemeCategory::Powerline]),
    ("peru",                  "Peru warm tones",                                      &[ThemeCategory::Colorful]),
    ("pixelrobots",           "Pixel robots fun theme",                               &[ThemeCategory::Colorful]),
    ("plague",                "Plague dark theme",                                    &[ThemeCategory::Dark]),
    ("powerlevel10k_classic", "P10K classic layout",                                  &[ThemeCategory::Powerline, ThemeCategory::Popular]),
    ("powerlevel10k_lean",    "P10K lean minimal",                                    &[ThemeCategory::Powerline, ThemeCategory::Minimal]),
    ("powerlevel10k_modern",  "P10K modern with icons",                               &[ThemeCategory::Powerline, ThemeCategory::Popular]),
    ("powerlevel10k_rainbow", "P10K rainbow colorful",                                &[ThemeCategory::Powerline, ThemeCategory::Colorful, ThemeCategory::Popular]),
    ("powerline",             "Classic powerline prompt",                             &[ThemeCategory::Powerline, ThemeCategory::Popular]),
    ("probua.minimal",        "Probua minimal prompt",                               &[ThemeCategory::Minimal]),
    ("pure",                  "Pure ultra-minimal prompt",                            &[ThemeCategory::Minimal, ThemeCategory::Popular]),
    ("quick-term",            "Quick terminal fast prompt",                           &[ThemeCategory::Minimal]),
    ("remk",                  "Remk creative theme",                                  &[ThemeCategory::Colorful]),
    ("robbyrussel",           "Robby Russell (oh-my-zsh default)",                    &[ThemeCategory::Minimal, ThemeCategory::Popular]),
    ("rudolfs-dark",          "Rudolf's dark theme",                                  &[ThemeCategory::Dark]),
    ("rudolfs-light",         "Rudolf's light theme",                                 &[ThemeCategory::Light]),
    ("slim",                  "Slim single-line prompt",                              &[ThemeCategory::Minimal]),
    ("slimfat",               "Slimfat two-line compact",                             &[ThemeCategory::Minimal]),
    ("smoothie",              "Smoothie gradient prompt",                             &[ThemeCategory::Colorful]),
    ("sonicboom_dark",        "Sonic Boom dark variant",                              &[ThemeCategory::Dark, ThemeCategory::Colorful]),
    ("sonicboom_light",       "Sonic Boom light variant",                             &[ThemeCategory::Light, ThemeCategory::Colorful]),
    ("space",                 "Space-themed prompt",                                  &[ThemeCategory::Dark]),
    ("spaceship",             "Spaceship two-line prompt",                            &[ThemeCategory::Popular, ThemeCategory::Powerline]),
    ("star",                  "Star minimal prompt",                                  &[ThemeCategory::Minimal]),
    ("stelbent-compact.minimal", "Stelbent compact minimal",                         &[ThemeCategory::Minimal]),
    ("stelbent.minimal",      "Stelbent minimal prompt",                              &[ThemeCategory::Minimal]),
    ("takuya",                "Takuya Japanese-style prompt",                         &[ThemeCategory::Colorful]),
    ("thecyberden",           "The Cyberden hacker prompt",                           &[ThemeCategory::Dark]),
    ("the-unnamed",           "The Unnamed mysterious prompt",                        &[ThemeCategory::Dark]),
    ("tiwahu",                "Tiwahu developer prompt",                              &[ThemeCategory::Powerline]),
    ("tokyonight_storm",      "Tokyo Night Storm colors",                             &[ThemeCategory::Dark, ThemeCategory::Popular]),
    ("uew",                   "UEW university prompt",                                &[ThemeCategory::Colorful]),
    ("unicorn",               "Unicorn rainbow prompt",                               &[ThemeCategory::Colorful]),
    ("velvet",                "Velvet rich dark prompt",                              &[ThemeCategory::Dark]),
    ("wholespace",            "Whole-space wide prompt",                              &[ThemeCategory::Powerline]),
    ("wopian",                "Wopian modern prompt",                                 &[ThemeCategory::Colorful]),
    ("xtoys",                 "X-toys creative prompt",                               &[ThemeCategory::Colorful]),
    ("ys",                    "YS two-line prompt",                                   &[ThemeCategory::Minimal]),
    ("zash",                  "Zash clean prompt",                                    &[ThemeCategory::Minimal]),
];

// ── Discovery ────────────────────────────────────────────────────────────────

/// Build the full theme list.
/// If `themes_dir` is available (OMP installed), we discover themes from disk
/// and merge with the built-in catalog.  Otherwise only the built-in list.
pub fn discover_themes(themes_dir: Option<&Path>) -> Vec<ThemeEntry> {
    let mut themes: Vec<ThemeEntry> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Scan disk themes
    if let Some(dir) = themes_dir {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = theme_name_from_path(&path) {
                    seen.insert(name.clone());
                    let builtin = lookup_builtin(&name);
                    themes.push(ThemeEntry {
                        name: name.clone(),
                        description: builtin.map(|b| b.1.to_string()).unwrap_or_else(|| format!("{name} theme")),
                        categories: builtin.map(|b| b.2.to_vec()).unwrap_or_else(|| vec![ThemeCategory::All]),
                        path: Some(path),
                        config: None,
                    });
                }
            }
        }
    }

    // Fill in any built-in themes not on disk
    for &(name, desc, cats) in BUILTIN_THEMES {
        if !seen.contains(name) {
            themes.push(ThemeEntry {
                name: name.to_string(),
                description: desc.to_string(),
                categories: cats.to_vec(),
                path: None,
                config: None,
            });
        }
    }

    themes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    themes
}

/// Extract theme name from a path like `/foo/paradox.omp.json`.
fn theme_name_from_path(path: &Path) -> Option<String> {
    let fname = path.file_name()?.to_str()?;
    if fname.ends_with(".omp.json") {
        Some(fname.trim_end_matches(".omp.json").to_string())
    } else if fname.ends_with(".omp.yaml") || fname.ends_with(".omp.toml") {
        let base = fname.rsplit_once('.').map(|(rest, _)| rest)?;
        let name = base.trim_end_matches(".omp");
        Some(name.to_string())
    } else {
        None
    }
}

fn lookup_builtin(name: &str) -> Option<&'static ThemeDef> {
    BUILTIN_THEMES.iter().find(|(n, _, _)| *n == name)
}

// ── Filtering ────────────────────────────────────────────────────────────────

/// Filter themes by category (All returns everything).
pub fn filter_themes(themes: &[ThemeEntry], cat: ThemeCategory) -> Vec<usize> {
    if cat == ThemeCategory::All {
        (0..themes.len()).collect()
    } else {
        themes.iter().enumerate()
            .filter(|(_, t)| t.categories.contains(&cat))
            .map(|(i, _)| i)
            .collect()
    }
}

/// Search themes by name/description substring.
pub fn search_themes(themes: &[ThemeEntry], query: &str) -> Vec<usize> {
    let q = query.to_lowercase();
    themes.iter().enumerate()
        .filter(|(_, t)| t.name.to_lowercase().contains(&q) || t.description.to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect()
}

// ── Theme download ───────────────────────────────────────────────────────────

/// The base URL for raw Oh My Posh theme files on GitHub.
const OMP_THEMES_RAW_URL: &str = "https://raw.githubusercontent.com/JanDeDobbeleer/oh-my-posh/main/themes";

/// Get or create the local themes cache directory.
/// Uses the OMP themes dir if available, otherwise falls back to a local cache.
pub fn themes_cache_dir(omp_themes_dir: Option<&Path>) -> Option<PathBuf> {
    if let Some(dir) = omp_themes_dir {
        if dir.is_dir() {
            return Some(dir.to_path_buf());
        }
    }
    // Fallback: create a cache under the user's data dir
    let base = dirs::data_local_dir()
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    let cache = base.join("omp-manager").join("themes");
    let _ = std::fs::create_dir_all(&cache);
    Some(cache)
}

/// Download a single theme file from the Oh My Posh GitHub repo.
/// Returns the local path on success.
pub fn download_theme(name: &str, dest_dir: &Path) -> Result<PathBuf, String> {
    let url = format!("{OMP_THEMES_RAW_URL}/{name}.omp.json");
    let dest = dest_dir.join(format!("{name}.omp.json"));

    // Use platform-native download
    #[cfg(target_os = "windows")]
    {
        let ps_cmd = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
             Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
            url,
            dest.to_string_lossy()
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {e}"))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Download failed: {}", err.trim()));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("curl")
            .args(["-fsSL", "-o", &dest.to_string_lossy(), &url])
            .output()
            .map_err(|e| format!("Failed to run curl: {e}"))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Download failed: {}", err.trim()));
        }
    }

    if dest.exists() && std::fs::metadata(&dest).map(|m| m.len() > 10).unwrap_or(false) {
        Ok(dest)
    } else {
        Err(format!("Downloaded file is missing or empty for '{name}'"))
    }
}

/// Download all built-in themes that are not yet on disk.
/// Returns (downloaded_count, error_count).
pub fn download_all_themes(dest_dir: &Path) -> (usize, usize) {
    let mut ok = 0;
    let mut fail = 0;
    for &(name, _, _) in BUILTIN_THEMES {
        let target = dest_dir.join(format!("{name}.omp.json"));
        if target.exists() {
            continue;
        }
        match download_theme(name, dest_dir) {
            Ok(_) => ok += 1,
            Err(_) => fail += 1,
        }
    }
    (ok, fail)
}

// ── Application ──────────────────────────────────────────────────────────────

/// Get the path to a theme file, downloading it if necessary.
pub fn get_theme_file_path(theme: &ThemeEntry, themes_dir: Option<&Path>) -> Option<PathBuf> {
    // Prefer on-disk path
    if let Some(ref p) = theme.path {
        if p.exists() {
            return Some(p.clone());
        }
    }
    // Try to find in themes dir
    if let Some(dir) = themes_dir {
        let p = dir.join(format!("{}.omp.json", theme.name));
        if p.exists() {
            return Some(p);
        }
    }
    // Try downloading
    let cache = themes_cache_dir(themes_dir)?;
    match download_theme(&theme.name, &cache) {
        Ok(path) => Some(path),
        Err(_) => None,
    }
}
