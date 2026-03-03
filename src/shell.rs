// ─── Shell Profile Management ────────────────────────────────────────────────
//
// Detects shells, resolves their RC/profile file paths, and reads / writes
// the `oh-my-posh init …` line in each config.

use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::process::Command;

// ── Shell enum ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shell {
    Pwsh,             // PowerShell 7+
    WindowsPowerShell,// Windows PowerShell 5.1
    Bash,
    Zsh,
    Fish,
    Nushell,
    Cmd,
    Elvish,
}

impl Shell {
    pub const ALL: &'static [Shell] = &[
        Self::Pwsh,
        Self::WindowsPowerShell,
        Self::Bash,
        Self::Zsh,
        Self::Fish,
        Self::Nushell,
        Self::Cmd,
        Self::Elvish,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Pwsh              => "PowerShell 7+",
            Self::WindowsPowerShell => "Windows PowerShell",
            Self::Bash              => "Bash",
            Self::Zsh               => "Zsh",
            Self::Fish              => "Fish",
            Self::Nushell           => "Nushell",
            Self::Cmd               => "Cmd (via Clink)",
            Self::Elvish            => "Elvish",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pwsh | Self::WindowsPowerShell => "",
            Self::Bash              => "",
            Self::Zsh               => "",
            Self::Fish              => "",
            Self::Nushell           => ">",
            Self::Cmd               => "",
            Self::Elvish            => "λ",
        }
    }

    /// The shell name as expected by `oh-my-posh init <shell>`.
    pub fn omp_name(&self) -> &'static str {
        match self {
            Self::Pwsh | Self::WindowsPowerShell => "pwsh",
            Self::Bash    => "bash",
            Self::Zsh     => "zsh",
            Self::Fish    => "fish",
            Self::Nushell => "nu",
            Self::Cmd     => "cmd",
            Self::Elvish  => "elvish",
        }
    }
}

// ── Shell info (detection result) ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ShellInfo {
    pub shell: Shell,
    /// Whether the shell binary was found on PATH.
    pub available: bool,
    /// Resolved path to the profile / RC file (may not exist yet).
    pub profile_path: PathBuf,
    /// Whether the profile file already contains an oh-my-posh init line.
    pub omp_configured: bool,
    /// The theme path currently configured in the init line (if any).
    pub current_theme: Option<String>,
}

// ── Profile path resolution ──────────────────────────────────────────────────

/// Return the default profile file path for a given shell.
pub fn profile_path(shell: Shell) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    match shell {
        Shell::Pwsh => {
            // Cross-platform PowerShell 7: ~/Documents/PowerShell/Microsoft.PowerShell_profile.ps1
            #[cfg(target_os = "windows")]
            {
                let docs = dirs::document_dir().unwrap_or_else(|| home.join("Documents"));
                docs.join("PowerShell").join("Microsoft.PowerShell_profile.ps1")
            }
            #[cfg(not(target_os = "windows"))]
            {
                home.join(".config").join("powershell").join("Microsoft.PowerShell_profile.ps1")
            }
        }
        Shell::WindowsPowerShell => {
            let docs = dirs::document_dir().unwrap_or_else(|| home.join("Documents"));
            docs.join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1")
        }
        Shell::Bash => home.join(".bashrc"),
        Shell::Zsh  => home.join(".zshrc"),
        Shell::Fish => home.join(".config").join("fish").join("config.fish"),
        Shell::Nushell => {
            #[cfg(target_os = "windows")]
            { home.join("AppData").join("Roaming").join("nushell").join("config.nu") }
            #[cfg(not(target_os = "windows"))]
            { home.join(".config").join("nushell").join("config.nu") }
        }
        Shell::Cmd => {
            // Clink Lua script for cmd
            home.join(".config").join("clink").join("oh-my-posh.lua")
        }
        Shell::Elvish => home.join(".config").join("elvish").join("rc.elv"),
    }
}

// ── Init command generation ──────────────────────────────────────────────────

