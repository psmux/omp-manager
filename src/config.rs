// ─── Oh My Posh JSON Configuration Parsing / Editing ─────────────────────────
//
// Handles reading, modifying, and saving Oh My Posh theme configuration files.
// The structures mirror the official JSON schema so we can round-trip edits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ── Top-level config ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OmpConfig {
    /// JSON schema URL (preserved for round-tripping).
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Final result indicator — the very last character of the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_space: Option<bool>,

    /// Terminal console title template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_title_template: Option<String>,

    /// Prompt blocks — the core of every theme.
    #[serde(default)]
    pub blocks: Vec<Block>,

    /// Transient prompt settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transient_prompt: Option<TransientPrompt>,

    /// Secondary prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_prompt: Option<SecondaryPrompt>,

    /// Tooltip segments (show on hover/tab-complete in supported shells).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tooltips: Vec<Segment>,

    /// Global palette — named color aliases.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub palette: HashMap<String, String>,

    /// Cycle through multiple palettes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub palettes: Vec<PaletteEntry>,

    /// Catch-all for unknown top-level keys.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ── Block ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// `"prompt"` | `"rprompt"` | `"newline"`
    #[serde(rename = "type")]
    pub block_type: String,

    /// `"left"` | `"right"`
    #[serde(default = "default_alignment")]
    pub alignment: String,

    /// Optional newline before this block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newline: Option<bool>,

    /// Segments within this block.
    #[serde(default)]
    pub segments: Vec<Segment>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_alignment() -> String { "left".into() }

// ── Segment ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    /// Segment type name (e.g. `"git"`, `"path"`, `"time"`).
    #[serde(rename = "type")]
    pub seg_type: String,

    /// Display style: `"powerline"` | `"plain"` | `"diamond"` | `"accordion"`
    #[serde(default = "default_style")]
    pub style: String,

    /// Foreground color (hex, palette name, or named color).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground: Option<String>,

    /// Background color.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,

    /// Foreground when segment is empty/inactive (diamond/accordion).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground_templates: Option<Vec<String>>,

    /// Background templates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_templates: Option<Vec<String>>,

    /// Powerline symbol (default `""` / nerd-font right-arrow).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub powerline_symbol: Option<String>,

    /// Invert powerline arrow direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invert_powerline: Option<bool>,

    /// Leading diamond character.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leading_diamond: Option<String>,

    /// Trailing diamond character.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_diamond: Option<String>,

    /// Template string for segment output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// Segment-specific properties.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, serde_json::Value>,

    /// Catch-all for unknown keys.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_style() -> String { "powerline".into() }

// ── Secondary structures ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientPrompt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryPrompt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub palette: HashMap<String, String>,
}

// ── File I/O ─────────────────────────────────────────────────────────────────

/// Load and parse a `.omp.json` theme file.
pub fn load_config(path: &Path) -> anyhow::Result<OmpConfig> {
    let content = std::fs::read_to_string(path)?;
    let cfg: OmpConfig = serde_json::from_str(&content)?;
    Ok(cfg)
}

/// Save an `OmpConfig` back to disk as pretty-printed JSON.
pub fn save_config(path: &Path, config: &OmpConfig) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)?;
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

impl OmpConfig {
    /// Count total segments across all blocks.
    pub fn total_segments(&self) -> usize {
        self.blocks.iter().map(|b| b.segments.len()).sum()
    }

    /// Get a flat list of all segment type names.
    pub fn segment_types(&self) -> Vec<String> {
        self.blocks
            .iter()
            .flat_map(|b| b.segments.iter().map(|s| s.seg_type.clone()))
            .collect()
    }
}

impl Block {
    /// Create a new empty prompt block.
    pub fn new_prompt(alignment: &str) -> Self {
        Block {
            block_type: "prompt".into(),
            alignment: alignment.into(),
            newline: None,
            segments: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

impl Segment {
    /// Create a minimal segment of the given type.
    pub fn new(seg_type: &str) -> Self {
        Segment {
            seg_type: seg_type.into(),
            style: "powerline".into(),
            foreground: Some("#ffffff".into()),
            background: Some("#61AFEF".into()),
            foreground_templates: None,
            background_templates: None,
            powerline_symbol: Some("\u{e0b0}".into()),
            invert_powerline: None,
            leading_diamond: None,
            trailing_diamond: None,
            template: None,
            properties: HashMap::new(),
            extra: HashMap::new(),
        }
    }
}

// ── User config path ─────────────────────────────────────────────────────────

/// Return a suitable path for the user's custom config.
/// `~/.config/omp-manager/my-theme.omp.json`
pub fn user_config_path() -> std::path::PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    config_dir.join("omp-manager").join("my-theme.omp.json")
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config_roundtrip() {
        let json = r##"{
            "$schema": "https://raw.githubusercontent.com/JanDeDobbeleer/oh-my-posh/main/themes/schema.json",
            "blocks": [
                {
                    "type": "prompt",
                    "alignment": "left",
                    "segments": [
                        {
                            "type": "path",
                            "style": "powerline",
                            "foreground": "#ffffff",
                            "background": "#61AFEF"
                        }
                    ]
                }
            ],
            "final_space": true
        }"##;
        let cfg: OmpConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.blocks.len(), 1);
        assert_eq!(cfg.blocks[0].segments.len(), 1);
        assert_eq!(cfg.blocks[0].segments[0].seg_type, "path");

        // Round-trip
        let out = serde_json::to_string_pretty(&cfg).unwrap();
        let cfg2: OmpConfig = serde_json::from_str(&out).unwrap();
        assert_eq!(cfg2.total_segments(), 1);
    }

    #[test]
    fn test_segment_new() {
        let seg = Segment::new("git");
        assert_eq!(seg.seg_type, "git");
        assert_eq!(seg.style, "powerline");
    }
}
