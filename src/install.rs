// ─── Oh My Posh Installation & Shell Configuration ───────────────────────────
//
// Platform-specific installation of Oh My Posh and automatic shell profile
// configuration.  Designed for one-click setup by non-technical users.

use std::process::Command;

use crate::shell::ShellInfo;

// ── Result type ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OpResult {
    pub success: bool,
    pub message: String,
}

// ── Oh My Posh Installation ──────────────────────────────────────────────────

/// Install Oh My Posh using the best method for the current platform.
pub fn install_omp() -> OpResult {
    #[cfg(target_os = "windows")]
    { install_omp_windows() }
    #[cfg(target_os = "macos")]
    { install_omp_macos() }
    #[cfg(target_os = "linux")]
    { install_omp_linux() }
}

#[cfg(target_os = "windows")]
fn install_omp_windows() -> OpResult {
    // Try winget first
    let result = Command::new("winget")
        .args(["install", "JanDeDobbeleer.OhMyPosh", "--source", "winget", "--accept-package-agreements", "--accept-source-agreements"])
        .output();

    match result {
        Ok(o) if o.status.success() => OpResult {
            success: true,
            message: "Oh My Posh installed via winget".into(),
        },
        _ => {
            // Fallback: PowerShell install script
            let result2 = Command::new("powershell")
                .args([
                    "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command",
                    "Set-ExecutionPolicy Bypass -Scope Process -Force; Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://ohmyposh.dev/install.ps1'))"
                ])
                .output();
            match result2 {
                Ok(o) if o.status.success() => OpResult {
                    success: true,
                    message: "Oh My Posh installed via install script".into(),
                },
                Ok(o) => OpResult {
                    success: false,
                    message: format!("Install failed: {}", String::from_utf8_lossy(&o.stderr).trim()),
                },
                Err(e) => OpResult {
                    success: false,
                    message: format!("Could not run installer: {e}"),
                },
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn install_omp_macos() -> OpResult {
    let result = Command::new("brew")
        .args(["install", "jandedobbeleer/oh-my-posh/oh-my-posh"])
        .output();
    match result {
        Ok(o) if o.status.success() => OpResult {
            success: true,
            message: "Oh My Posh installed via Homebrew".into(),
        },
        Ok(o) => OpResult {
            success: false,
            message: format!("brew install failed: {}", String::from_utf8_lossy(&o.stderr).trim()),
        },
        Err(_) => {
            // Homebrew not installed; try manual
            let result2 = Command::new("bash")
                .args(["-c", "curl -s https://ohmyposh.dev/install.sh | bash -s"])
                .output();
            match result2 {
                Ok(o) if o.status.success() => OpResult {
                    success: true,
                    message: "Oh My Posh installed via install script".into(),
                },
                Ok(o) => OpResult { success: false, message: format!("Install failed: {}", String::from_utf8_lossy(&o.stderr).trim()) },
                Err(e) => OpResult { success: false, message: format!("Could not run installer: {e}") },
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn install_omp_linux() -> OpResult {
    let result = Command::new("bash")
        .args(["-c", "curl -s https://ohmyposh.dev/install.sh | bash -s"])
        .output();
    match result {
        Ok(o) if o.status.success() => OpResult {
            success: true,
            message: "Oh My Posh installed via install script".into(),
        },
        Ok(o) => OpResult {
            success: false,
            message: format!("Install failed: {}", String::from_utf8_lossy(&o.stderr).trim()),
        },
        Err(e) => OpResult {
            success: false,
            message: format!("Could not run installer: {e}"),
        },
    }
}

// ── Update ───────────────────────────────────────────────────────────────────

/// Update Oh My Posh to the latest version.
pub fn update_omp() -> OpResult {
    #[cfg(target_os = "windows")]
    {
        let result = Command::new("winget")
            .args(["upgrade", "JanDeDobbeleer.OhMyPosh", "--source", "winget"])
            .output();
        match result {
            Ok(o) if o.status.success() => OpResult { success: true, message: "Oh My Posh updated".into() },
            _ => OpResult { success: false, message: "Update failed - try manually: winget upgrade JanDeDobbeleer.OhMyPosh".into() },
        }
    }
    #[cfg(target_os = "macos")]
    {
        let result = Command::new("brew")
            .args(["upgrade", "oh-my-posh"])
            .output();
        match result {
            Ok(o) if o.status.success() => OpResult { success: true, message: "Oh My Posh updated".into() },
            _ => OpResult { success: false, message: "Update failed - try: brew upgrade oh-my-posh".into() },
        }
    }
    #[cfg(target_os = "linux")]
    {
        let result = Command::new("bash")
            .args(["-c", "curl -s https://ohmyposh.dev/install.sh | bash -s"])
            .output();
        match result {
            Ok(o) if o.status.success() => OpResult { success: true, message: "Oh My Posh updated".into() },
            _ => OpResult { success: false, message: "Update failed - try: curl -s https://ohmyposh.dev/install.sh | bash -s".into() },
        }
    }
}

// ── Shell configuration ──────────────────────────────────────────────────────

/// Configure a shell to use Oh My Posh with the given theme.
pub fn configure_shell(shell_info: &ShellInfo, theme_path: Option<&str>) -> OpResult {
    let path = &shell_info.profile_path;
    match crate::shell::write_init_to_profile(shell_info.shell, path, theme_path) {
        Ok(()) => OpResult {
            success: true,
            message: format!("{} configured - restart your shell to see the new prompt", shell_info.shell.label()),
        },
        Err(e) => OpResult {
            success: false,
            message: format!("Failed to update {}: {e}", path.display()),
        },
    }
}

/// Remove Oh My Posh configuration from a shell.
pub fn unconfigure_shell(shell_info: &ShellInfo) -> OpResult {
    let path = &shell_info.profile_path;
    match crate::shell::remove_init_from_profile(path) {
        Ok(()) => OpResult {
            success: true,
            message: format!("{} unconfigured", shell_info.shell.label()),
        },
        Err(e) => OpResult {
            success: false,
            message: format!("Failed to update {}: {e}", path.display()),
        },
    }
}

// ── Quick Setup (all-in-one) ─────────────────────────────────────────────────

/// Represents the state of a setup wizard step.
#[derive(Debug, Clone)]
pub struct SetupStep {
    pub label: String,
    pub description: String,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Done,
    Failed,
    Skipped,
}

impl StepStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pending    => "○",
            Self::InProgress => "◉",
            Self::Done       => "✓",
            Self::Failed     => "✗",
            Self::Skipped    => "⊘",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending    => "Pending",
            Self::InProgress => "In progress…",
            Self::Done       => "Complete",
            Self::Failed     => "Failed",
            Self::Skipped    => "Skipped",
        }
    }
}

/// Create the initial setup wizard steps.
pub fn create_setup_steps(omp_installed: bool, has_font: bool) -> Vec<SetupStep> {
    vec![
        SetupStep {
            label: "Install Oh My Posh".into(),
            description: "Download and install the Oh My Posh binary".into(),
            status: if omp_installed { StepStatus::Done } else { StepStatus::Pending },
        },
        SetupStep {
            label: "Install a Nerd Font".into(),
            description: "Icons require a Nerd Font - we'll install one for you".into(),
            status: if has_font { StepStatus::Done } else { StepStatus::Pending },
        },
        SetupStep {
            label: "Choose a Theme".into(),
            description: "Pick a prompt theme that suits your style".into(),
            status: StepStatus::Pending,
        },
        SetupStep {
            label: "Configure Shell(s)".into(),
            description: "Add Oh My Posh to your shell profile so it loads automatically".into(),
            status: StepStatus::Pending,
        },
    ]
}