/// Build the init line that should appear in the shell's profile.
/// `theme_path` is the absolute path to a `.omp.json` theme file (or `None`
/// for the default theme).
pub fn init_command(shell: Shell, theme_path: Option<&str>) -> String {
    let cfg_flag = match theme_path {
        Some(p) => format!(" --config '{}'", p),
        None    => String::new(),
    };
    match shell {
        Shell::Pwsh | Shell::WindowsPowerShell => {
            format!("oh-my-posh init pwsh{} | Invoke-Expression", cfg_flag)
        }
        Shell::Bash => {
            format!("eval \"$(oh-my-posh init bash{})\"", cfg_flag)
        }
        Shell::Zsh => {
            format!("eval \"$(oh-my-posh init zsh{})\"", cfg_flag)
        }
        Shell::Fish => {
            format!("oh-my-posh init fish{} | source", cfg_flag)
        }
        Shell::Nushell => {
            // Nushell requires sourcing from env.nu; simplified single-line
            format!("oh-my-posh init nu{}", cfg_flag)
        }
        Shell::Cmd => {
            format!("load(io.popen('oh-my-posh init cmd{}'):read(\"*a\"))()", cfg_flag)
        }
        Shell::Elvish => {
            format!("eval (oh-my-posh init elvish{})", cfg_flag)
        }
    }
}

// ── Profile reading / writing ────────────────────────────────────────────────

/// Marker we insert so we can find our own line later.
const OMP_MARKER: &str = "# [omp-manager]";

/// Read a profile file and determine if it already has an OMP init line.
/// Returns (is_configured, current_theme_path).
pub fn parse_profile(profile: &std::path::Path) -> (bool, Option<String>) {
    let Ok(content) = std::fs::read_to_string(profile) else {
        return (false, None);
    };
    for line in content.lines() {
        if line.contains("oh-my-posh") && line.contains("init") {
            // Extract --config '<path>' if present
            let theme = extract_config_path(line);
            return (true, theme);
        }
    }
    (false, None)
}

/// Extract the `--config '...'` or `--config "..."` value from an init line.
fn extract_config_path(line: &str) -> Option<String> {
    let re = regex::Regex::new(r#"--config\s+['"](.*?)['"]"#).ok()?;
    re.captures(line).map(|c| c[1].to_string())
}

/// Add or update the oh-my-posh init line in a shell profile.
/// Creates the file (and parent dirs) if it doesn't exist.
pub fn write_init_to_profile(
    shell: Shell,
    profile: &std::path::Path,
    theme_path: Option<&str>,
) -> anyhow::Result<()> {
    let init_line = init_command(shell, theme_path);
    let tagged = format!("{init_line}  {OMP_MARKER}");

    // Ensure parent directory exists
    if let Some(parent) = profile.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = std::fs::read_to_string(profile).unwrap_or_default();

    // Remove any existing OMP init lines
    let mut lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.contains("oh-my-posh") || !l.contains("init"))
        .collect();

    lines.push(&tagged);
    let new_content = lines.join("\n") + "\n";
    std::fs::write(profile, new_content)?;
    Ok(())
}

/// Remove the oh-my-posh init line from a shell profile.
pub fn remove_init_from_profile(profile: &std::path::Path) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(profile)?;
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| !(l.contains("oh-my-posh") && l.contains("init")))
        .collect();
    let new_content = lines.join("\n") + "\n";
    std::fs::write(profile, new_content)?;
    Ok(())
}

// ── Shell availability detection ─────────────────────────────────────────────

/// Check whether a shell binary is available on the system.
pub fn is_shell_available(shell: Shell) -> bool {
    let bin = match shell {
        Shell::Pwsh              => "pwsh",
        Shell::WindowsPowerShell => "powershell",
        Shell::Bash              => "bash",
        Shell::Zsh               => "zsh",
        Shell::Fish              => "fish",
        Shell::Nushell           => "nu",
        Shell::Cmd               => {
            #[cfg(target_os = "windows")]
            { return true; } // cmd always exists on Windows
            #[cfg(not(target_os = "windows"))]
            { return false; }
        }
        Shell::Elvish            => "elvish",
    };
    which_exists(bin)
}

/// Simple PATH-based check (no version parsing needed here).
fn which_exists(bin: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        Command::new("where")
            .arg(bin)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("which")
            .arg(bin)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Detect all shells and return a list of ShellInfo.
pub fn detect_all_shells() -> Vec<ShellInfo> {
    let mut shells = Vec::new();
    for &s in Shell::ALL {
        // Skip WindowsPowerShell on non-Windows
        #[cfg(not(target_os = "windows"))]
        if s == Shell::WindowsPowerShell || s == Shell::Cmd {
            continue;
        }
        let available = is_shell_available(s);
        let path = profile_path(s);
        let (configured, theme) = if path.exists() {
            parse_profile(&path)
        } else {
            (false, None)
        };
        shells.push(ShellInfo {
            shell: s,
            available,
            profile_path: path,
            omp_configured: configured,
            current_theme: theme,
        });
    }
    shells
}
