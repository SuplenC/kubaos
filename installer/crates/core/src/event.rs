//! The progress stream. `dotinstall run --events json` writes one `Event`
//! per line on stdout; the GUI reads exactly that.

use serde::{Deserialize, Serialize};

use crate::step::StepId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// First event. Steps in run order; weights sum to the overall bar.
    Plan {
        steps: Vec<StepInfo>,
    },
    StepStarted {
        id: StepId,
    },
    /// `fraction` in 0.0..=1.0 within the step when known.
    Progress {
        id: StepId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fraction: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    Log {
        id: StepId,
        level: Level,
        line: String,
    },
    StepFinished {
        id: StepId,
        elapsed_ms: u64,
    },
    /// Last event on failure. The process then exits 1.
    Failed {
        id: StepId,
        error: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hint: Option<String>,
    },
    /// Last event on success.
    Done {
        elapsed_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepInfo {
    pub id: StepId,
    pub title: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Event {
    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).expect("events always serialize")
    }

    pub fn from_json_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}
