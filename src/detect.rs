// ─── OS / Binary / Font Detection ────────────────────────────────────────────
//
// Probes the current system for Oh My Posh, Nerd Fonts, and basic platform
// metadata.  Everything is synchronous (`Command::new`) - no network.

use std::path::PathBuf;
use std::process::Command;

// ── OS info ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OsInfo {
    pub name: String,       // "Windows", "macOS", "Linux"
    pub version: String,    // e.g. "11 23H2", "14.2", "Ubuntu 22.04"
    pub is_wsl: bool,
}

pub fn detect_os() -> OsInfo {
    #[cfg(target_os = "windows")]
    {
        let version = Command::new("cmd")
            .args(["/c", "ver"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .trim()
            .to_string();
        OsInfo { name: "Windows".into(), version, is_wsl: false }
    }
    #[cfg(target_os = "macos")]
    {
        let version = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .trim()
            .to_string();
        OsInfo { name: "macOS".into(), version, is_wsl: false }
    }
    #[cfg(target_os = "linux")]
    {
        let is_wsl = std::fs::read_to_string("/proc/version")
            .map(|v| v.to_lowercase().contains("microsoft"))
            .unwrap_or(false);
        let version = Command::new("lsb_release")
            .arg("-ds")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|| "Linux".into())
            .trim()
            .trim_matches('"')
            .to_string();
        OsInfo { name: "Linux".into(), version, is_wsl }
    }
}

// ── Oh My Posh binary detection ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OmpInfo {
    pub version: String,
    pub executable: PathBuf,
    pub themes_path: Option<PathBuf>,
    pub cache_path: Option<PathBuf>,
}

/// Try to find `oh-my-posh` and get its version & paths.
pub fn detect_omp() -> Option<OmpInfo> {
    // Try getting version
    let output = Command::new("oh-my-posh")
        .arg("version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        return None;
    }

    // Resolve executable path
    let executable = resolve_executable().unwrap_or_else(|| PathBuf::from("oh-my-posh"));

    // Get themes path: first try POSH_THEMES_PATH env var
    let themes_path = std::env::var("POSH_THEMES_PATH").ok().map(PathBuf::from)
        .or_else(|| {
            // Fallback: next to the executable under ../themes or in cache
            let cache = get_cache_path();
            cache.as_ref().map(|c| c.join("themes"))
        })
        .filter(|p| p.is_dir());

    let cache_path = get_cache_path();

    Some(OmpInfo { version, executable, themes_path, cache_path })
}

/// Resolve the full path to the oh-my-posh binary.
fn resolve_executable() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        Command::new("where")
            .arg("oh-my-posh")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.lines().next().map(|l| PathBuf::from(l.trim())))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("which")
            .arg("oh-my-posh")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| PathBuf::from(s.trim()))
    }
}

/// Ask OMP for its cache path.
fn get_cache_path() -> Option<PathBuf> {
    Command::new("oh-my-posh")
        .args(["config", "get", "cache-path"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| !p.as_os_str().is_empty())
}

// ── Nerd Font detection ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FontStatus {
    /// Some known Nerd Font family names found on the system.
    pub installed_fonts: Vec<String>,
    /// `true` if at least one Nerd Font is detected.
    pub has_nerd_font: bool,
}

/// Detect installed Nerd Fonts.
/// On Windows uses PowerShell font enumeration; on Unix checks fc-list.
pub fn detect_fonts() -> FontStatus {
    let installed = list_nerd_fonts();
    let has = !installed.is_empty();
    FontStatus { installed_fonts: installed, has_nerd_font: has }
}

fn list_nerd_fonts() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        // PowerShell: query installed font families
        let script = r#"
            [System.Reflection.Assembly]::LoadWithPartialName('System.Drawing') | Out-Null
            $fonts = (New-Object System.Drawing.Text.InstalledFontCollection).Families
            $fonts | Where-Object { $_.Name -match 'Nerd|NF' } |
                     Select-Object -ExpandProperty Name
        "#;
        Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
            .unwrap_or_default()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("fc-list")
            .args([":", "family"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                s.lines()
                    .filter(|l| {
                        let low = l.to_lowercase();
                        low.contains("nerd") || low.contains(" nf")
                    })
                    .map(|l| l.trim().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ── Full detection report ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DetectionReport {
    pub os: OsInfo,
    pub omp: Option<OmpInfo>,
    pub fonts: FontStatus,
}

/// Run all detection probes and return a full report.
pub fn detect_all() -> DetectionReport {
    DetectionReport {
        os: detect_os(),
        omp: detect_omp(),
        fonts: detect_fonts(),
    }
}
