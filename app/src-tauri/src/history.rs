use crate::config::HISTORY_MAX;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub text: String,
    pub duration_ms: u64,
    #[serde(default)]
    pub error: Option<String>,
}

pub struct HistoryStore {
    path: PathBuf,
    items: Mutex<Vec<HistoryItem>>,
}

impl HistoryStore {
    pub fn load() -> Result<Self> {
        let dir = dirs::data_dir()
            .ok_or_else(|| anyhow!("no data dir"))?
            .join("com.vibetalk.dictation");
        fs::create_dir_all(&dir)?;
        let path = dir.join("history.json");
        let items = if path.exists() {
            serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or_default()
        } else {
            Vec::new()
        };
        Ok(Self {
            path,
            items: Mutex::new(items),
        })
    }

    pub fn add(&self, item: HistoryItem) -> Result<()> {
        {
            let mut items = self.items.lock();
            items.insert(0, item);
            if items.len() > HISTORY_MAX {
                items.truncate(HISTORY_MAX);
            }
        }
        self.persist()
    }

    pub fn list(&self) -> Vec<HistoryItem> {
        self.items.lock().clone()
    }

    pub fn get(&self, id: &str) -> Option<HistoryItem> {
        self.items.lock().iter().find(|i| i.id == id).cloned()
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.items.lock().retain(|i| i.id != id);
        self.persist()
    }

    pub fn clear(&self) -> Result<()> {
        self.items.lock().clear();
        self.persist()
    }

    fn persist(&self) -> Result<()> {
        let items = self.items.lock().clone();
        fs::write(&self.path, serde_json::to_string_pretty(&items)?)?;
        Ok(())
    }
}
