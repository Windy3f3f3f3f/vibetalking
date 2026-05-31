use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub app_id: String,
    pub access_token: String,
    pub resource_id: String,
    pub language: String,
    pub transcribe_provider: String,
    pub dashscope_api_key: String,
    pub dashscope_base_url: String,
    pub qwen_asr_model: String,
    pub qwen_asr_language: String,
    pub omni_prompt: String,
    pub refine_enabled: bool,
    pub refine_api_key: String,
    pub refine_base_url: String,
    pub refine_model: String,
    pub refine_prompt: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            app_id: crate::config::APP_ID.into(),
            access_token: crate::config::ACCESS_TOKEN.into(),
            resource_id: crate::config::RESOURCE_ID.into(),
            language: crate::config::LANGUAGE.into(),
            transcribe_provider: crate::config::TRANSCRIBE_PROVIDER.into(),
            dashscope_api_key: String::new(),
            dashscope_base_url: crate::config::DASHSCOPE_BASE_URL.into(),
            qwen_asr_model: crate::config::QWEN_ASR_MODEL.into(),
            qwen_asr_language: String::new(),
            omni_prompt: crate::config::OMNI_PROMPT.into(),
            refine_enabled: true,
            refine_api_key: String::new(),
            refine_base_url: crate::config::REFINE_BASE_URL.into(),
            refine_model: crate::config::REFINE_MODEL.into(),
            refine_prompt: crate::config::REFINE_PROMPT.into(),
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
