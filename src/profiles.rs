//! Portable team snapshots. Input device paths and machine-specific panel geometry stay local.
use crate::config::{CharacterHotkey, Config, DisplayMode};
use crate::preview_positions::PreviewPositions;
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamProfiles {
    pub active: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, TeamProfile>,
    #[serde(default)]
    pub revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TeamProfile {
    pub characters: Vec<String>,
    pub character_hotkeys: HashMap<String, CharacterHotkey>,
    pub forward_key: u16,
    pub backward_key: u16,
    pub modifier_key: Option<u16>,
    pub forward_button: u16,
    pub backward_button: u16,
    pub enable_keyboard_buttons: bool,
    pub enable_mouse_buttons: bool,
    pub minimize_inactive: bool,
    pub toggle_previews_key: Option<u16>,
    pub toggle_previews_modifier: Option<u16>,
    pub preview_width: u32,
    pub preview_height: u32,
    pub preview_opacity: u32,
    #[serde(default = "crate::config::default_preview_fps")]
    pub preview_fps: u32,
    pub display_mode: DisplayMode,
    pub show_previews: bool,
    pub hide_active_preview: bool,
    pub positions_locked: bool,
    pub constrain_aspect: bool,
    pub layout: PreviewPositions,
}

impl TeamProfile {
    pub fn capture(config: &Config, layout: PreviewPositions) -> Self {
        Self {
            characters: config.characters.clone(),
            character_hotkeys: config.character_hotkeys.clone(),
            forward_key: config.forward_key,
            backward_key: config.backward_key,
            modifier_key: config.modifier_key,
            forward_button: config.forward_button,
            backward_button: config.backward_button,
            enable_keyboard_buttons: config.enable_keyboard_buttons,
            enable_mouse_buttons: config.enable_mouse_buttons,
            minimize_inactive: config.minimize_inactive,
            toggle_previews_key: config.toggle_previews_key,
            toggle_previews_modifier: config.toggle_previews_modifier,
            preview_width: config.preview_width,
            preview_height: config.preview_height,
            preview_opacity: config.preview_opacity,
            preview_fps: config.preview_fps,
            display_mode: config.display_mode,
            show_previews: config.show_previews,
            hide_active_preview: config.hide_active_preview,
            positions_locked: config.positions_locked,
            constrain_aspect: config.constrain_aspect,
            layout,
        }
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.characters.len() <= 100,
            "Höchstens 100 Charaktere pro Profil"
        );
        let mut names = HashSet::new();
        for name in &self.characters {
            validate_name(name)?;
            ensure!(names.insert(name), "Doppelter Charakter: {name}");
        }
        ensure!(
            (120..=800).contains(&self.preview_width),
            "Vorschaubreite außerhalb 120–800 px"
        );
        ensure!(
            (80..=600).contains(&self.preview_height),
            "Vorschauhöhe außerhalb 80–600 px"
        );
        ensure!(
            (10..=100).contains(&self.preview_opacity),
            "Deckkraft außerhalb 10–100 %"
        );
        for name in self.character_hotkeys.keys() {
            ensure!(names.contains(name), "Hotkey gehört nicht zum Team: {name}");
        }
        ensure!(
            (5..=60).contains(&self.preview_fps),
            "Vorschau-FPS außerhalb 5–60"
        );
        // Modifier + key combinations must target only one action.
        let mut bindings = HashSet::new();
        let backward_modifier = if self.forward_key == self.backward_key {
            self.modifier_key
        } else {
            None
        };
        if self.enable_keyboard_buttons {
            ensure!(
                self.forward_key != 0 && self.backward_key != 0,
                "Leere Wechseltaste"
            );
            bindings.insert((self.forward_key, None));
            ensure!(
                bindings.insert((self.backward_key, backward_modifier)),
                "Vorwärts-/Rückwärts-Hotkey kollidieren"
            );
        }
        for hotkey in self.character_hotkeys.values().filter(|h| h.vk != 0) {
            ensure!(
                bindings.insert((hotkey.vk, hotkey.modifier)),
                "Doppelte oder kollidierende Hotkeys"
            );
        }
        if let Some(key) = self.toggle_previews_key {
            ensure!(
                bindings.insert((key, self.toggle_previews_modifier)),
                "Vorschau-Hotkey kollidiert"
            );
        }
        for (name, (x, y)) in &self.layout.positions {
            validate_name(name)?;
            validate_position(*x, *y)?;
        }
        if let Some((x, y)) = self.layout.list_position {
            validate_position(x, y)?;
        }
        Ok(())
    }

    pub fn apply_settings(&self, config: &mut Config) {
        config.characters = self.characters.clone();
        config.character_hotkeys = self.character_hotkeys.clone();
        config.forward_key = self.forward_key;
        config.backward_key = self.backward_key;
        config.modifier_key = self.modifier_key;
        config.forward_button = self.forward_button;
        config.backward_button = self.backward_button;
        config.enable_keyboard_buttons = self.enable_keyboard_buttons;
        config.enable_mouse_buttons = self.enable_mouse_buttons;
        config.minimize_inactive = self.minimize_inactive;
        config.toggle_previews_key = self.toggle_previews_key;
        config.toggle_previews_modifier = self.toggle_previews_modifier;
        config.preview_width = self.preview_width;
        config.preview_height = self.preview_height;
        config.preview_opacity = self.preview_opacity;
        config.preview_fps = self.preview_fps;
        config.display_mode = self.display_mode;
        config.show_previews = self.show_previews;
        config.hide_active_preview = self.hide_active_preview;
        config.positions_locked = self.positions_locked;
        config.constrain_aspect = self.constrain_aspect;
    }
}

