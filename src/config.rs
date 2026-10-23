//! Declarative configuration management for TIDE workspace.
//!
//! Loads settings from `./.tide.toml`, `./.tide/config.toml`,
//! or the user global configuration directory (`~/.config/tide/tide.toml` or `%APPDATA%\tide\config.toml`).

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    pub default_shell: Option<String>,
    pub font_zoom: Option<i8>,
}

fn default_theme() -> String {
    "TideOcean".to_string()
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            default_shell: None,
            font_zoom: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorktreeBootstrapConfig {
    #[serde(default = "default_copy_files")]
    pub copy_files: Vec<String>,
    #[serde(default = "default_true")]
    pub share_cargo_target: bool,
    #[serde(default = "default_true")]
    pub link_node_modules: bool,
}

fn default_copy_files() -> Vec<String> {
    vec![".env".to_string(), ".env.local".to_string()]
}

fn default_true() -> bool {
    true
}

impl Default for WorktreeBootstrapConfig {
    fn default() -> Self {
        Self {
            copy_files: default_copy_files(),
            share_cargo_target: true,
            link_node_modules: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorktreeConfig {
    #[serde(default)]
    pub bootstrap: WorktreeBootstrapConfig,
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            bootstrap: WorktreeBootstrapConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TideConfig {
    #[serde(default)]
    pub workspace: WorkspaceConfig,
    #[serde(default)]
    pub worktree: WorktreeConfig,
    #[serde(default)]
    pub agents: Vec<AgentConfig>,
}

impl TideConfig {
    /// Discovers and loads configuration from the workspace or global user directory.
    pub fn load(workspace_dir: &Path) -> Self {
        // Priority 1: ./.tide.toml in workspace
        let local_file = workspace_dir.join(".tide.toml");
        if let Ok(cfg) = Self::from_file(&local_file) {
            return cfg;
        }

        // Priority 2: ./.tide/config.toml
        let local_dir_file = workspace_dir.join(".tide").join("config.toml");
        if let Ok(cfg) = Self::from_file(&local_dir_file) {
            return cfg;
        }

        // Priority 3: Global user config
        if let Some(global_path) = Self::global_config_path() {
            if let Ok(cfg) = Self::from_file(&global_path) {
                return cfg;
            }
        }

        Self::default()
    }

    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let cfg: Self = toml::from_str(&content)?;
        Ok(cfg)
    }

    pub fn global_config_path() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return Some(PathBuf::from(appdata).join("tide").join("config.toml"));
            }
        }
        #[cfg(not(windows))]
        {
            if let Ok(home) = std::env::var("HOME") {
                return Some(PathBuf::from(home).join(".config").join("tide").join("tide.toml"));
            }
        }
        None
    }
}
