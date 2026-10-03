//! Offline subprocess tests isolate every settings directory and exercise the shipped CLI.
use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("nicotine-profile-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        let fixture = Self(path);
        fixture.ok(&["profile", "list"]);
        fixture
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_Nicotine"))
            .args(args)
            .env("NICOTINE_CONFIG_DIR", &self.0)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .output()
            .unwrap()
    }
    fn ok(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "{:?}: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }
    fn read(&self) -> toml::Value {
        toml::from_str(&std::fs::read_to_string(self.0.join("config.toml")).unwrap()).unwrap()
    }
    fn write(&self, config: toml::Value) {
        std::fs::write(
            self.0.join("config.toml"),
            toml::to_string(&config).unwrap(),
        )
        .unwrap();
    }
    fn seed(&self, names: &[&str], x: i32) {
        let mut config = self.read();
        config["characters"] = toml::Value::Array(
            names
                .iter()
                .map(|name| toml::Value::String((*name).into()))
                .collect(),
        );
        config["enable_keyboard_buttons"] = false.into();
        config["enable_mouse_buttons"] = false.into();
        self.write(config);
        std::fs::write(
            self.0.join("preview_positions.toml"),
            format!("[positions]\nPilot = [{x}, 70]\n"),
        )
        .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn offline_profile_switch_restores_team_layout_and_survives_restart() {
    let f = Fixture::new();
    f.seed(&["Miner", "Hauler"], 50);
    f.ok(&["profile", "save", "Mining"]);
    f.seed(&["Fighter"], -1500);
    f.ok(&["profile", "save", "Missionen"]);
    f.ok(&["profile", "apply", "Mining"]);
    assert_eq!(f.read()["characters"].as_array().unwrap().len(), 2);
    let layout: toml::Value =
        toml::from_str(&std::fs::read_to_string(f.0.join("preview_positions.toml")).unwrap())
            .unwrap();
    assert_eq!(layout["positions"]["Pilot"][0].as_integer(), Some(50));
    assert!(f.ok(&["profile", "list"]).contains("*\tMining"));
    f.ok(&["profile", "apply", "Missionen"]);
    assert_eq!(f.read()["characters"][0].as_str(), Some("Fighter"));
}

#[test]
fn import_export_refuse_overwrites_and_keep_local_device_paths_private() {
    let f = Fixture::new();
    let mut config = f.read();
    config.as_table_mut().unwrap().insert(
        "keyboard_device_path".into(),
        "/dev/input/local-secret-path".into(),
    );
    f.write(config);
    f.seed(&["Pilot"], 20);
    f.ok(&["profile", "save", "Transport"]);
    let export = f.0.join("export.json");
    let file = export.to_str().unwrap();
    f.ok(&["profile", "export", "Transport", file]);
    assert!(!std::fs::read_to_string(&export)
        .unwrap()
        .contains("local-secret-path"));
    assert!(!f
        .run(&["profile", "export", "Transport", file])
        .status
        .success());
    assert!(!f.run(&["profile", "import", file]).status.success());
    f.ok(&["profile", "delete", "Transport"]);
    f.ok(&["profile", "import", file]);
    assert!(f.ok(&["profile", "list"]).contains("Transport"));
    f.ok(&["profile", "apply", "Transport"]);
    assert_eq!(
        f.read()["keyboard_device_path"].as_str(),
        Some("/dev/input/local-secret-path")
    );
}

#[test]
fn malformed_layout_or_future_import_cannot_replace_current_settings() {
    let f = Fixture::new();
    f.seed(&["Pilot"], 10);
    f.ok(&["profile", "save", "Mining"]);
    std::fs::write(f.0.join("preview_positions.toml"), "broken = [").unwrap();
    let before = std::fs::read(f.0.join("config.toml")).unwrap();
    assert!(!f.run(&["profile", "save", "Neu"]).status.success());
    assert!(!f.run(&["profile", "apply", "Mining"]).status.success());
    assert_eq!(before, std::fs::read(f.0.join("config.toml")).unwrap());
    let export = f.0.join("future.json");
    f.ok(&["profile", "export", "Mining", export.to_str().unwrap()]);
    let mut data: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&export).unwrap()).unwrap();
    data["schema_version"] = 99.into();
    std::fs::write(&export, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(!f
        .run(&["profile", "import", export.to_str().unwrap()])
        .status
        .success());
    assert_eq!(before, std::fs::read(f.0.join("config.toml")).unwrap());
}

#[test]
fn unknown_profile_does_not_modify_layout() {
    let f = Fixture::new();
    f.seed(&["Pilot"], 10);
    f.ok(&["profile", "save", "Mining"]);
    let before = std::fs::read(f.0.join("preview_positions.toml")).unwrap();
    let missing = f.run(&["profile", "apply", "Unknown"]);
    assert!(!missing.status.success());
    assert_eq!(
        before,
        std::fs::read(f.0.join("preview_positions.toml")).unwrap()
    );
}