pub fn validate_name(name: &str) -> Result<()> {
    ensure!(!name.trim().is_empty(), "Name darf nicht leer sein");
    ensure!(
        name == name.trim(),
        "Name darf nicht mit Leerzeichen beginnen oder enden"
    );
    ensure!(
        name.chars().count() <= 100 && !name.chars().any(char::is_control),
        "Ungültiger Name"
    );
    Ok(())
}

fn validate_position(x: i32, y: i32) -> Result<()> {
    ensure!(
        i16::try_from(x).is_ok() && i16::try_from(y).is_ok(),
        "Vorschauposition außerhalb des unterstützten Bereichs"
    );
    Ok(())
}

impl Config {
    pub fn cycle_order(&self) -> Option<Vec<String>> {
        // An empty selected team must never accidentally cycle unrelated clients.
        if self.team.active.is_some() || !self.characters.is_empty() {
            Some(self.characters.clone())
        } else {
            None
        }
    }

    pub fn save_team(&mut self, name: &str, replace: bool) -> Result<()> {
        validate_name(name)?;
        ensure!(
            replace || !self.team.profiles.contains_key(name),
            "Profil existiert bereits: {name}"
        );
        ensure!(
            !replace || self.team.profiles.contains_key(name),
            "Profil fehlt: {name}"
        );
        let profile = TeamProfile::capture(self, PreviewPositions::load_checked()?);
        profile.validate()?;
        let mut next = self.clone();
        next.team.profiles.insert(name.to_owned(), profile);
        next.team.active = Some(name.to_owned());
        next.team.revision = next.team.revision.wrapping_add(1);
        next.save()?;
        *self = next;
        Ok(())
    }

    pub fn apply_team(&mut self, name: &str) -> Result<()> {
        let profile = self
            .team
            .profiles
            .get(name)
            .context("Profil nicht gefunden")?
            .clone();
        profile.validate()?;
        let previous_layout = PreviewPositions::load_checked()?;
        let mut next = self.clone();
        profile.apply_settings(&mut next);
        next.team.active = Some(name.to_owned());
        next.team.revision = next.team.revision.wrapping_add(1);
        profile.layout.save_checked()?;
        if let Err(error) = next.save() {
            previous_layout.save_checked().context(
                "Profilwechsel fehlgeschlagen; Layout-Rückkehr ebenfalls fehlgeschlagen",
            )?;
            return Err(error);
        }
        *self = next;
        Ok(())
    }

    pub fn delete_team(&mut self, name: &str) -> Result<()> {
        ensure!(
            self.team.profiles.contains_key(name),
            "Profil nicht gefunden"
        );
        let mut next = self.clone();
        next.team.profiles.remove(name);
        if next.team.active.as_deref() == Some(name) {
            next.team.active = None;
        }
        next.team.revision = next.team.revision.wrapping_add(1);
        next.save()?;
        *self = next;
        Ok(())
    }

    pub fn export_team(&self, name: &str, path: &Path) -> Result<()> {
        let profile = self
            .team
            .profiles
            .get(name)
            .context("Profil nicht gefunden")?;
        profile.validate()?;
        let export = ProfileExport {
            schema_version: 1,
            name: name.to_owned(),
            profile: profile.clone(),
        };
        ensure!(!path.exists(), "Exportdatei existiert bereits");
        let contents = serde_json::to_vec_pretty(&export)?;
        // create_new makes refusing overwrite race-safe.
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(path)?.write_all(&contents)?;
        Ok(())
    }

