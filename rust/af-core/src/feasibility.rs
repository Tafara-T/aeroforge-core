use crate::core_types::{RealitySnapshot, StateValue, AttributeId};
use crate::mission_model::{MissionModel, FeasibilityReport, ConstraintViolation, ConstraintOperator};
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[pyclass]
pub struct PyFeasibilityAnalyzer;

#[pymethods]
impl PyFeasibilityAnalyzer {
    #[new]
    pub fn new() -> Self {
        PyFeasibilityAnalyzer
    }

    pub fn evaluate_feasibility(&self, mission_json: String, snapshot_json: String) -> PyResult<String> {
        let mission: MissionModel = serde_json::from_str(&mission_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse mission: {}", e)))?;
        
        let snapshot: RealitySnapshot = serde_json::from_str(&snapshot_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse snapshot: {}", e)))?;

        let mut report = FeasibilityReport {
            mission_id: mission.mission_id.clone(),
            is_feasible: true,
            violations: Vec::new(),
            warnings: Vec::new(),
        };

        // Find the assigned entity in the snapshot
        let entity_state = snapshot.entities.iter().find(|e| e.entity_id == mission.assigned_entity);

        if entity_state.is_none() {
            report.is_feasible = false;
            report.warnings.push(format!("Assigned entity {:?} not found in reality snapshot", mission.assigned_entity.0));
        }

        // Evaluate constraints for each task
        for task in &mission.tasks {
            for constraint in &task.constraints {
                let actual_value = entity_state.and_then(|e| e.attributes.get(&constraint.attribute).cloned());
                
                let violated = match (&actual_value, &constraint.value) {
                    (Some(StateValue::Float { value: actual, .. }), StateValue::Float { value: required, .. }) => {
                        match constraint.operator {
                            ConstraintOperator::GreaterThan => actual <= required,
                            ConstraintOperator::LessThan => actual >= required,
                            ConstraintOperator::Equal => (actual - required).abs() > 0.001,
                            ConstraintOperator::NotEqual => (actual - required).abs() <= 0.001,
                        }
                    }
                    (None, _) => true, // Missing attribute is a violation if constrained
                    _ => false, // Simplified for Stage 2 initiation
                };

                if violated {
                    report.is_feasible = false;
                    let expected_str = match &constraint.value {
                        StateValue::Float { value, unit } => format!("{} {}", value, unit),
                        _ => format!("{:?}", constraint.value),
                    };
                    let actual_str = actual_value.as_ref().map(|v| match v {
                        StateValue::Float { value, unit } => format!("{} {}", value, unit),
                        _ => format!("{:?}", v),
                    });

                    report.violations.push(ConstraintViolation {
                        task_id: task.task_id.clone(),
                        attribute: constraint.attribute.clone(),
                        expected: expected_str,
                        actual: actual_str,
                        reason: format!("Constraint {:?} {:?} not met", constraint.attribute.0, constraint.operator),
                    });
                }
            }
        }

        serde_json::to_string(&report)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize report: {}", e)))
    }
}
