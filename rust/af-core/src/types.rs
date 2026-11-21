use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum AfValue {
    Numerical { magnitude: f64, unit: String },
    Categorical(String),
    Boolean(bool),
    Structured(serde_json::Value),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", content = "detail")]
pub enum UncertaintyLevel {
    Measured,
    Verified,
    Estimated,
    Predicted,
    Assumed,
    Unknown,
    Contradicted { conflicting_sources: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalMetadata {
    pub timestamp_observed: DateTime<Utc>,
    pub timestamp_ingested: DateTime<Utc>,
    pub validity_duration_ms: i64,
    pub freshness_threshold_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceMetadata {
    pub source_id: String,
    pub provenance_hash: String,
    pub calibration_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfStateUpdate {
    pub entity_id: String,
    pub variable_name: String,
    pub value: AfValue,
    pub temporal: TemporalMetadata,
    pub provenance: ProvenanceMetadata,
    pub uncertainty: UncertaintyLevel,
}
