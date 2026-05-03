//! # LFMF Counter Support
//!
//! Phase 3 of b00t self-documenting system:
//! - Track $🦨++ and $🐛++ counters per tool
//! - Provide task/problem spot discovery for agents
//! - Support subsequent agent idle discovery
//!
//! Usage:
//! ```ignore
//! use lfmf_counter::LfmfCounter;
//!
//! let mut counter = LfmfCounter::load().await?;
//! counter.increment_bug("mcp").await?;
//! let stats = counter.get_stats().await?;
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Counter file name for storing statistics
const COUNTER_FILE: &str = ".lfmf-counters.json";

/// LFMF lesson counter per tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCounter {
    /// Tool name (e.g., "mcp", "rust", "bash")
    pub tool: String,

    /// Count of 🦨 skunk emoji (code changes/rename markers)
    pub skunk_count: usize,

    /// Count of 🐛 bug emoji (lessons learned from failures)
    pub bug_count: usize,

    /// Total lessons recorded for this tool
    pub total_lessons: usize,

    /// Last timestamp when counter was updated
    pub last_updated: String,
}

impl ToolCounter {
    /// Create a new counter for a tool
    pub fn new(tool: &str) -> Self {
        Self {
            tool: tool.to_string(),
            skunk_count: 0,
            bug_count: 0,
            total_lessons: 0,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Increment 🦨 counter (code change markers)
    pub fn increment_skunk(&mut self) {
        self.skunk_count += 1;
        self.total_lessons += 1;
        self.update_timestamp();
    }

    /// Increment 🐛 counter (lesson learned)
    pub fn increment_bug(&mut self) {
        self.bug_count += 1;
        self.total_lessons += 1;
        self.update_timestamp();
    }

    /// Get classification based on counts
    pub fn get_classification(&self) -> CounterClassification {
        if self.skunk_count > self.bug_count * 2 {
            CounterClassification::CodeHeavy
        } else if self.bug_count > self.skunk_count * 2 {
            CounterClassification::ErrorProne
        } else {
            CounterClassification::Balanced
        }
    }

    /// Update timestamp
    fn update_timestamp(&mut self) {
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }
}

/// Classification of tool counter state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CounterClassification {
    /// Tool has many skunk markers (code changes)
    CodeHeavy,

    /// Tool has many bug markers (error recovery)
    ErrorProne,

    /// Balanced learning from both code and errors
    Balanced,
}

/// LFMF counter system for managing tool-level statistics
#[derive(Debug, Clone)]
pub struct LfmfCounter {
    /// Per-tool counters
    counters: HashMap<String, ToolCounter>,

    /// Storage path for counter file
    storage_path: PathBuf,
}

impl LfmfCounter {
    /// Load counters from storage file
    pub async fn load() -> Result<Self> {
        let storage_path = Self::default_storage_path();
        let counters = if storage_path.exists() {
            let content = fs::read_to_string(&storage_path)
                .await
                .context("Failed to read counter file")?;

            serde_json::from_str(&content).context("Failed to parse counter file")?
        } else {
            HashMap::new()
        };

        Ok(Self {
            counters,
            storage_path,
        })
    }

    /// Get default storage path
    fn default_storage_path() -> PathBuf {
        std::env::current_dir()
            .expect("cannot determine current directory for counter storage")
            .join(COUNTER_FILE)
    }

    /// Increment skunk counter for a tool
    ///
    /// This is called when a lesson contains 🦨 marker
    pub async fn increment_skunk(&mut self, tool: &str) -> Result<()> {
        let counter = self
            .counters
            .entry(tool.to_string())
            .or_insert_with(|| ToolCounter::new(tool));

        counter.increment_skunk();
        self.save().await?;

        Ok(())
    }

    /// Increment bug counter for a tool
    ///
    /// This is called when a lesson is recorded (🐛)
    pub async fn increment_bug(&mut self, tool: &str) -> Result<()> {
        let counter = self
            .counters
            .entry(tool.to_string())
            .or_insert_with(|| ToolCounter::new(tool));

        counter.increment_bug();
        self.save().await?;

        Ok(())
    }

    /// Get statistics for all tools
    pub async fn get_stats(&self) -> Result<Vec<ToolCounter>> {
        let mut stats: Vec<_> = self.counters.values().cloned().collect();
        stats.sort_by(|a, b| a.tool.cmp(&b.tool));
        Ok(stats)
    }

    /// Get counter for specific tool
    pub fn get_counter(&self, tool: &str) -> Option<&ToolCounter> {
        self.counters.get(tool)
    }

    /// Get tools sorted by total lessons (most to least)
    pub fn get_tools_by_activity(&self) -> Vec<&ToolCounter> {
        let mut tools: Vec<_> = self.counters.values().collect();
        tools.sort_by_key(|b| std::cmp::Reverse(b.total_lessons));
        tools
    }

