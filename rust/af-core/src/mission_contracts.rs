use crate::core_types::{EntityId, AttributeId, StateValue, StateQuality, SourceReference, ValueRange};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// MODULE 1: ADVANCED MISSION CONSTRAINTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthorityType {
    Certification,
    Regulation,
    Operator,
    Mission,
    System,
    Design,
    Analysis,
    Operational,
    Historical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintAuthority {
    pub authority_type: AuthorityType,
    pub authority_reference: String,
    pub approval_provenance: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintEnforcement {
    HardGate {
        justification: String,
        authority: ConstraintAuthority,
    },
    RegulatoryGate {
        jurisdiction: String,
        regulation_reference: String,
        applicable_aircraft_types: Vec<String>,
        applicable_operations: Vec<String>,
        authority: ConstraintAuthority,
    },
    MissionRequirement {
        objective_link: String,
        authority: ConstraintAuthority,
    },
    SoftPreference {
        preference_weight: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintEvaluationOutcome {
    Satisfied,
    Violated {
        actual_value: StateValue,
        required_value: StateValue,
        evidence: Vec<SourceReference>,
    },
    MissingEvidence {
        required_attribute: AttributeId,
        required_entity: EntityId,
        reason: String,
    },
    UnreliableEvidence {
        attribute: AttributeId,
        quality: StateQuality,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintEvaluationResult {
    pub constraint_id: String,
    pub outcome: ConstraintEvaluationOutcome,
    pub enforcement: ConstraintEnforcement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockingReason {
    HardConstraintViolated { count: usize },
    RegulatoryConstraintViolated { count: usize },
    MissingEvidence { count: usize },
    UnreliableEvidence { count: usize },
    Multiple(Vec<BlockingReason>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateConstraintResult {
    pub hard_gate_violations: Vec<ConstraintEvaluationResult>,
    pub regulatory_gate_violations: Vec<ConstraintEvaluationResult>,
    pub mission_requirement_violations: Vec<ConstraintEvaluationResult>,
    pub soft_preference_violations: Vec<ConstraintEvaluationResult>,
    pub missing_evidence: Vec<ConstraintEvaluationResult>,
    pub unreliable_evidence: Vec<ConstraintEvaluationResult>,
    pub is_executable: bool,
    pub blocking_reason: Option<BlockingReason>,
}

// ============================================================================
// MODULE 2: PAYLOAD & LINK CAPABILITY MODELING
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UncertaintyType {
    BoundedInterval { lower: f64, upper: f64 },
    Gaussian { mean: f64, standard_deviation: f64 },
    SampleSet { samples: Vec<f64>, weights: Vec<f64> },
    Structured { dimensions: Vec<(String, Box<UncertaintyRepresentation>)> },
    Unquantified { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UncertaintyBasis {
    CalibrationCertificate { certificate_id: String, valid_until: DateTime<Utc> },
    StatisticalAnalysis { sample_count: usize, analysis_method: String },
    PhysicsModel { model_id: String, model_version: String, validation_evidence: String },
    ExpertJudgment { expert_id: String, rationale: String },
    Assumption { rationale: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correlation {
    pub correlated_quantity: String,
    pub correlation_coefficient: f64,
    pub correlation_basis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyRepresentation {
    pub representation_type: UncertaintyType,
    pub confidence_level: f64,
    pub basis: UncertaintyBasis,
    pub correlations: Vec<Correlation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityObservation {
    pub attribute: AttributeId,
    pub observed_value: StateValue,
    pub uncertainty: UncertaintyRepresentation,
    pub timestamp: DateTime<Utc>,
    pub source: SourceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySpecification {
    pub attribute: AttributeId,
    pub nominal_value: StateValue,
    pub operating_limits: ValueRange,
    pub specification_source: String,
    pub specification_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Validated { validation_evidence: String },
    Unvalidated { reason: String },
    Extrapolated { validation_evidence: String, extrapolation_rationale: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPrediction {
    pub attribute: AttributeId,
    pub predicted_value: StateValue,
    pub uncertainty: UncertaintyRepresentation,
    pub prediction_horizon_s: f64,
    pub model_id: String,
    pub model_version: String,
    pub input_observations: Vec<SourceReference>,
    pub known_limitations: Vec<String>,
    pub validation_status: ValidationStatus,
}

// ============================================================================
// MODULE 3: MISSION RISK ASSESSMENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskDimensionType {
    Safety,
    Operational,
    Technical,
    Regulatory,
    Effectiveness,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SeverityLevel {
    Negligible,
    Marginal,
    Critical,
    Catastrophic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LikelihoodLevel {
    Improbable,
    Remote,
    Occasional,
    Probable,
    Frequent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HazardEvidence {
    pub evidence_type: String,
    pub evidence_reference: String,
    pub evidence_summary: String,
    pub confidence_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockingAuthority {
    pub authority_type: AuthorityType,
    pub authority_reference: String,
    pub approval_provenance: String,
    pub approval_date: DateTime<Utc>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hazard {
    pub hazard_id: String,
    pub description: String,
    pub hazard_authority: ConstraintAuthority,
    pub evidence: Vec<HazardEvidence>,
    pub triggering_conditions: Vec<String>,
    pub affected_dimensions: Vec<RiskDimensionType>,
    pub severity: SeverityLevel,
    pub is_blocking: bool,
    pub blocking_authority: Option<BlockingAuthority>,
    pub blocking_rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalibrationMethod {
    HistoricalCalibration { sample_size: usize, calibration_evidence: String },
    SimulationCalibration { simulation_fidelity: String, validation_evidence: String },
    ExpertCalibration { expert_id: String, rationale: String },
    Uncalibrated { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceAssessment {
    pub confidence_level: f64,
    pub confidence_definition: String,
    pub calibration_method: CalibrationMethod,
    pub confidence_reducers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalJustification {
    pub is_justified: bool,
    pub justification_reason: String,
    pub sample_size: Option<usize>,
    pub validation_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedFailureMode {
    pub mode_id: String,
    pub description: String,
    pub affected_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionSuccessEstimate {
    pub estimate: Option<f64>,
    pub confidence_interval: Option<(f64, f64)>,
    pub statistical_justification: StatisticalJustification,
    pub correlated_failure_modes: Vec<CorrelatedFailureMode>,
    pub model_version: String,
    pub known_limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub mission_id: String,
    pub assessment_timestamp: DateTime<Utc>,
    pub safety_risk: String,
    pub operational_risk: String,
    pub technical_risk: String,
    pub regulatory_risk: String,
    pub effectiveness_risk: String,
    pub identified_hazards: Vec<Hazard>,
    pub blocking_hazards: Vec<Hazard>,
    pub mission_success_estimate: Option<MissionSuccessEstimate>,
    pub assessment_confidence: ConfidenceAssessment,
    pub known_hazard_gaps: Vec<String>,
    pub model_limitations: Vec<String>,
}

// ============================================================================
// MODULE 4: MISSION PLANNING & MULTI-OBJECTIVE OPTIMIZATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecompositionMethod {
    TemplateBased { template_id: String },
    SpatialCoverage { coverage_algorithm: String },
    TemporalSequencing { sequencing_rules: String },
    ResourceBased { resource_constraints: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDecompositionRules {
    pub max_decomposition_depth: usize,
    pub allowed_methods: Vec<DecompositionMethod>,
    pub prohibited_inventions: Vec<String>,
    pub require_approval_for_novel_tasks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelTaskApproval {
    pub task_id: String,
    pub approval_authority: String,
    pub approval_rationale: String,
    pub approval_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    pub predecessor: String,
    pub successor: String,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraph {
    pub tasks: Vec<crate::mission_model::Task>,
    pub dependencies: Vec<TaskDependency>,
    pub generation_method: DecompositionMethod,
    pub contains_novel_tasks: bool,
    pub novel_task_approvals: Vec<NovelTaskApproval>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ObjectiveType {
    MissionDuration,
    EnergyConsumption,
    PayloadEffectiveness,
    OperationalCost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveValue {
    pub value: f64,
    pub unit: String,
    pub uncertainty: Option<UncertaintyRepresentation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConsumption {
    pub energy_wh: f64,
    pub time_s: f64,
    pub cost_units: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionAlternative {
    pub alternative_id: String,
    pub task_graph: TaskGraph,
    pub objective_values: HashMap<ObjectiveType, ObjectiveValue>,
    pub constraint_evaluation: AggregateConstraintResult,
    pub risk_assessment: RiskAssessment,
    pub resource_consumption: ResourceConsumption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningMetadata {
    pub candidates_generated: usize,
    pub candidates_filtered_by_hard_constraints: usize,
    pub pareto_optimal_count: usize,
    pub objectives_considered: Vec<ObjectiveType>,
    pub planner_version: String,
    pub decomposition_rules: TaskDecompositionRules,
    pub known_limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningResult {
    pub mission_id: String,
    pub feasible_alternatives: Vec<MissionAlternative>,
    pub infeasible_alternatives: Vec<(MissionAlternative, Vec<ConstraintEvaluationResult>)>,
    pub pareto_front: Vec<MissionAlternative>,
    pub planning_metadata: PlanningMetadata,
}
