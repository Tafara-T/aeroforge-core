use crate::core_types::{RealitySnapshot, StateValue, StateQuality, EntityId, AttributeId};
use crate::mission_contracts::{
    ConstraintEnforcement, ConstraintEvaluationOutcome, ConstraintEvaluationResult,
    AggregateConstraintResult, BlockingReason,
};
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use serde::{Deserialize, Serialize};

// A simplified constraint definition for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluableConstraint {
    pub constraint_id: String,
    pub entity_id: String,
    pub attribute_id: String,
    pub required_value: StateValue,
    pub enforcement: ConstraintEnforcement,
}

#[pyclass]
pub struct PyAdvancedConstraintEvaluator;

#[pymethods]
impl PyAdvancedConstraintEvaluator {
    #[new]
    pub fn new() -> Self {
        PyAdvancedConstraintEvaluator
    }

    pub fn evaluate_constraints(
        &self,
        constraints_json: String,
        snapshot_json: String,
    ) -> PyResult<String> {
        let constraints: Vec<EvaluableConstraint> = serde_json::from_str(&constraints_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse constraints: {}", e)))?;
        
        let snapshot: RealitySnapshot = serde_json::from_str(&snapshot_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse snapshot: {}", e)))?;

        let mut result = AggregateConstraintResult {
            hard_gate_violations: Vec::new(),
            regulatory_gate_violations: Vec::new(),
            mission_requirement_violations: Vec::new(),
            soft_preference_violations: Vec::new(),
            missing_evidence: Vec::new(),
            unreliable_evidence: Vec::new(),
            is_executable: true,
            blocking_reason: None,
        };

        for constraint in constraints {
            let entity_id = EntityId(constraint.entity_id.clone());
            let attribute_id = AttributeId(constraint.attribute_id.clone());

            // Find the entity in the snapshot
            let entity = snapshot.entities.iter().find(|e| e.entity_id == entity_id);

            let evaluation_result = match entity {
                None => {
                    // Entity itself is missing
                    ConstraintEvaluationResult {
                        constraint_id: constraint.constraint_id.clone(),
                        outcome: ConstraintEvaluationOutcome::MissingEvidence {
                            required_attribute: attribute_id.clone(),
                            required_entity: entity_id.clone(),
                            reason: "Entity not found in reality snapshot".to_string(),
                        },
                        enforcement: constraint.enforcement.clone(),
                    }
                }
                Some(e) => {
                    // Entity exists, check the attribute
                    match e.attributes.get(&attribute_id) {
                        None => {
                            ConstraintEvaluationResult {
                                constraint_id: constraint.constraint_id.clone(),
                                outcome: ConstraintEvaluationOutcome::MissingEvidence {
                                    required_attribute: attribute_id.clone(),
                                    required_entity: entity_id.clone(),
                                    reason: "Attribute not found in entity state".to_string(),
                                },
                                enforcement: constraint.enforcement.clone(),
                            }
                        }
                        Some(state_value) => {
                            // Check state quality first
                            if e.metadata.quality == StateQuality::Unknown || 
                               matches!(e.metadata.quality, StateQuality::Contradicted { .. }) {
                                ConstraintEvaluationResult {
                                    constraint_id: constraint.constraint_id.clone(),
                                    outcome: ConstraintEvaluationOutcome::UnreliableEvidence {
                                        attribute: attribute_id.clone(),
                                        quality: e.metadata.quality.clone(),
                                        reason: "State quality is Unknown or Contradicted".to_string(),
                                    },
                                    enforcement: constraint.enforcement.clone(),
                                }
                            } else {
                                // Evaluate the constraint (simplified for Stage 2: exact match or basic comparison)
                                // For this test, we'll do a simple equality check for demonstration
                                let is_satisfied = match (&constraint.required_value, state_value) {
                                    (StateValue::Float { value: req, .. }, StateValue::Float { value: actual, .. }) => {
                                        (actual - req).abs() < 0.001 // Tolerance for float comparison
                                    }
                                    (StateValue::Text(req), StateValue::Text(actual)) => req == actual,
                                    (StateValue::Boolean(req), StateValue::Boolean(actual)) => req == actual,
                                    _ => false, // Fallback for unimplemented types in this demo
                                };

                                if is_satisfied {
                                    ConstraintEvaluationResult {
                                        constraint_id: constraint.constraint_id.clone(),
                                        outcome: ConstraintEvaluationOutcome::Satisfied,
                                        enforcement: constraint.enforcement.clone(),
                                    }
                                } else {
                                    ConstraintEvaluationResult {
                                        constraint_id: constraint.constraint_id.clone(),
                                        outcome: ConstraintEvaluationOutcome::Violated {
                                            actual_value: state_value.clone(),
                                            required_value: constraint.required_value.clone(),
                                            evidence: e.metadata.source_references.clone(),
                                        },
                                        enforcement: constraint.enforcement.clone(),
                                    }
                                }
                            }
                        }
                    }
                }
            };

            // Categorize the result
            match &evaluation_result.outcome {
                ConstraintEvaluationOutcome::Satisfied => {}
                ConstraintEvaluationOutcome::Violated { .. } => {
                    match &evaluation_result.enforcement {
                        ConstraintEnforcement::HardGate { .. } => result.hard_gate_violations.push(evaluation_result),
                        ConstraintEnforcement::RegulatoryGate { .. } => result.regulatory_gate_violations.push(evaluation_result),
                        ConstraintEnforcement::MissionRequirement { .. } => result.mission_requirement_violations.push(evaluation_result),
                        ConstraintEnforcement::SoftPreference { .. } => result.soft_preference_violations.push(evaluation_result),
                    }
                }
                ConstraintEvaluationOutcome::MissingEvidence { .. } => {
                    result.missing_evidence.push(evaluation_result);
                }
                ConstraintEvaluationOutcome::UnreliableEvidence { .. } => {
                    result.unreliable_evidence.push(evaluation_result);
                }
            }
        }

        // Determine executability and blocking reason
        let mut blocking_reasons = Vec::new();
        
        if !result.hard_gate_violations.is_empty() {
            blocking_reasons.push(BlockingReason::HardConstraintViolated { count: result.hard_gate_violations.len() });
        }
        if !result.regulatory_gate_violations.is_empty() {
            blocking_reasons.push(BlockingReason::RegulatoryConstraintViolated { count: result.regulatory_gate_violations.len() });
        }
        if !result.missing_evidence.is_empty() {
            blocking_reasons.push(BlockingReason::MissingEvidence { count: result.missing_evidence.len() });
        }
        if !result.unreliable_evidence.is_empty() {
            blocking_reasons.push(BlockingReason::UnreliableEvidence { count: result.unreliable_evidence.len() });
        }

        result.is_executable = blocking_reasons.is_empty();
        result.blocking_reason = match blocking_reasons.len() {
            0 => None,
            1 => Some(blocking_reasons.remove(0)),
            _ => Some(BlockingReason::Multiple(blocking_reasons)),
        };

        serde_json::to_string(&result)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize result: {}", e)))
    }
}
