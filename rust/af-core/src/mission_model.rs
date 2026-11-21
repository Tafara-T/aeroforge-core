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