    /// Get tools that need attention (high error rate)
    pub fn get_problematic_tools(&self) -> Vec<&ToolCounter> {
        self.counters
            .values()
            .filter(|c| c.get_classification() == CounterClassification::ErrorProne)
            .collect()
    }

    /// Save counters to storage file
    async fn save(&self) -> Result<()> {
        let content =
            serde_json::to_string_pretty(&self.counters).context("Failed to serialize counters")?;

        fs::write(&self.storage_path, content)
            .await
            .context("Failed to write counter file")?;

        Ok(())
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> CounterSummary {
        let total_tools = self.counters.len();
        let total_skunk: usize = self.counters.values().map(|c| c.skunk_count).sum();
        let total_bugs: usize = self.counters.values().map(|c| c.bug_count).sum();
        let total_lessons: usize = self.counters.values().map(|c| c.total_lessons).sum();

        CounterSummary {
            total_tools,
            total_skunk,
            total_bugs,
            total_lessons,
            problematic_tools: self.get_problematic_tools().len(),
            most_active: self.get_tools_by_activity().first().map(|c| c.tool.clone()),
        }
    }
}

/// Summary statistics across all tools
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CounterSummary {
    /// Total number of tools with recorded lessons
    pub total_tools: usize,

    /// Total 🦨 markers across all tools
    pub total_skunk: usize,

    /// Total 🐛 lessons learned across all tools
    pub total_bugs: usize,

    /// Total lessons recorded (skunk + bugs)
    pub total_lessons: usize,

    /// Number of tools that need attention (high error rate)
    pub problematic_tools: usize,

    /// Most active tool (highest lesson count)
    pub most_active: Option<String>,
}

impl Default for LfmfCounter {
    fn default() -> Self {
        Self {
            counters: HashMap::new(),
            storage_path: Self::default_storage_path(),
        }
    }
}

/// CLI-friendly counter display formatting
pub struct CounterDisplay {
    pub tool: String,
    pub classification: String,
    pub skunk: usize,
    pub bugs: usize,
    pub total: usize,
    pub emoji: String,
}

impl CounterDisplay {
    /// Format counter for display
    pub fn format(&self) -> String {
        format!(
            "{} {}: {} ({}) - 🦨: {} 🐛: {}",
            self.emoji, self.tool, self.classification, self.total, self.skunk, self.bugs
        )
    }

    /// Create display from counter
    pub fn from_counter(counter: &ToolCounter) -> Self {
        let classification = match counter.get_classification() {
            CounterClassification::CodeHeavy => "💻 Code-Heavy",
            CounterClassification::ErrorProne => "⚠️ Error-Prone",
            CounterClassification::Balanced => "✅ Balanced",
        };

        Self {
            tool: counter.tool.clone(),
            classification: classification.to_string(),
            skunk: counter.skunk_count,
            bugs: counter.bug_count,
            total: counter.total_lessons,
            emoji: if counter.bug_count > counter.skunk_count {
                "🐛".to_string()
            } else {
                "🦨".to_string()
            },
        }
    }

    /// Get task/problem suggestion based on counter state
    pub fn get_suggestion(&self) -> Option<&'static str> {
        if self.bugs > self.skunk * 3 {
            Some("Consider: Focus on error prevention patterns and testing")
        } else if self.skunk > self.bugs * 3 {
            Some("Consider: Focus on architectural stability and code review")
        } else if self.total == 0 {
            Some("Start: Record first lesson to establish baseline")
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_counter() -> LfmfCounter {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        LfmfCounter {
            counters: HashMap::new(),
            storage_path: std::env::temp_dir().join(format!("lfmf-counter-{nonce}.json")),
        }
    }

    #[tokio::test]
    async fn test_counter_increment() {
        let mut counter = test_counter();

        // Test skunk increment
        counter.increment_skunk("test_tool").await.unwrap();
        let stats = counter.get_stats().await.unwrap();
        assert!(stats.len() == 1);
        assert!(stats[0].skunk_count == 1);

        // Test bug increment
        counter.increment_bug("test_tool").await.unwrap();
        let stats = counter.get_stats().await.unwrap();
        assert!(stats[0].bug_count == 1);
        assert!(stats[0].total_lessons == 2);
    }

    #[test]
    fn test_counter_classification() {
        let mut counter = ToolCounter::new("test");

        counter.increment_bug();
        assert_eq!(
            counter.get_classification(),
            CounterClassification::ErrorProne
        );

        counter.increment_skunk();
        counter.increment_skunk();
        counter.increment_skunk();

        assert_eq!(
            counter.get_classification(),
            CounterClassification::CodeHeavy
        );
    }

    #[tokio::test]
    async fn test_summary() {
        let mut counter = test_counter();

        counter.increment_skunk("tool1").await.unwrap();
        counter.increment_bug("tool2").await.unwrap();

        let summary = counter.get_summary();

        assert_eq!(summary.total_tools, 2);
        assert_eq!(summary.total_lessons, 2);
        assert_eq!(summary.total_skunk, 1);
        assert_eq!(summary.total_bugs, 1);
    }
}
