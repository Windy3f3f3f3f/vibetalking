use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub app_id: String,
    pub access_token: String,
    pub resource_id: String,
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            app_id: crate::config::APP_ID.into(),
            access_token: crate::config::ACCESS_TOKEN.into(),
            resource_id: crate::config::RESOURCE_ID.into(),
            language: crate::config::LANGUAGE.into(),
        }
    }
}

pub struct SettingsStore {
    path: PathBuf,
    inner: RwLock<Settings>,
}

impl SettingsStore {
    pub fn load() -> Result<Self> {
        let dir = dirs::data_dir()
            .ok_or_else(|| anyhow!("no data dir"))?
            .join("com.vibetalk.dictation");
        fs::create_dir_all(&dir)?;
        let path = dir.join("settings.json");
        let inner = if path.exists() {
            serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or_default()
        } else {
            Settings::default()
        };
        Ok(Self {
            path,
            inner: RwLock::new(inner),
        })
    }

    pub fn get(&self) -> Settings {
        self.inner.read().clone()
    }

    pub fn save(&self, new: Settings) -> Result<()> {
        {
            let mut w = self.inner.write();
            *w = new;
        }
        let data = self.inner.read().clone();
        fs::write(&self.path, serde_json::to_string_pretty(&data)?)?;
        Ok(())
    }
}
