//! Atomic replacement prevents a partial write from destroying the last valid settings.
use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;

pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().context("Speicherpfad hat kein Verzeichnis")?;
    std::fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".nicotine-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        file.write_all(contents)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}
