use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
#[cfg(target_os = "windows")]
use std::thread;
#[cfg(target_os = "windows")]
use std::time::Duration;
use tempfile::NamedTempFile;
use crate::domain::models::UserProfile;

// Current Schema Version (Should match UserProfile::default() schema_version)
const CURRENT_SCHEMA_VERSION: u32 = 1;

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

    /// Resolves the default configuration path using the `directories` crate.
    ///
    /// Windows: %LOCALAPPDATA%\SwitchLifeManager\profile.json
    /// Linux:   $XDG_DATA_HOME/SwitchLifeManager/profile.json (or ~/.local/share/...)
    pub fn get_default_config_path() -> Result<PathBuf> {
        // Use empty qualifier/org to get cleaner paths closer to user request
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "SwitchLifeManager") {
            let config_dir = proj_dirs.data_dir(); // On Windows this is usually %LOCALAPPDATA%\SwitchLifeManager
            Ok(config_dir.join("profile.json"))
        } else {
             // Fallback for weird environments
             let mut path = std::env::current_dir().context("Failed to get current dir")?;
             path.push("profile.json");
             Ok(path)
        }
    }

    /// Ensures the directory for the config file exists.
    fn ensure_directory(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).context(format!("Failed to create directory: {:?}", parent))?;
            }
        }
        Ok(())
    }

    // Platform specific rename
    #[cfg(target_os = "windows")]
    fn atomic_rename_with_retry(src: &Path, dst: &Path) -> Result<()> {
        use windows::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH};
        use std::os::windows::ffi::OsStrExt;

        let src_wide: Vec<u16> = src.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let dst_wide: Vec<u16> = dst.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

        let mut attempts = 0;
        const MAX_ATTEMPTS: i32 = 3;

        loop {
            let result = unsafe {
                MoveFileExW(
                    windows::core::PCWSTR(src_wide.as_ptr()),
                    windows::core::PCWSTR(dst_wide.as_ptr()),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH
                )
            };

            match result {
                Ok(_) => return Ok(()),
                Err(e) => {
                    attempts += 1;
                    if attempts >= MAX_ATTEMPTS {
                        return Err(anyhow::anyhow!("MoveFileExW failed after {} attempts: {}", attempts, e));
                    }
                    // Wait 50-100ms
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn atomic_rename_with_retry(src: &Path, dst: &Path) -> Result<()> {
        // Simple rename for non-Windows (or implement retry if desired, but requirement specified Windows logic)
        fs::rename(src, dst).context("Failed to rename temp file")
    }
}

impl ConfigRepository for FileConfigRepository {
    fn load(&self) -> Result<UserProfile> {
        if !self.path.exists() {
            // New profile: Ensure we can create the directory later.
            // Return default.
            return Ok(UserProfile::default());
        }

        let file = fs::File::open(&self.path).context(format!("Failed to open config file: {:?}", self.path))?;
        let reader = std::io::BufReader::new(file);
        let profile: UserProfile = serde_json::from_reader(reader).context("Failed to parse config file")?;

        // Schema Version Validation
        if profile.schema_version != CURRENT_SCHEMA_VERSION {
             // Future: Migration logic here
             return Err(anyhow!("Schema version mismatch: expected {}, found {}. Migration not implemented.", CURRENT_SCHEMA_VERSION, profile.schema_version));
        }

        Ok(profile)
    }

    fn save(&self, profile: &UserProfile) -> Result<()> {
        // Ensure parent directory exists
        self.ensure_directory()?;

        // Create temp file in the SAME DIRECTORY to ensure atomic rename works
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        let file = NamedTempFile::new_in(parent).context("Failed to create temp file")?;

        // Write JSON
        serde_json::to_writer_pretty(&file, profile).context("Failed to serialize config")?;

        // Flush and Sync
        file.as_file().sync_all().context("Failed to sync temp file")?;

        // Persist (Atomic Rename)
        // keep() detaches the file from the TempFile wrapper so it doesn't get deleted on drop
        let (temp_file, temp_path) = file.keep().context("Failed to keep temp file")?;

        // Explicitly drop the file handle to ensure it's closed before renaming
        drop(temp_file);

        Self::atomic_rename_with_retry(&temp_path, &self.path).map_err(|e| {
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
        let file_path = dir.path().join("subdir").join("config.json");
        let repo = FileConfigRepository::new(&file_path);

        let mut profile = UserProfile::default();
        profile.config.target_controller_index = 99;

        // Save should create directory
        repo.save(&profile).unwrap();

        assert!(file_path.exists());

        let loaded = repo.load().unwrap();
        assert_eq!(loaded.config.target_controller_index, 99);
    }

    #[test]
    fn test_schema_version_check() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.json");
        let repo = FileConfigRepository::new(&file_path);

        let mut profile = UserProfile::default();
        profile.schema_version = 999; // Invalid version

        // Manually write bad file
        let file = fs::File::create(&file_path).unwrap();
        serde_json::to_writer(&file, &profile).unwrap();

        // Load should fail
        let result = repo.load();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Schema version mismatch"));
    }

    #[test]
    fn test_get_default_config_path() {
        // Just check it returns something reasonable and doesn't crash
        let path = FileConfigRepository::get_default_config_path();
        assert!(path.is_ok());
        println!("Default Path: {:?}", path.unwrap());
    }
}
