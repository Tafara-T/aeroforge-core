use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Strongly typed identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttributeId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityType(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId(pub String);

// State values - extensible typed values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum StateValue {
    Boolean(bool),
    Integer(i64),
    Float { value: f64, unit: String },
    Text(String),
    Vector(Vec<f64>),
    Position(GeoPosition),
    Timestamp(DateTime<Utc>),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeoPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

// Generic entity state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
    pub schema_version: String,
    pub attributes: HashMap<AttributeId, StateValue>,
    pub metadata: StateMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    pub last_updated: DateTime<Utc>,
    pub source_references: Vec<SourceReference>,
    pub quality: StateQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReference {
    pub source_id: String,
    pub timestamp_observed: DateTime<Utc>,
    pub provenance_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", content = "detail")]
pub enum StateQuality {
    Valid,
    Stale,
    Contradicted { sources: Vec<String> },
    Unknown,
    Invalid { reason: String },
}

// Schema definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySchema {
    pub entity_type: EntityType,
    pub schema_version: String,
    pub attributes: HashMap<AttributeId, AttributeDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    pub value_type: String,
    pub unit: Option<String>,
    pub required: bool,
    pub range: Option<ValueRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

// Uncertainty policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyPolicy {
    pub entity_id: Option<EntityId>,
    pub attribute_id: AttributeId,
    pub tolerance: Option<f64>,
    pub source: ToleranceSource,
    pub validity: ValidityPeriod,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToleranceSource {
    Specification,
    Configuration,
    Mission,
    Default,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidityPeriod {
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", content = "detail")]
pub enum UncertaintyStatus {
    Known,
    Estimated,
    Unknown,
    Invalid { reason: String },
    Expired,
}

// Reality snapshot - dynamic aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealitySnapshot {
    pub snapshot_id: SnapshotId,
    pub captured_at: DateTime<Utc>,
    pub entities: Vec<EntityState>,
    pub provenance: Vec<SourceReference>,
    pub quality: StateQuality,
}
