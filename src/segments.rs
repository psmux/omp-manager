// ─── Oh My Posh Segment Catalog ──────────────────────────────────────────────
//
// A comprehensive catalog of every segment type supported by Oh My Posh,
// grouped into logical categories.  Used by the Config tab for the segment
// picker and by the preview renderer for icon/label lookups.

use serde::{Deserialize, Serialize};

// ── Segment category ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SegmentCategory {
    System,
    Scm,
    Language,
    Cloud,
    Shell,
    Music,
    Other,
}

impl SegmentCategory {
    pub const ALL: &'static [SegmentCategory] = &[
        Self::System,
        Self::Scm,
        Self::Language,
        Self::Cloud,
        Self::Shell,
        Self::Music,
        Self::Other,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::System   => "System",
            Self::Scm      => "Source Control",
            Self::Language  => "Languages",
            Self::Cloud     => "Cloud / DevOps",
            Self::Shell     => "Shell",
            Self::Music     => "Music / Activity",
            Self::Other     => "Other",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::System   => "🖥",
            Self::Scm      => "🔀",
            Self::Language  => "🗂",
            Self::Cloud     => "☁",
            Self::Shell     => "⌨",
            Self::Music     => "🎵",
            Self::Other     => "📋",
        }
    }
}

// ── Single catalog entry ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SegmentInfo {
    /// Oh My Posh segment `type` string (e.g. `"git"`, `"path"`).
    pub type_name: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// One-line description.
    pub description: &'static str,
    /// Category for grouping.
    pub category: SegmentCategory,
    /// Representative icon (Nerd Font glyph or emoji fallback).
    pub icon: &'static str,
    /// Example text shown in theme previews.
    pub example: &'static str,
}

// ── Full catalog ─────────────────────────────────────────────────────────────

