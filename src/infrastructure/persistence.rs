use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use tempfile::NamedTempFile;
use crate::domain::models::UserProfile;

pub trait ConfigRepository {
    fn load(&self) -> Result<UserProfile>;
    fn save(&self, profile: &UserProfile) -> Result<()>;
}

pub struct FileConfigRepository {
    path: PathBuf,
}

impl FileConfigRepository {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    // Platform specific rename
    #[cfg(target_os = "windows")]
    fn atomic_rename(src: &Path, dst: &Path) -> Result<()> {
        use windows::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH};
        use std::os::windows::ffi::OsStrExt;

        let src_wide: Vec<u16> = src.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let dst_wide: Vec<u16> = dst.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

        let result = unsafe {
            MoveFileExW(
                windows::core::PCWSTR(src_wide.as_ptr()),
                windows::core::PCWSTR(dst_wide.as_ptr()),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH
            )
        };

        // MoveFileExW returns BOOL (which is i32 alias or similar in windows-rs, usually can be checked as boolean logic or .as_bool())
        // In recent windows-rs, some functions return windows::core::Result<()> directly, but MoveFileExW returns BOOL.
        // Let's check if it returns Result or BOOL.
        // Documentation says BOOL.
        // windows 0.52+ wraps many things.
        // Error: no method named `as_bool` found for enum `Result<T, E>`
        // This implies `MoveFileExW` returns `windows::core::Result<()>`.

        // If it returns Result<()>, we just bubble it up or map it.
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("MoveFileExW failed: {}", e)),
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn atomic_rename(src: &Path, dst: &Path) -> Result<()> {
        fs::rename(src, dst).context("Failed to rename temp file")
    }
}

impl ConfigRepository for FileConfigRepository {
    fn load(&self) -> Result<UserProfile> {
        if !self.path.exists() {
            return Ok(UserProfile::default());
        }
        let file = fs::File::open(&self.path).context("Failed to open config file")?;
        let reader = std::io::BufReader::new(file);
        let profile = serde_json::from_reader(reader).context("Failed to parse config file")?;
        Ok(profile)
    }

    fn save(&self, profile: &UserProfile) -> Result<()> {
        // Create temp file in the SAME DIRECTORY to ensure atomic rename works
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        let file = NamedTempFile::new_in(parent).context("Failed to create temp file")?;

        // Write JSON
        serde_json::to_writer_pretty(&file, profile).context("Failed to serialize config")?;

        // Flush
        // TempFile flushes on drop/persist, but we can be explicit
        // file.as_file().sync_all()?; // Optional but good for safety

        // Persist (Atomic Rename)
        let (temp_file, temp_path) = file.keep().context("Failed to keep temp file")?;

        // We explicitly close the file handle before renaming on Windows especially,
        // though `keep()` detaches it.
        // Wait, `keep()` returns `(File, PathBuf)`. The file is still open.
        // We should close it to be safe before moving, although MoveFileEx might handle open handles if share mode allows,
        // but it's cleaner to drop the file handle.
        drop(temp_file);

        Self::atomic_rename(&temp_path, &self.path).map_err(|e| {
            // Attempt cleanup if rename fails
            let _ = fs::remove_file(&temp_path);
            e
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_load() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.json");
        let repo = FileConfigRepository::new(&file_path);

        let mut profile = UserProfile::default();
        profile.config.target_controller_index = 99;

        repo.save(&profile).unwrap();

        assert!(file_path.exists());

        let loaded = repo.load().unwrap();
        assert_eq!(loaded.config.target_controller_index, 99);
    }

    #[test]
    fn test_atomic_overwrite() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.json");
        let repo = FileConfigRepository::new(&file_path);

        // Save initial
        let mut p1 = UserProfile::default();
        p1.config.target_controller_index = 1;
        repo.save(&p1).unwrap();

        // Save update
        let mut p2 = UserProfile::default();
        p2.config.target_controller_index = 2;
        repo.save(&p2).unwrap();

        let loaded = repo.load().unwrap();
        assert_eq!(loaded.config.target_controller_index, 2);
    }
}
