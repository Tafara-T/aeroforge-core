use crate::core_types::{AttributeId, GeoPosition, SourceReference, StateValue};
use crate::mission_contracts::{
    CapabilityPrediction, UncertaintyBasis, UncertaintyRepresentation,
    UncertaintyType, ValidationStatus,
};
use crate::mission_model::{Task, TaskScheduleEntry, TransitSegment};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// INPUT TYPES FOR PREDICTION FUNCTIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelState {
    pub tank_id: String,
    pub contents_lbs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineFlowState {
    pub engine_id: String,
    pub flow_pps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioConfig {
    pub freq_mhz: f64,
    pub tx_power_dbm: f64,
    pub tx_gain_dbi: f64,
    pub rx_gain_dbi: f64,
    pub rx_sensitivity_dbm: f64,
    pub fade_margin_db: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindVector {
    pub speed_kts: f64,
    pub direction_deg: f64,
}

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

fn is_valid_finite(v: f64) -> bool {
    v.is_finite()
}

fn is_valid_non_negative(v: f64) -> bool {
    v.is_finite() && v >= 0.0
}

fn is_valid_positive(v: f64) -> bool {
    v.is_finite() && v > 0.0
}

fn is_valid_heading(v: f64) -> bool {
    v.is_finite() && (0.0..=360.0).contains(&v)
}

fn make_null_prediction(
    attribute: &str,
    reason: &str,
    model_id: &str,
    model_version: &str,
    input_obs: Vec<SourceReference>,
) -> CapabilityPrediction {
    CapabilityPrediction {
        attribute: AttributeId(attribute.to_string()),
        predicted_value: StateValue::Null,
        uncertainty: UncertaintyRepresentation {
            representation_type: UncertaintyType::Unquantified {
                reason: "Insufficient valid input data".to_string(),
            },
            confidence_level: 0.0,
            basis: UncertaintyBasis::Assumption {
                rationale: "No valid inputs available".to_string(),
            },
            correlations: vec![],
        },
        prediction_horizon_s: 0.0,
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        input_observations: input_obs,
        known_limitations: vec![reason.to_string()],
        validation_status: ValidationStatus::Unvalidated {
            reason: reason.to_string(),
        },
    }
}

// ============================================================================
// MODULE A: FUEL ENDURANCE PREDICTION
// ============================================================================

pub fn predict_fuel_endurance(
    tanks: &[FuelState],
    engines: &[EngineFlowState],
    reserve_lbs: f64,
    input_obs: Vec<SourceReference>,
) -> CapabilityPrediction {
    let model_id = "fuel_endurance_v1";
    let model_version = "1.0.0";

    if tanks.is_empty() {
        return make_null_prediction(
            "remaining_flight_time_min",
            "No fuel tank data provided",
            model_id,
            model_version,
            input_obs,
        );
    }
    for tank in tanks {
        if !is_valid_non_negative(tank.contents_lbs) {
            return make_null_prediction(
                "remaining_flight_time_min",
                &format!("Invalid fuel tank data for tank {}", tank.tank_id),
                model_id,
                model_version,
                input_obs,
            );
        }
    }

    if engines.is_empty() {
        return make_null_prediction(
            "remaining_flight_time_min",
            "No engine flow data provided",
            model_id,
            model_version,
            input_obs,
        );
    }
    for engine in engines {
        if !is_valid_non_negative(engine.flow_pps) {
            return make_null_prediction(
                "remaining_flight_time_min",
                &format!("Invalid engine flow data for engine {}", engine.engine_id),
                model_id,
                model_version,
                input_obs,
            );
        }
    }

    if !is_valid_non_negative(reserve_lbs) {
        return make_null_prediction(
            "remaining_flight_time_min",
            "Invalid reserve fuel configuration",
            model_id,
            model_version,
            input_obs,
        );
    }

    let total_fuel: f64 = tanks.iter().map(|t| t.contents_lbs).sum();
    let total_flow_pps: f64 = engines.iter().map(|e| e.flow_pps).sum();

    if total_flow_pps <= 0.0 {
        return make_null_prediction(
            "remaining_flight_time_min",
            "Total fuel flow is zero or negative",
            model_id,
            model_version,
            input_obs,
        );
    }

    if total_fuel <= reserve_lbs {
        return make_null_prediction(
            "remaining_flight_time_min",
            "Fuel at or below reserve minimum",
            model_id,
            model_version,
            input_obs,
        );
    }

    let usable_fuel = total_fuel - reserve_lbs;
    let endurance_min = usable_fuel / (total_flow_pps * 60.0);

    CapabilityPrediction {
        attribute: AttributeId("remaining_flight_time_min".to_string()),
        predicted_value: StateValue::Float {
            value: (endurance_min * 100.0).round() / 100.0,
            unit: "min".to_string(),
        },
        uncertainty: UncertaintyRepresentation {
            representation_type: UncertaintyType::BoundedInterval {
                lower: endurance_min * 0.85,
                upper: endurance_min * 1.15,
            },
            confidence_level: 0.85,
            basis: UncertaintyBasis::Assumption {
                rationale: "Heuristic +/- 15% bounding box; assumes constant flow profile".to_string(),
            },
            correlations: vec![],
        },
        prediction_horizon_s: endurance_min * 60.0,
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        input_observations: input_obs,
        known_limitations: vec![
            "Assumes constant flow profile. Does not model sudden headwinds or reserve fuel mandates.".to_string(),
        ],
        validation_status: ValidationStatus::Unvalidated {
            reason: "Heuristic bounding box, not statistically calibrated".to_string(),
        },
    }
}

// ============================================================================
// MODULE B: LINK RELIABILITY PREDICTION
// ============================================================================

pub fn predict_link_reliability(
    ac_pos: &GeoPosition,
    gs_pos: &GeoPosition,
    radio: &RadioConfig,
    input_obs: Vec<SourceReference>,
) -> CapabilityPrediction {
    let model_id = "link_reliability_v1";
    let model_version = "1.0.0";

    if !is_valid_positive(radio.freq_mhz) {
        return make_null_prediction(
            "link_degradation_risk_score",
            "Frequency must be strictly positive",
            model_id,
            model_version,
            input_obs,
        );
    }

    for (name, pos) in [("aircraft", ac_pos), ("ground_station", gs_pos)] {
        if !is_valid_finite(pos.latitude)
            || !is_valid_finite(pos.longitude)
            || !is_valid_finite(pos.altitude)
        {
            return make_null_prediction(
                "link_degradation_risk_score",
                &format!("Invalid {} position data", name),
                model_id,
                model_version,
                input_obs,
            );
        }
    }

    // 3D slant range (Haversine surface + altitude difference)
    let r_km = 6371.0;
    let dlat = (ac_pos.latitude - gs_pos.latitude).to_radians();
    let dlon = (ac_pos.longitude - gs_pos.longitude).to_radians();
    let lat1_rad = gs_pos.latitude.to_radians();
    let lat2_rad = ac_pos.latitude.to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    let surface_dist_km = r_km * c;

    // GeoPosition.altitude is in meters per core_types.rs
    let alt_diff_km = (ac_pos.altitude - gs_pos.altitude) / 1000.0;
    let slant_dist_km = (surface_dist_km.powi(2) + alt_diff_km.powi(2)).sqrt();

    if slant_dist_km < 0.001 {
        return make_null_prediction(
            "link_degradation_risk_score",
            "Aircraft and ground station co-located; FSPL model invalid",
            model_id,
            model_version,
            input_obs,
        );
    }

    // Terrain screening (conservative): aircraft alt < GS alt + 10m
    if ac_pos.altitude < gs_pos.altitude + 10.0 {
        return make_null_prediction(
            "link_degradation_risk_score",
            "Potential terrain obstruction; FSPL model outside valid operating conditions",
            model_id,
            model_version,
            input_obs,
        );
    }

    let fspl_db = 20.0 * slant_dist_km.log10() + 20.0 * radio.freq_mhz.log10() + 32.44;
    let margin_db = radio.tx_power_dbm + radio.tx_gain_dbi + radio.rx_gain_dbi
        - fspl_db
        - radio.rx_sensitivity_dbm;

    let risk_score = if margin_db >= radio.fade_margin_db {
        0.05
    } else if margin_db >= 0.0 {
        0.05 + 0.90 * (1.0 - (margin_db / radio.fade_margin_db))
    } else {
        0.95
    };

    CapabilityPrediction {
        attribute: AttributeId("link_degradation_risk_score".to_string()),
        predicted_value: StateValue::Float {
            value: (risk_score * 1000.0).round() / 1000.0,
            unit: "probability".to_string(),
        },
        uncertainty: UncertaintyRepresentation {
            representation_type: UncertaintyType::Unquantified {
                reason: "Heuristic link-margin proxy, not empirically calibrated packet-loss probability".to_string(),
            },
            confidence_level: 0.0,
            basis: UncertaintyBasis::Assumption {
                rationale: "Uncalibrated heuristic; uses FSPL free-space assumption".to_string(),
            },
            correlations: vec![],
        },
        prediction_horizon_s: 60.0,
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        input_observations: input_obs,
        known_limitations: vec![
            "Assumes GeoPosition altitude matches WGS84 MSL datum. Geoid undulation differences are not modeled.".to_string(),
            "Terrain buffer is a conservative geometric screening rule, not a verified terrain clearance.".to_string(),
        ],
        validation_status: ValidationStatus::Unvalidated {
            reason: "Heuristic risk score, not calibrated probability".to_string(),
        },
    }
}

// ============================================================================
// MODULE C: MISSION DURATION PREDICTION (DAG-based CPM)
// ============================================================================

struct ScheduleNode {
    id: String,
    duration_min: f64,
    resource_id: String,
    dependencies: Vec<String>,
}

pub fn predict_mission_duration(
    tasks: &[Task],
    transits: &[TransitSegment],
    schedule: &[TaskScheduleEntry],
    tas_kts: f64,
    wind: &WindVector,
    input_obs: Vec<SourceReference>,
) -> CapabilityPrediction {
    let model_id = "mission_duration_v1";
    let model_version = "1.0.0";

    // Suppress unused warning for tasks (kept for API symmetry; scheduling overlay drives durations)
    let _ = tasks;

    if !is_valid_positive(tas_kts) {
        return make_null_prediction(
            "total_mission_duration_min",
            "Invalid or missing true airspeed",
            model_id,
            model_version,
            input_obs,
        );
    }

    if !is_valid_non_negative(wind.speed_kts) {
        return make_null_prediction(
            "total_mission_duration_min",
            "Invalid wind speed",
            model_id,
            model_version,
            input_obs,
        );
    }

    if !is_valid_heading(wind.direction_deg) {
        return make_null_prediction(
            "total_mission_duration_min",
            "Invalid wind direction",
            model_id,
            model_version,
            input_obs,
        );
    }

    for transit in transits {
        if !is_valid_non_negative(transit.distance_nm) {
            return make_null_prediction(
                "total_mission_duration_min",
                &format!("Invalid transit distance for segment {}", transit.segment_id),
                model_id,
                model_version,
                input_obs,
            );
        }
        if !is_valid_heading(transit.track_deg) {
            return make_null_prediction(
                "total_mission_duration_min",
                &format!("Invalid track heading for segment {}", transit.segment_id),
                model_id,
                model_version,
                input_obs,
            );
        }
        if transit.resource_id.is_empty() {
            return make_null_prediction(
                "total_mission_duration_min",
                "Missing resource assignment for transit segment",
                model_id,
                model_version,
                input_obs,
            );
        }
    }

    for entry in schedule {
        if !is_valid_non_negative(entry.estimated_duration_min) {
            return make_null_prediction(
                "total_mission_duration_min",
                &format!("Invalid duration for task {}", entry.task_id),
                model_id,
                model_version,
                input_obs,
            );
        }
        if entry.resource_id.is_empty() {
            return make_null_prediction(
                "total_mission_duration_min",
                "Missing resource assignment for task",
                model_id,
                model_version,
                input_obs,
            );
        }
    }

    // Build unified DAG nodes
    let mut nodes: Vec<ScheduleNode> = Vec::new();

    for entry in schedule {
        nodes.push(ScheduleNode {
            id: format!("task:{}", entry.task_id),
            duration_min: entry.estimated_duration_min,
            resource_id: entry.resource_id.clone(),
            dependencies: entry.dependencies.clone(),
        });
    }

    let wind_rad = wind.direction_deg.to_radians();

    for transit in transits {
        let track_rad = transit.track_deg.to_radians();
        let v_wind_along = -wind.speed_kts * (wind_rad - track_rad).cos();
        let v_wind_cross = wind.speed_kts * (wind_rad - track_rad).sin();

        if v_wind_cross.abs() >= tas_kts {
            return make_null_prediction(
                "total_mission_duration_min",
                "Crosswind exceeds true airspeed; track maintenance infeasible",
                model_id,
                model_version,
                input_obs,
            );
        }

        let v_ground = (tas_kts.powi(2) - v_wind_cross.powi(2)).sqrt() + v_wind_along;

        if v_ground <= 0.0 {
            return make_null_prediction(
                "total_mission_duration_min",
                "Adverse wind exceeds true airspeed; forward progress infeasible",
                model_id,
                model_version,
                input_obs,
            );
        }

        let transit_time_min = (transit.distance_nm / v_ground) * 60.0;

        nodes.push(ScheduleNode {
            id: format!("transit:{}", transit.segment_id),
            duration_min: transit_time_min,
            resource_id: transit.resource_id.clone(),
            dependencies: transit.dependencies.clone(),
        });
    }

    // Handle empty schedule
    if nodes.is_empty() {
        return CapabilityPrediction {
            attribute: AttributeId("total_mission_duration_min".to_string()),
            predicted_value: StateValue::Float {
                value: 0.0,
                unit: "min".to_string(),
            },
            uncertainty: UncertaintyRepresentation {
                representation_type: UncertaintyType::BoundedInterval {
                    lower: 0.0,
                    upper: 0.0,
                },
                confidence_level: 1.0,
                basis: UncertaintyBasis::Assumption {
                    rationale: "Empty task and transit collections".to_string(),
                },
                correlations: vec![],
            },
            prediction_horizon_s: 0.0,
            model_id: model_id.to_string(),
            model_version: model_version.to_string(),
            input_observations: input_obs,
            known_limitations: vec!["No tasks or transits to schedule".to_string()],
            validation_status: ValidationStatus::Unvalidated {
                reason: "Empty schedule".to_string(),
            },
        };
    }

    // Kahn's algorithm: topological sort with cycle detection
    let id_to_idx: HashMap<String, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), i))
        .collect();

    let mut in_degree: Vec<usize> = vec![0; nodes.len()];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];

    for (idx, node) in nodes.iter().enumerate() {
        for dep in &node.dependencies {
            if let Some(&dep_idx) = id_to_idx.get(dep) {
                adj[dep_idx].push(idx);
                in_degree[idx] += 1;
            }
        }
    }

    let mut queue: Vec<usize> = (0..nodes.len())
        .filter(|&i| in_degree[i] == 0)
        .collect();
    let mut topo_order: Vec<usize> = Vec::with_capacity(nodes.len());

    while let Some(idx) = queue.pop() {
        topo_order.push(idx);
        for &next_idx in &adj[idx] {
            in_degree[next_idx] -= 1;
            if in_degree[next_idx] == 0 {
                queue.push(next_idx);
            }
        }
    }

    if topo_order.len() != nodes.len() {
        return make_null_prediction(
            "total_mission_duration_min",
            "Cyclic dependencies detected in schedule",
            model_id,
            model_version,
            input_obs,
        );
    }

    // Calculate earliest end times with resource serialization
    let mut end_times: Vec<f64> = vec![0.0; nodes.len()];
    let mut resource_end_times: HashMap<String, f64> = HashMap::new();

    for &idx in &topo_order {
        let node = &nodes[idx];

        let mut earliest_start: f64 = 0.0;
        for dep in &node.dependencies {
            if let Some(&dep_idx) = id_to_idx.get(dep) {
                earliest_start = earliest_start.max(end_times[dep_idx]);
            }
        }

        let resource_available = resource_end_times
            .get(&node.resource_id)
            .copied()
            .unwrap_or(0.0);
        let actual_start = earliest_start.max(resource_available);
        let actual_end = actual_start + node.duration_min;

        end_times[idx] = actual_end;
        resource_end_times.insert(node.resource_id.clone(), actual_end);
    }

    let total_duration_min = end_times.iter().copied().fold(0.0_f64, f64::max);

    CapabilityPrediction {
        attribute: AttributeId("total_mission_duration_min".to_string()),
        predicted_value: StateValue::Float {
            value: (total_duration_min * 100.0).round() / 100.0,
            unit: "min".to_string(),
        },
        uncertainty: UncertaintyRepresentation {
            representation_type: UncertaintyType::BoundedInterval {
                lower: total_duration_min * 0.90,
                upper: total_duration_min * 1.10,
            },
            confidence_level: 0.80,
            basis: UncertaintyBasis::Assumption {
                rationale: "Heuristic +/- 10% bounding box for ATC delays and variable winds".to_string(),
            },
            correlations: vec![],
        },
        prediction_horizon_s: total_duration_min * 60.0,
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        input_observations: input_obs,
        known_limitations: vec![
            "Assumes 1 unit of capacity per resource_id. Does not model complex resource pooling.".to_string(),
        ],
        validation_status: ValidationStatus::Unvalidated {
            reason: "Heuristic bounding box, not statistically calibrated".to_string(),
        },
    }
}

