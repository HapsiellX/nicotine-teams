// Per-character preview window positions, persisted to disk so previews
// come back where the user left them across daemon restarts. Schema is
// platform-agnostic; both the Windows DWM thumbnail manager and the
// Linux XComposite manager share this file at
// `~/.config/nicotine/preview_positions.toml`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PreviewPositions {
    #[serde(default)]
    pub positions: HashMap<String, (i32, i32)>,
    /// Last-known position of the single list-view window (Display Mode =
    /// List). Saved on drag-end, restored on window (re)create. Without
    /// this, toggling display mode would reset the list window to its
    /// default spawn position every time. Currently Windows-only —
    /// Linux ships List mode in a later phase.
    #[serde(default)]
    pub list_position: Option<(i32, i32)>,
}

impl PreviewPositions {
    fn config_path() -> PathBuf {
        let mut p = crate::config::Config::config_dir();
        p.push("preview_positions.toml");
        p
    }

    pub fn load_checked() -> Result<Self> {
        match std::fs::read_to_string(Self::config_path()) {
            Ok(contents) => toml::from_str(&contents).context("Ungültiges Vorschau-Layout"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn load() -> Self {
        Self::load_checked().unwrap_or_else(|error| {
            eprintln!("Vorschau-Layout konnte nicht geladen werden: {error}");
            Self::default()
        })
    }

    pub fn save_checked(&self) -> Result<()> {
        crate::storage::atomic_write(
            &Self::config_path(),
            toml::to_string_pretty(self)?.as_bytes(),
        )
    }

    pub fn save(&self) {
        if let Err(error) = self.save_checked() {
            eprintln!("Vorschau-Layout konnte nicht gespeichert werden: {error}");
        }
    }

    pub fn get(&self, name: &str) -> Option<(i32, i32)> {
        self.positions.get(name).copied()
    }

    pub fn set(&mut self, name: String, x: i32, y: i32) {
        self.positions.insert(name, (x, y));
    }
}