    pub fn import_team(&mut self, path: &Path) -> Result<String> {
        ensure!(
            std::fs::metadata(path)?.len() <= 1_048_576,
            "Profilimport ist größer als 1 MiB"
        );
        let export: ProfileExport =
            serde_json::from_slice(&std::fs::read(path)?).context("Ungültiger Profilimport")?;
        ensure!(export.schema_version == 1, "Unbekannte Profilversion");
        validate_name(&export.name)?;
        export.profile.validate()?;
        ensure!(
            !self.team.profiles.contains_key(&export.name),
            "Profil existiert bereits"
        );
        let mut next = self.clone();
        next.team
            .profiles
            .insert(export.name.clone(), export.profile);
        next.team.revision = next.team.revision.wrapping_add(1);
        next.save()?;
        *self = next;
        Ok(export.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileExport {
    schema_version: u32,
    name: String,
    profile: TeamProfile,
}

pub fn cli(config: &mut Config, args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("list") => {
            for (name, profile) in &config.team.profiles {
                println!("{}\t{}\t{}", if config.team.active.as_deref() == Some(name) { "*" } else { " " }, name, profile.characters.join(", "));
            }
        }
        Some("save") | Some("update") if args.len() == 2 => config.save_team(&args[1], args[0] == "update")?,
        Some("apply") if args.len() == 2 => config.apply_team(&args[1])?,
        Some("delete") if args.len() == 2 => config.delete_team(&args[1])?,
        Some("export") if args.len() == 3 => config.export_team(&args[1], Path::new(&args[2]))?,
        Some("import") if args.len() == 2 => { println!("Importiert: {}", config.import_team(Path::new(&args[1]))?); }
        _ => bail!("Verwendung: profile list | save NAME | update NAME | apply NAME | delete NAME | export NAME DATEI | import DATEI"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TeamProfile {
        TeamProfile::capture(
            &Config::build_default(1920, 1080),
            PreviewPositions::default(),
        )
    }

    #[test]
    fn rejects_duplicates_and_orphan_hotkeys() {
        let mut profile = sample();
        profile.characters = vec!["Pilot".into(), "Pilot".into()];
        assert!(profile.validate().is_err());
        profile.characters.pop();
        profile.character_hotkeys.insert(
            "Fremd".into(),
            CharacterHotkey {
                vk: 59,
                modifier: None,
            },
        );
        assert!(profile.validate().is_err());
    }

    #[test]
    fn rejects_colliding_bindings_but_allows_modifier_combinations() {
        let mut profile = sample();
        profile.enable_keyboard_buttons = true;
        profile.forward_key = 59;
        profile.backward_key = 59;
        profile.modifier_key = None;
        assert!(profile.validate().is_err());
        profile.modifier_key = Some(42);
        assert!(profile.validate().is_ok());
        profile.characters = vec!["Pilot".into()];
        profile.character_hotkeys.insert(
            "Pilot".into(),
            CharacterHotkey {
                vk: 59,
                modifier: Some(42),
            },
        );
        assert!(profile.validate().is_err());
    }

    #[test]
    fn supports_negative_monitor_coordinates_but_rejects_overflow() {
        let mut profile = sample();
        profile.layout.set("Pilot".into(), -1600, 100);
        assert!(profile.validate().is_ok());
        profile.layout.set("Pilot".into(), 100_000, 100);
        assert!(profile.validate().is_err());
    }

    #[test]
    fn portable_snapshot_preserves_local_device_and_panel_settings() {
        let mut config = Config::build_default(1920, 1080);
        config.keyboard_device_path = Some("/dev/input/private-local-device".into());
        config.window_width = 1234;
        let mut profile = sample();
        profile.characters = vec!["Miner".into(), "Hauler".into()];
        profile.preview_width = 400;
        profile.apply_settings(&mut config);
        assert_eq!(config.window_width, 1234);
        assert_eq!(
            config.keyboard_device_path.as_deref(),
            Some("/dev/input/private-local-device")
        );
        assert_eq!(config.characters, vec!["Miner", "Hauler"]);
        let json = serde_json::to_string(&profile).unwrap();
        assert!(!json.contains("device_path"));
        assert!(!json.contains("window_width"));
    }

    #[test]
    fn empty_selected_team_does_not_fall_back_to_all_windows() {
        let mut config = Config::build_default(1920, 1080);
        assert_eq!(config.cycle_order(), None);
        config.team.active = Some("Leer".into());
        assert_eq!(config.cycle_order(), Some(vec![]));
    }

    #[test]
    fn validates_name_and_snapshot_round_trip() {
        for name in ["", " ", " Mining", "Mining\n"] {
            assert!(validate_name(name).is_err());
        }
        let profile = sample();
        let encoded = serde_json::to_string(&profile).unwrap();
        let decoded: TeamProfile = serde_json::from_str(&encoded).unwrap();
        assert!(decoded.validate().is_ok());
    }
}
