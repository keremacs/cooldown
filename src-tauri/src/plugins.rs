//! Plugin registry — extensible event sources beyond VS Code / terminal.

use std::collections::HashMap;
use parking_lot::RwLock;

use crate::models::{DevEvent, PluginInfo};

pub struct PluginRegistry {
    inner: RwLock<HashMap<String, PluginRecord>>,
}

struct PluginRecord {
    name: String,
    events: u64,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let mut defaults = HashMap::new();
        defaults.insert(
            "vscode".into(),
            PluginRecord {
                name: "VS Code".into(),
                events: 0,
            },
        );
        defaults.insert(
            "terminal".into(),
            PluginRecord {
                name: "Terminal".into(),
                events: 0,
            },
        );
        defaults.insert(
            "powershell".into(),
            PluginRecord {
                name: "PowerShell".into(),
                events: 0,
            },
        );
        Self {
            inner: RwLock::new(defaults),
        }
    }

    pub fn ingest(&self, event: &DevEvent) {
        let id = event
            .plugin
            .clone()
            .unwrap_or_else(|| event.source.clone());
        let mut inner = self.inner.write();
        let record = inner.entry(id.clone()).or_insert_with(|| PluginRecord {
            name: id.clone(),
            events: 0,
        });
        record.events += 1;
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        self.inner
            .read()
            .iter()
            .map(|(id, r)| PluginInfo {
                id: id.clone(),
                name: r.name.clone(),
                events_received: r.events,
            })
            .collect()
    }
}
