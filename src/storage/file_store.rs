use anyhow::Result;
use directories::UserDirs;
use std::{fs, path::PathBuf};

use crate::models::config::Config;
use crate::models::patterns::PatternOverrides;
use crate::storage::store::Store;

pub struct FileStore {
    path: PathBuf,
    patterns_path: PathBuf,
}

impl FileStore {
    fn save_config(&self, config: &Config) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    fn load_patterns(&self) -> Result<PatternOverrides> {
        if !self.patterns_path.exists() {
            return Ok(PatternOverrides::default());
        }
        let content = fs::read_to_string(&self.patterns_path)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn save_patterns(&self, overrides: &PatternOverrides) -> Result<()> {
        let content = serde_json::to_string_pretty(overrides)?;
        fs::write(&self.patterns_path, content)?;
        Ok(())
    }
}

impl Store for FileStore {
    fn load_config(&self) -> Result<Config> {
        if !self.path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&content)?)
    }
    fn get_email(&self) -> Result<String> {
        let config = self.load_config()?;
        return Ok(config.email);
    }
    fn get_delay(&self) -> Result<i64> {
        let config = self.load_config()?;
        return Ok(config.delay);
    }
    fn set_email(&self, value: String) -> Result<()> {
        let mut config = self.load_config()?;
        config.email = value;
        self.save_config(&config)
    }
    fn set_delay(&self, value: i64) -> Result<()> {
        let mut config = self.load_config()?;
        config.delay = value;
        self.save_config(&config)
    }

    fn get_patterns(&self) -> Result<Vec<String>> {
        Ok(self.load_patterns()?.effective())
    }

    fn add_pattern(&self, pattern: String) -> Result<()> {
        let mut overrides = self.load_patterns()?;
        overrides.add(pattern);
        self.save_patterns(&overrides)
    }

    fn remove_pattern(&self, pattern: String) -> Result<()> {
        let mut overrides = self.load_patterns()?;
        overrides.remove(&pattern);
        self.save_patterns(&overrides)
    }

    fn reset_patterns(&self) -> Result<()> {
        let mut overrides = self.load_patterns()?;
        overrides.reset();
        self.save_patterns(&overrides)
    }
}

impl Default for FileStore {
    fn default() -> Self {
        let home = UserDirs::new()
            .expect("unable to locate home directory")
            .home_dir()
            .to_path_buf();

        let config_dir = home.join(".mailcheck");
        std::fs::create_dir_all(&config_dir).expect("unable to create config directory");
        let config_path = config_dir.join("config.json");
        let patterns_path = config_dir.join("patterns.json");

        if !config_path.exists() {
            let config = crate::models::config::Config::default();
            let content = serde_json::to_string_pretty(&config).unwrap();
            std::fs::write(&config_path, content).unwrap();
        }

        Self {
            path: config_path,
            patterns_path: patterns_path,
        }
    }
}