pub static SEGMENT_CATALOG: &[SegmentInfo] = &[
    // ── System ───────────────────────────────────────────────────────────────
    SegmentInfo { type_name: "os",             label: "OS",               description: "Operating system icon",                  category: SegmentCategory::System, icon: "",  example: "" },
    SegmentInfo { type_name: "path",           label: "Path",             description: "Current working directory",               category: SegmentCategory::System, icon: "",  example: "~/projects/app" },
    SegmentInfo { type_name: "session",        label: "Session",          description: "User@host session info",                  category: SegmentCategory::System, icon: "",  example: "user@host" },
    SegmentInfo { type_name: "time",           label: "Time",             description: "Current time",                            category: SegmentCategory::System, icon: "",  example: "14:32" },
    SegmentInfo { type_name: "executiontime",  label: "Exec Time",        description: "Duration of last command",                category: SegmentCategory::System, icon: "",  example: "3s" },
    SegmentInfo { type_name: "battery",        label: "Battery",          description: "Battery percentage & status",             category: SegmentCategory::System, icon: "",  example: "85%" },
    SegmentInfo { type_name: "sysinfo",        label: "Sysinfo",         description: "CPU / memory usage",                      category: SegmentCategory::System, icon: "",  example: "45%" },
    SegmentInfo { type_name: "text",           label: "Text",            description: "Custom static text",                      category: SegmentCategory::System, icon: "",   example: "hello" },
    SegmentInfo { type_name: "exit",           label: "Exit Code",       description: "Previous command exit status",            category: SegmentCategory::System, icon: "✓",  example: "✓" },
    SegmentInfo { type_name: "root",           label: "Root",            description: "Root / admin indicator",                  category: SegmentCategory::System, icon: "⚡", example: "⚡" },
    SegmentInfo { type_name: "status",         label: "Status",          description: "Command status indicator",                category: SegmentCategory::System, icon: "✓",  example: "✓" },
    SegmentInfo { type_name: "upgrade",        label: "Upgrade",         description: "Oh My Posh update available",             category: SegmentCategory::System, icon: "⬆",  example: "⬆" },
    SegmentInfo { type_name: "project",        label: "Project",         description: "Current project name & version",          category: SegmentCategory::System, icon: "",  example: "myapp v1.2" },
    SegmentInfo { type_name: "ipify",          label: "Public IP",       description: "Public IP address via ipify",             category: SegmentCategory::System, icon: "",  example: "1.2.3.4" },
    SegmentInfo { type_name: "connection",     label: "Connection",      description: "Network connection type & SSID",          category: SegmentCategory::System, icon: "直", example: "WiFi" },
    SegmentInfo { type_name: "winreg",         label: "Win Registry",    description: "Windows Registry value",                  category: SegmentCategory::System, icon: "",  example: "value" },
    SegmentInfo { type_name: "owm",            label: "Weather (OWM)",   description: "OpenWeatherMap current weather",          category: SegmentCategory::System, icon: "",  example: "22°C" },

    // ── Source Control ───────────────────────────────────────────────────────
    SegmentInfo { type_name: "git",            label: "Git",             description: "Branch, status, stash, upstream",         category: SegmentCategory::Scm, icon: "",  example: " main" },
    SegmentInfo { type_name: "mercurial",      label: "Mercurial",       description: "Mercurial branch & status",               category: SegmentCategory::Scm, icon: "",  example: "default" },
    SegmentInfo { type_name: "svn",            label: "SVN",             description: "Subversion status",                       category: SegmentCategory::Scm, icon: "",  example: "trunk" },
    SegmentInfo { type_name: "plastic",        label: "Plastic SCM",    description: "Plastic SCM info",                        category: SegmentCategory::Scm, icon: "",  example: "main" },
    SegmentInfo { type_name: "fossil",         label: "Fossil",          description: "Fossil SCM info",                         category: SegmentCategory::Scm, icon: "",  example: "trunk" },
    SegmentInfo { type_name: "sapling",        label: "Sapling",         description: "Sapling SCM info",                        category: SegmentCategory::Scm, icon: "",  example: "main" },

    // ── Languages ────────────────────────────────────────────────────────────
    SegmentInfo { type_name: "angular",   label: "Angular",    description: "Angular CLI version",          category: SegmentCategory::Language, icon: "",  example: "17.0" },
    SegmentInfo { type_name: "bun",       label: "Bun",        description: "Bun runtime version",          category: SegmentCategory::Language, icon: "🥟", example: "1.0" },
    SegmentInfo { type_name: "crystal",   label: "Crystal",    description: "Crystal language version",     category: SegmentCategory::Language, icon: "",  example: "1.10" },
    SegmentInfo { type_name: "dart",      label: "Dart",       description: "Dart SDK version",             category: SegmentCategory::Language, icon: "",  example: "3.2" },
    SegmentInfo { type_name: "deno",      label: "Deno",       description: "Deno runtime version",         category: SegmentCategory::Language, icon: "🦕", example: "1.38" },
    SegmentInfo { type_name: "dotnet",    label: ".NET",       description: ".NET SDK version",             category: SegmentCategory::Language, icon: "",  example: "8.0" },
    SegmentInfo { type_name: "elixir",    label: "Elixir",     description: "Elixir version",               category: SegmentCategory::Language, icon: "",  example: "1.15" },
    SegmentInfo { type_name: "flutter",   label: "Flutter",    description: "Flutter SDK version",          category: SegmentCategory::Language, icon: "",  example: "3.16" },
    SegmentInfo { type_name: "go",        label: "Go",         description: "Go version",                   category: SegmentCategory::Language, icon: "",  example: "1.21" },
    SegmentInfo { type_name: "haskell",   label: "Haskell",    description: "GHC / Stack version",          category: SegmentCategory::Language, icon: "",  example: "9.6" },
    SegmentInfo { type_name: "java",      label: "Java",       description: "Java / JDK version",           category: SegmentCategory::Language, icon: "",  example: "21" },
    SegmentInfo { type_name: "julia",     label: "Julia",      description: "Julia language version",       category: SegmentCategory::Language, icon: "",  example: "1.10" },
    SegmentInfo { type_name: "kotlin",    label: "Kotlin",     description: "Kotlin version",               category: SegmentCategory::Language, icon: "",  example: "1.9" },
    SegmentInfo { type_name: "lua",       label: "Lua",        description: "Lua version",                  category: SegmentCategory::Language, icon: "",  example: "5.4" },
    SegmentInfo { type_name: "node",      label: "Node.js",    description: "Node.js version",              category: SegmentCategory::Language, icon: "",  example: "20.9" },
    SegmentInfo { type_name: "ocaml",     label: "OCaml",      description: "OCaml version",                category: SegmentCategory::Language, icon: "",  example: "5.1" },
    SegmentInfo { type_name: "perl",      label: "Perl",       description: "Perl version",                 category: SegmentCategory::Language, icon: "",  example: "5.38" },
    SegmentInfo { type_name: "php",       label: "PHP",        description: "PHP version",                  category: SegmentCategory::Language, icon: "",  example: "8.3" },
    SegmentInfo { type_name: "python",    label: "Python",     description: "Python version & virtualenv",  category: SegmentCategory::Language, icon: "",  example: "3.12" },
    SegmentInfo { type_name: "r",         label: "R",          description: "R language version",            category: SegmentCategory::Language, icon: "📊", example: "4.3" },
    SegmentInfo { type_name: "ruby",      label: "Ruby",       description: "Ruby version",                 category: SegmentCategory::Language, icon: "",  example: "3.3" },
    SegmentInfo { type_name: "rust",      label: "Rust",       description: "Rust toolchain version",       category: SegmentCategory::Language, icon: "",  example: "1.74" },
    SegmentInfo { type_name: "scala",     label: "Scala",      description: "Scala version",                category: SegmentCategory::Language, icon: "",  example: "3.3" },
    SegmentInfo { type_name: "swift",     label: "Swift",      description: "Swift version",                category: SegmentCategory::Language, icon: "",  example: "5.9" },
    SegmentInfo { type_name: "zig",       label: "Zig",        description: "Zig version",                  category: SegmentCategory::Language, icon: "",  example: "0.11" },
    SegmentInfo { type_name: "cmake",     label: "CMake",      description: "CMake version",                category: SegmentCategory::Language, icon: "△",  example: "3.28" },
    SegmentInfo { type_name: "nim",       label: "Nim",        description: "Nim version",                  category: SegmentCategory::Language, icon: "👑", example: "2.0" },
    SegmentInfo { type_name: "mojo",      label: "Mojo",       description: "Mojo version",                 category: SegmentCategory::Language, icon: "🔥", example: "0.6" },
    SegmentInfo { type_name: "v",         label: "V",          description: "V language version",            category: SegmentCategory::Language, icon: "V",  example: "0.4" },
    SegmentInfo { type_name: "fortran",   label: "Fortran",    description: "Fortran compiler version",     category: SegmentCategory::Language, icon: "F",  example: "13.2" },

    // ── Cloud / DevOps ───────────────────────────────────────────────────────
    SegmentInfo { type_name: "aws",        label: "AWS",         description: "AWS profile & region",            category: SegmentCategory::Cloud, icon: "",  example: "prod" },
    SegmentInfo { type_name: "az",         label: "Azure",       description: "Azure subscription info",         category: SegmentCategory::Cloud, icon: "󰠅",  example: "my-sub" },
    SegmentInfo { type_name: "gcp",        label: "GCP",         description: "Google Cloud project",            category: SegmentCategory::Cloud, icon: "",  example: "my-proj" },
    SegmentInfo { type_name: "docker",     label: "Docker",      description: "Docker context",                  category: SegmentCategory::Cloud, icon: "",  example: "default" },
    SegmentInfo { type_name: "kubectl",    label: "Kubernetes",  description: "K8s context & namespace",         category: SegmentCategory::Cloud, icon: "󱃾",  example: "prod" },
    SegmentInfo { type_name: "helm",       label: "Helm",        description: "Helm version",                    category: SegmentCategory::Cloud, icon: "",  example: "3.13" },
    SegmentInfo { type_name: "terraform",  label: "Terraform",   description: "Terraform workspace & version",   category: SegmentCategory::Cloud, icon: "󱁢",  example: "default" },
    SegmentInfo { type_name: "pulumi",     label: "Pulumi",      description: "Pulumi stack name",               category: SegmentCategory::Cloud, icon: "",  example: "dev" },
    SegmentInfo { type_name: "cf",         label: "Cloud F.",    description: "Cloud Foundry target",            category: SegmentCategory::Cloud, icon: "",  example: "api" },
    SegmentInfo { type_name: "heroku",     label: "Heroku",      description: "Heroku app info",                 category: SegmentCategory::Cloud, icon: "",  example: "myapp" },
    SegmentInfo { type_name: "firebase",   label: "Firebase",    description: "Firebase project",                category: SegmentCategory::Cloud, icon: "",  example: "proj" },
    SegmentInfo { type_name: "argocd",     label: "ArgoCD",      description: "ArgoCD context",                  category: SegmentCategory::Cloud, icon: "",  example: "prod" },
    SegmentInfo { type_name: "buf",        label: "Buf",         description: "Buf version",                     category: SegmentCategory::Cloud, icon: "",  example: "1.28" },
    SegmentInfo { type_name: "nx",         label: "Nx",          description: "Nx workspace info",               category: SegmentCategory::Cloud, icon: "",  example: "myws" },
    SegmentInfo { type_name: "bazel",      label: "Bazel",       description: "Bazel version",                   category: SegmentCategory::Cloud, icon: "",  example: "7.0" },
    SegmentInfo { type_name: "cds",        label: "SAP CDS",     description: "SAP CDS version",                 category: SegmentCategory::Cloud, icon: "",  example: "7.4" },
    SegmentInfo { type_name: "npm",        label: "npm",         description: "npm package version",              category: SegmentCategory::Cloud, icon: "",  example: "10.2" },

    // ── Shell ────────────────────────────────────────────────────────────────
    SegmentInfo { type_name: "shell",      label: "Shell",       description: "Current shell name",              category: SegmentCategory::Shell, icon: "",  example: "pwsh" },
    SegmentInfo { type_name: "commander",  label: "Commander",   description: "Prompt character / command mode",  category: SegmentCategory::Shell, icon: "❯",  example: "❯" },

    // ── Music / Activity ─────────────────────────────────────────────────────
    SegmentInfo { type_name: "spotify",    label: "Spotify",     description: "Currently playing track",         category: SegmentCategory::Music, icon: "",  example: "♫ Song" },
    SegmentInfo { type_name: "lastfm",     label: "Last.fm",     description: "Last.fm scrobble info",           category: SegmentCategory::Music, icon: "",  example: "♫ Track" },
    SegmentInfo { type_name: "strava",     label: "Strava",      description: "Strava activity info",            category: SegmentCategory::Music, icon: "",  example: "5km" },
    SegmentInfo { type_name: "wakatime",   label: "WakaTime",    description: "WakaTime coding stats",           category: SegmentCategory::Music, icon: "",  example: "3h" },
    SegmentInfo { type_name: "withings",   label: "Withings",    description: "Withings health data",            category: SegmentCategory::Music, icon: "❤",  example: "72bpm" },
    SegmentInfo { type_name: "ytm",        label: "YouTube Music", description: "YouTube Music now playing",     category: SegmentCategory::Music, icon: "",  example: "♫ Now" },
    SegmentInfo { type_name: "nba",        label: "NBA",         description: "NBA game scores",                 category: SegmentCategory::Music, icon: "🏀", example: "LAL 110" },

    // ── Other ────────────────────────────────────────────────────────────────
    SegmentInfo { type_name: "brewfather",  label: "Brewfather",  description: "Brewfather batch status",        category: SegmentCategory::Other, icon: "🍺", example: "Batch" },
    SegmentInfo { type_name: "nbgv",        label: "NBGV",       description: "Nerdbank.GitVersioning version",  category: SegmentCategory::Other, icon: "",  example: "1.0.0" },
    SegmentInfo { type_name: "unity",       label: "Unity",       description: "Unity editor version",            category: SegmentCategory::Other, icon: "",  example: "2023.2" },
    SegmentInfo { type_name: "xmake",       label: "xmake",       description: "xmake build system version",     category: SegmentCategory::Other, icon: "",  example: "2.8" },
    SegmentInfo { type_name: "quasar",      label: "Quasar",      description: "Quasar framework version",       category: SegmentCategory::Other, icon: "",  example: "2.14" },
    SegmentInfo { type_name: "react",       label: "React",       description: "React version",                   category: SegmentCategory::Other, icon: "",  example: "18.2" },
    SegmentInfo { type_name: "talosctl",    label: "Talos",       description: "Talos Linux context",             category: SegmentCategory::Other, icon: "",  example: "ctx" },
    SegmentInfo { type_name: "vfox",        label: "vfox",        description: "vfox version manager",            category: SegmentCategory::Other, icon: "🦊", example: "0.3" },
    SegmentInfo { type_name: "azfunc",      label: "Azure Func",  description: "Azure Functions version",         category: SegmentCategory::Other, icon: "󰡯",  example: "4" },
    SegmentInfo { type_name: "umbraco",     label: "Umbraco",     description: "Umbraco CMS version",             category: SegmentCategory::Other, icon: "",  example: "13" },
];

/// Look up a segment by its `type` name.
pub fn lookup_segment(type_name: &str) -> Option<&'static SegmentInfo> {
    SEGMENT_CATALOG.iter().find(|s| s.type_name == type_name)
}

/// Return all segments belonging to a category.
pub fn segments_in_category(cat: SegmentCategory) -> Vec<&'static SegmentInfo> {
    SEGMENT_CATALOG.iter().filter(|s| s.category == cat).collect()
}