// ============================================================================
// PyO3 BINDINGS
// ============================================================================

#[pyclass]
pub struct PyPredictionEngine {}

#[pymethods]
impl PyPredictionEngine {
    #[new]
    pub fn new() -> Self {
        PyPredictionEngine {}
    }

    pub fn predict_fuel_endurance_json(
        &self,
        tanks_json: String,
        engines_json: String,
        reserve_lbs: f64,
        input_obs_json: String,
    ) -> PyResult<String> {
        let tanks: Vec<FuelState> = serde_json::from_str(&tanks_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse tanks: {}", e)))?;
        let engines: Vec<EngineFlowState> = serde_json::from_str(&engines_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse engines: {}", e)))?;
        let input_obs: Vec<SourceReference> = serde_json::from_str(&input_obs_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse input observations: {}", e)))?;

        let result = predict_fuel_endurance(&tanks, &engines, reserve_lbs, input_obs);
        serde_json::to_string(&result)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize result: {}", e)))
    }

    pub fn predict_link_reliability_json(
        &self,
        ac_pos_json: String,
        gs_pos_json: String,
        radio_json: String,
        input_obs_json: String,
    ) -> PyResult<String> {
        let ac_pos: GeoPosition = serde_json::from_str(&ac_pos_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse aircraft position: {}", e)))?;
        let gs_pos: GeoPosition = serde_json::from_str(&gs_pos_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse ground station position: {}", e)))?;
        let radio: RadioConfig = serde_json::from_str(&radio_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse radio config: {}", e)))?;
        let input_obs: Vec<SourceReference> = serde_json::from_str(&input_obs_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse input observations: {}", e)))?;

        let result = predict_link_reliability(&ac_pos, &gs_pos, &radio, input_obs);
        serde_json::to_string(&result)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize result: {}", e)))
    }

    pub fn predict_mission_duration_json(
        &self,
        tasks_json: String,
        transits_json: String,
        schedule_json: String,
        tas_kts: f64,
        wind_json: String,
        input_obs_json: String,
    ) -> PyResult<String> {
        let tasks: Vec<Task> = serde_json::from_str(&tasks_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse tasks: {}", e)))?;
        let transits: Vec<TransitSegment> = serde_json::from_str(&transits_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse transits: {}", e)))?;
        let schedule: Vec<TaskScheduleEntry> = serde_json::from_str(&schedule_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse schedule: {}", e)))?;
        let wind: WindVector = serde_json::from_str(&wind_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse wind: {}", e)))?;
        let input_obs: Vec<SourceReference> = serde_json::from_str(&input_obs_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse input observations: {}", e)))?;

        let result = predict_mission_duration(&tasks, &transits, &schedule, tas_kts, &wind, input_obs);
        serde_json::to_string(&result)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize result: {}", e)))
    }
}