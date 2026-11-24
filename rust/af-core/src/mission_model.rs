use crate::core_types::{EntityId, AttributeId, StateValue};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MissionId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TaskId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionObjective {
    pub description: String,
    pub success_criteria: HashMap<String, StateValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstraintOperator {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionConstraint {
    pub attribute: AttributeId,
    pub operator: ConstraintOperator,
    pub value: StateValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: TaskId,
    pub description: String,
    pub dependencies: Vec<TaskId>,
    pub constraints: Vec<MissionConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionModel {
    pub mission_id: MissionId,
    pub objective: MissionObjective,
    pub tasks: Vec<Task>,
    pub assigned_entity: EntityId,
    pub deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    pub task_id: TaskId,
    pub attribute: AttributeId,
    pub expected: String,
    pub actual: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeasibilityReport {
    pub mission_id: MissionId,
    pub is_feasible: bool,
    pub violations: Vec<ConstraintViolation>,
    pub warnings: Vec<String>,
}

// ============================================================================
// STAGE 3: PREDICTION SCHEDULING CONTRACTS (Backward Compatible)
// ============================================================================

/// Represents a physical movement between locations for a specific resource.
/// Kept separate from `Task` to preserve the Single Responsibility Principle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitSegment {
    pub segment_id: String,
    pub origin_task_id: Option<String>,      // None if starting from mission origin
    pub destination_task_id: Option<String>, // None if ending at mission destination
    pub distance_nm: f64,
    pub track_deg: f64, // True track in degrees (0.0 to 360.0)
    pub resource_id: String,
    pub dependencies: Vec<String>, // Task IDs or Segment IDs that must complete first
}

/// Represents the execution planning overlay for a specific task.
/// Decouples "what" (Task) from "who, how long, and when" (Schedule Entry).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskScheduleEntry {
    pub task_id: String,
    pub resource_id: String,
    pub estimated_duration_min: f64,
    pub dependencies: Vec<String>, // Task IDs or Segment IDs that must complete first
}