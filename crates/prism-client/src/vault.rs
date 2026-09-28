use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single encrypted or private context record stored on the user's edge device
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextRecord {
    pub category: String, // e.g. "health", "ecommerce", "travel"
    pub key: String,      // e.g. "weekly_run_distance_km", "avg_sleep_hours", "diet"
    pub value_num: Option<f64>,
    pub value_str: Option<String>,
    pub timestamp: i64,
}

/// Sovereign on-device vault representing the user's private life data
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SovereignContextVault {
    pub records: HashMap<String, ContextRecord>,
}

impl SovereignContextVault {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Record or update a numeric context metric (e.g., Apple HealthKit or Google Fit sync)
    pub fn record_numeric(&mut self, category: &str, key: &str, val: f64) {
        let entry_key = format!("{}:{}", category, key);
        self.records.insert(
            entry_key,
            ContextRecord {
                category: category.to_string(),
                key: key.to_string(),
                value_num: Some(val),
                value_str: None,
                timestamp: chrono::Utc::now().timestamp(),
            },
        );
    }

    /// Record a categorical or string context metric (e.g., preferred airline, dietary preference)
    pub fn record_string(&mut self, category: &str, key: &str, val: &str) {
        let entry_key = format!("{}:{}", category, key);
        self.records.insert(
            entry_key,
            ContextRecord {
                category: category.to_string(),
                key: key.to_string(),
                value_num: None,
                value_str: Some(val.to_string()),
                timestamp: chrono::Utc::now().timestamp(),
            },
        );
    }

    /// Retrieve a numeric metric value
    pub fn get_numeric(&self, category: &str, key: &str) -> Option<f64> {
        let entry_key = format!("{}:{}", category, key);
        self.records.get(&entry_key).and_then(|r| r.value_num)
    }

    /// Retrieve a string metric value
    pub fn get_string(&self, category: &str, key: &str) -> Option<&str> {
        let entry_key = format!("{}:{}", category, key);
        self.records.get(&entry_key).and_then(|r| r.value_str.as_deref())
    }
}
