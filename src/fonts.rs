// ─── Nerd Font Catalog & Installation ────────────────────────────────────────
//
// Maintains a curated list of popular Nerd Fonts and drives installation
// through `oh-my-posh font install <name>`.

use std::process::Command;

// ── Font entry ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NerdFont {
    /// The name as passed to `oh-my-posh font install <name>`.
    pub name: &'static str,
    /// Human-readable display name.
    pub display: &'static str,
    /// Short description / style.
    pub description: &'static str,
    /// Whether this is a recommended default.
    pub recommended: bool,
}

// ── Curated font list ────────────────────────────────────────────────────────

pub static FONT_CATALOG: &[NerdFont] = &[
    NerdFont { name: "Meslo",          display: "MesloLGS Nerd Font",        description: "Recommended by Oh My Posh — clean and reliable",  recommended: true  },
    NerdFont { name: "FiraCode",       display: "FiraCode Nerd Font",        description: "Popular ligature-rich coding font",                recommended: true  },
    NerdFont { name: "CascadiaCode",   display: "Cascadia Code Nerd Font",   description: "Microsoft's default Windows Terminal font",        recommended: true  },
    NerdFont { name: "JetBrainsMono",  display: "JetBrains Mono Nerd Font",  description: "JetBrains' developer-friendly monospace",          recommended: false },
    NerdFont { name: "Hack",           display: "Hack Nerd Font",            description: "Classic clean hacking font",                       recommended: false },
    NerdFont { name: "SourceCodePro",  display: "Source Code Pro Nerd Font", description: "Adobe's open-source coding font",                  recommended: false },
    NerdFont { name: "UbuntuMono",     display: "Ubuntu Mono Nerd Font",     description: "Ubuntu's signature monospace",                     recommended: false },
    NerdFont { name: "RobotoMono",     display: "Roboto Mono Nerd Font",     description: "Google's Roboto family monospace",                 recommended: false },
    NerdFont { name: "Inconsolata",    display: "Inconsolata Nerd Font",     description: "Raph Levien's humanist monospace",                 recommended: false },
    NerdFont { name: "DejaVuSansMono", display: "DejaVu Sans Mono NF",      description: "Extended Unicode coverage",                        recommended: false },
    NerdFont { name: "Iosevka",        display: "Iosevka Nerd Font",         description: "Narrow, efficient coding font",                    recommended: false },
    NerdFont { name: "VictorMono",     display: "Victor Mono Nerd Font",     description: "Elegant with cursive italics",                     recommended: false },
    NerdFont { name: "IBMPlexMono",    display: "IBM Plex Mono Nerd Font",   description: "IBM's modern monospace family",                    recommended: false },
    NerdFont { name: "SpaceMono",      display: "Space Mono Nerd Font",      description: "Google Fonts geometric monospace",                 recommended: false },
    NerdFont { name: "Agave",          display: "Agave Nerd Font",           description: "Small, pleasant, rounded",                         recommended: false },
    NerdFont { name: "ComicShannsMono",display: "Comic Shanns Nerd Font",    description: "Playful Comic Sans alternative for code",          recommended: false },
    NerdFont { name: "Monaspace",      display: "Monaspace Nerd Font",       description: "GitHub's superfamily of coding fonts",             recommended: false },
    NerdFont { name: "GeistMono",      display: "Geist Mono Nerd Font",      description: "Vercel's modern monospace",                        recommended: false },
];

// ── Installation ─────────────────────────────────────────────────────────────

pub struct FontInstallResult {
    pub success: bool,
    pub message: String,
}

/// Install a Nerd Font using `oh-my-posh font install <name>`.
pub fn install_font(font_name: &str) -> FontInstallResult {
    let output = Command::new("oh-my-posh")
        .args(["font", "install", font_name])
        .output();

    match output {
        Ok(o) if o.status.success() => FontInstallResult {
            success: true,
            message: format!("Successfully installed {font_name} Nerd Font"),
        },
        Ok(o) => FontInstallResult {
            success: false,
            message: format!(
                "Font install failed: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            ),
        },
        Err(e) => FontInstallResult {
            success: false,
            message: format!("Could not run oh-my-posh: {e}"),
        },
    }
}
