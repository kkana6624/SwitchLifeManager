use crate::infrastructure::persistence::FileConfigRepository;

/// Initialize the file logger.
/// Writes to `app.log` in the same directory as the profile config.
pub fn init_logger() {
    if let Ok(config_path) = FileConfigRepository::get_default_config_path() {
        let log_path = config_path.with_file_name("app.log");
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let _ = simplelog::WriteLogger::init(
            simplelog::LevelFilter::Info,
            simplelog::Config::default(),
            std::fs::File::create(log_path)
                .unwrap_or_else(|_| std::fs::File::create("switch_life_manager.log").unwrap()),
        );
    }
}
