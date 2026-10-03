//! Restricted GNOME Shell interface for native Wine Wayland clients.
use crate::config::{Config, LiveSettings};
use crate::preview_positions::PreviewPositions;
use crate::window_manager::{EveWindow, WindowManager};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use zbus::blocking::Connection;

const PATH: &str = "/org/gnome/Shell/Extensions/NicotineTeams";
const INTERFACE: &str = "org.gnome.Shell.Extensions.NicotineTeams";

pub struct GnomeBridgeManager {
    connection: Connection,
}

#[derive(Deserialize)]
struct BridgeWindow {
    id: u32,
    title: String,
    pid: u32,
}

impl GnomeBridgeManager {
    pub fn new() -> Result<Self> {
        let manager = Self {
            connection: zbus::blocking::connection::Builder::session()?
                .method_timeout(Duration::from_secs(2))
                .build()?,
        };
        let version: u32 = manager
            .connection
            .call_method(
                Some("org.gnome.Shell"),
                PATH,
                Some(INTERFACE),
                "Version",
                &(),
            )?
            .body()
            .deserialize()?;
        if version != 1 {
            bail!("Unsupported GNOME bridge version: {version}");
        }
        Ok(manager)
    }

    fn json_call(&self, method: &str) -> Result<String> {
        Ok(self
            .connection
            .call_method(Some("org.gnome.Shell"), PATH, Some(INTERFACE), method, &())?
            .body()
            .deserialize()?)
    }

    fn action(&self, method: &str, id: u32) -> Result<()> {
        let changed: bool = self
            .connection
            .call_method(
                Some("org.gnome.Shell"),
                PATH,
                Some(INTERFACE),
                method,
                &(id,),
            )?
            .body()
            .deserialize()?;
        if !changed {
            bail!("EVE-Fenster {id} ist nicht mehr verfügbar");
        }
        Ok(())
    }
}

fn parse_windows(json: &str, is_eve: impl Fn(u32) -> bool) -> Result<Vec<EveWindow>> {
    let windows: Vec<BridgeWindow> =
        serde_json::from_str(json).context("Invalid GNOME window list")?;
    Ok(windows
        .into_iter()
        .filter(|w| w.id != 0 && w.title.starts_with("EVE - ") && is_eve(w.pid))
        .map(|w| EveWindow {
            id: w.id,
            title: w.title.trim_start_matches("EVE - ").to_owned(),
        })
        .collect())
}

impl WindowManager for GnomeBridgeManager {
    fn uses_compositor_previews(&self) -> bool {
        true
    }
    fn get_eve_windows(&self) -> Result<Vec<EveWindow>> {
        parse_windows(
            &self.json_call("List")?,
            crate::eve_match::pid_is_eve_client,
        )
    }
    fn activate_window(&self, id: u32) -> Result<()> {
        self.action("Activate", id)
    }
    fn minimize_window(&self, id: u32) -> Result<()> {
        self.action("Minimize", id)
    }
    fn restore_window(&self, id: u32) -> Result<()> {
        self.action("Restore", id)
    }
    fn get_active_window(&self) -> Result<u32> {
        Ok(self
            .connection
            .call_method(
                Some("org.gnome.Shell"),
                PATH,
                Some(INTERFACE),
                "Active",
                &(),
            )?
            .body()
            .deserialize()?)
    }
    fn stack_windows(&self, _: &[EveWindow], _: &Config) -> Result<()> {
        bail!("Fensterlayout-Editor für GNOME ist noch nicht umgesetzt")
    }
}

pub fn spawn_previews(live: Arc<Mutex<LiveSettings>>) {
    std::thread::spawn(move || {
        let Ok(manager) = GnomeBridgeManager::new() else {
            return;
        };
        loop {
            let result = (|| -> Result<()> {
                let config = Config::load()?;
                let settings = live
                    .lock()
                    .map_err(|_| anyhow::anyhow!("settings lock"))?
                    .clone();
                let positions = PreviewPositions::load_checked()?;
                let payload = serde_json::json!({
                    "show": settings.show_previews,
                    "hide_active": settings.hide_active_preview,
                    "characters": config.cycle_order(),
                    "width": settings.preview_width,
                    "height": settings.preview_height,
                    "opacity": settings.preview_opacity,
                    "mode": format!("{:?}", settings.display_mode),
                    "revision": settings.layout_revision,
                    "positions": positions.positions,
                })
                .to_string();
                manager.connection.call_method(
                    Some("org.gnome.Shell"),
                    PATH,
                    Some(INTERFACE),
                    "Previews",
                    &(payload,),
                )?;
                Ok(())
            })();
            if let Err(error) = result {
                eprintln!("GNOME-Vorschauen: {error}");
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bridge_filters_processes_and_preserves_names() {
        let json = r#"[{"id":7,"pid":10,"title":"EVE - Pilot' One"},{"id":8,"pid":20,"title":"EVE - Browser"},{"id":9,"pid":10,"title":"Launcher"},{"id":0,"pid":10,"title":"EVE - Invalid"}]"#;
        let windows = parse_windows(json, |pid| pid == 10).unwrap();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].title, "Pilot' One");
        assert_eq!(windows[0].id, 7);
    }
    #[test]
    fn bridge_rejects_malformed_data() {
        assert!(parse_windows("{}", |_| true).is_err());
        assert!(parse_windows(r#"[{"id":1,"title":"EVE - Missing PID"}]"#, |_| true).is_err());
    }
}
