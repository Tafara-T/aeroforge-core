use crate::mission_contracts::{
    PlanningResult, MissionAlternative, PlanningMetadata, TaskGraph, TaskDependency,
    DecompositionMethod, TaskDecompositionRules, ObjectiveType, ObjectiveValue,
    ResourceConsumption, AggregateConstraintResult, RiskAssessment, ConfidenceAssessment, CalibrationMethod,
};
use crate::mission_model::Task;
use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[pyclass]
pub struct PyMissionPlanner {
    decomposition_rules: TaskDecompositionRules,
}

#[pymethods]
impl PyMissionPlanner {
    #[new]
    pub fn new() -> Self {
        PyMissionPlanner {
            decomposition_rules: TaskDecompositionRules {
                max_decomposition_depth: 3,
                allowed_methods: vec![DecompositionMethod::TemplateBased { template_id: "default".to_string() }],
                prohibited_inventions: vec!["autonomous_waypoint_generation".to_string()],
                require_approval_for_novel_tasks: true,
            },
        }
    }

    pub fn generate_plan(
        &self,
        mission_id: String,
        constraint_results_json: String,
        risk_assessments_json: String,
    ) -> PyResult<String> {
        let constraint_results: Vec<AggregateConstraintResult> = serde_json::from_str(&constraint_results_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse constraint results: {}", e)))?;
        
        let risk_assessments: Vec<RiskAssessment> = serde_json::from_str(&risk_assessments_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse risk assessments: {}", e)))?;

        let mut feasible_alternatives = Vec::new();
        let mut infeasible_alternatives = Vec::new();

        for i in 0..3 {
            let alternative_id = format!("alt_{}", i + 1);
            
            let task_graph = TaskGraph {
                tasks: vec![
                    Task {
                        task_id: crate::mission_model::TaskId(format!("task_{}_1", i + 1)),
                        description: format!("Task 1 for alternative {}", i + 1),
                        dependencies: vec![],
                        constraints: vec![],
                    },
                    Task {
                        task_id: crate::mission_model::TaskId(format!("task_{}_2", i + 1)),
                        description: format!("Task 2 for alternative {}", i + 1),
                        dependencies: vec![crate::mission_model::TaskId(format!("task_{}_1", i + 1))],
                        constraints: vec![],
                    },
                ],
                dependencies: vec![
                    TaskDependency {
                        predecessor: format!("task_{}_1", i + 1),
                        successor: format!("task_{}_2", i + 1),
                        dependency_type: "FinishToStart".to_string(),
                    },
                ],
                generation_method: DecompositionMethod::TemplateBased { template_id: "default".to_string() },
                contains_novel_tasks: false,
                novel_task_approvals: vec![],
            };

            let mut objective_values = HashMap::new();
            match i {
                0 => {
                    objective_values.insert(ObjectiveType::MissionDuration, ObjectiveValue { value: 30.0, unit: "minutes".to_string(), uncertainty: None });
                    objective_values.insert(ObjectiveType::EnergyConsumption, ObjectiveValue { value: 500.0, unit: "Wh".to_string(), uncertainty: None });
                    objective_values.insert(ObjectiveType::OperationalCost, ObjectiveValue { value: 150.0, unit: "USD".to_string(), uncertainty: None });
                }
                1 => {
                    objective_values.insert(ObjectiveType::MissionDuration, ObjectiveValue { value: 45.0, unit: "minutes".to_string(), uncertainty: None });
                    objective_values.insert(ObjectiveType::EnergyConsumption, ObjectiveValue { value: 350.0, unit: "Wh".to_string(), uncertainty: None });
                    objective_values.insert(ObjectiveType::OperationalCost, ObjectiveValue { value: 100.0, unit: "USD".to_string(), uncertainty: None });
                }
                2 => {
                    objective_values.insert(ObjectiveType::MissionDuration, ObjectiveValue { value: 60.0, unit: "minutes".to_string(), uncertainty: None });
                    objective_values.insert(ObjectiveType::EnergyConsumption, ObjectiveValue { value: 250.0, unit: "Wh".to_string(), uncertainty: None });
                    objective_values.insert(ObjectiveType::OperationalCost, ObjectiveValue { value: 80.0, unit: "USD".to_string(), uncertainty: None });
                }
                _ => unreachable!(),
            }

            let constraint_evaluation = if i < constraint_results.len() {
                constraint_results[i].clone()
            } else {
                AggregateConstraintResult {
                    hard_gate_violations: vec![],
                    regulatory_gate_violations: vec![],
                    mission_requirement_violations: vec![],
                    soft_preference_violations: vec![],
                    missing_evidence: vec![],
                    unreliable_evidence: vec![],
                    is_executable: true,
                    blocking_reason: None,
                }
            };

            let risk_assessment = if i < risk_assessments.len() {
                risk_assessments[i].clone()
            } else {
                RiskAssessment {
                    mission_id: mission_id.clone(),
                    assessment_timestamp: chrono::Utc::now(),
                    safety_risk: "Low".to_string(),
                    operational_risk: "Low".to_string(),
                    technical_risk: "Low".to_string(),
                    regulatory_risk: "Negligible".to_string(),
                    effectiveness_risk: "Low".to_string(),
                    identified_hazards: vec![],
                    blocking_hazards: vec![],
                    mission_success_estimate: None,
                    assessment_confidence: ConfidenceAssessment {
                        confidence_level: 0.8,
                        confidence_definition: "Demo confidence".to_string(),
                        calibration_method: CalibrationMethod::Uncalibrated { reason: "Demo".to_string() },
                        confidence_reducers: vec![],
                    },
                    known_hazard_gaps: vec![],
                    model_limitations: vec![],
                }
            };

            let resource_consumption = ResourceConsumption {
                energy_wh: objective_values.get(&ObjectiveType::EnergyConsumption).map(|v| v.value).unwrap_or(0.0),
                time_s: objective_values.get(&ObjectiveType::MissionDuration).map(|v| v.value * 60.0).unwrap_or(0.0),
                cost_units: objective_values.get(&ObjectiveType::OperationalCost).map(|v| v.value).unwrap_or(0.0),
            };

            let alternative = MissionAlternative {
                alternative_id: alternative_id.clone(),
                task_graph,
                objective_values,
                constraint_evaluation: constraint_evaluation.clone(),
                risk_assessment,
                resource_consumption,
            };

            if constraint_evaluation.is_executable {
                feasible_alternatives.push(alternative);
            } else {
                infeasible_alternatives.push((alternative, constraint_evaluation.hard_gate_violations));
            }
        }

        let pareto_front = compute_pareto_front(&feasible_alternatives);

        let planning_metadata = PlanningMetadata {
            candidates_generated: 3,
            candidates_filtered_by_hard_constraints: infeasible_alternatives.len(),
            pareto_optimal_count: pareto_front.len(),
            objectives_considered: vec![
                ObjectiveType::MissionDuration,
                ObjectiveType::EnergyConsumption,
                ObjectiveType::OperationalCost,
            ],
            planner_version: "1.0.0".to_string(),
            decomposition_rules: self.decomposition_rules.clone(),
            known_limitations: vec!["Demo planner with fixed alternatives".to_string()],
        };

        let result = PlanningResult {
            mission_id,
            feasible_alternatives,
            infeasible_alternatives,
            pareto_front,
            planning_metadata,
        };

        serde_json::to_string(&result)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize result: {}", e)))
    }
}

// Pure Rust helper function, completely outside the #[pymethods] block
fn compute_pareto_front(alternatives: &[MissionAlternative]) -> Vec<MissionAlternative> {
    let mut pareto_front = Vec::new();

    for (i, alt_i) in alternatives.iter().enumerate() {
        let mut is_dominated = false;

        for (j, alt_j) in alternatives.iter().enumerate() {
            if i == j { continue; }

            let objectives = [
                ObjectiveType::MissionDuration,
                ObjectiveType::EnergyConsumption,
                ObjectiveType::OperationalCost,
            ];

            let mut all_better_or_equal = true;
            let mut at_least_one_strictly_better = false;

            for obj in &objectives {
                let val_i = alt_i.objective_values.get(obj).map(|v| v.value).unwrap_or(f64::MAX);
                let val_j = alt_j.objective_values.get(obj).map(|v| v.value).unwrap_or(f64::MAX);

                if val_j > val_i {
                    all_better_or_equal = false;
                    break;
                }
                if val_j < val_i {
                    at_least_one_strictly_better = true;
                }
            }

            if all_better_or_equal && at_least_one_strictly_better {
                is_dominated = true;
                break;
            }
        }

        if !is_dominated {
            pareto_front.push(alt_i.clone());
        }
    }

    pareto_front
}
