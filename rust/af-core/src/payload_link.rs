use crate::core_types::{EntityId, AttributeId, StateValue, ValueRange};
use crate::mission_contracts::{
    CapabilityObservation, CapabilitySpecification,
};
use serde_json::json;
use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[pyclass]
pub struct PyPayloadLinkModeler {
    specifications: HashMap<(EntityId, AttributeId), CapabilitySpecification>,
    observations: HashMap<(EntityId, AttributeId), Vec<CapabilityObservation>>,
}

#[pymethods]
impl PyPayloadLinkModeler {
    #[new]
    pub fn new() -> Self {
        PyPayloadLinkModeler {
            specifications: HashMap::new(),
            observations: HashMap::new(),
        }
    }

    pub fn register_specification(&mut self, entity_id: String, attribute_id: String, spec_json: String) -> PyResult<()> {
        let spec: CapabilitySpecification = serde_json::from_str(&spec_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse specification: {}", e)))?;
        self.specifications.insert((EntityId(entity_id), AttributeId(attribute_id)), spec);
        Ok(())
    }

    pub fn ingest_observation(&mut self, entity_id: String, attribute_id: String, obs_json: String) -> PyResult<()> {
        let obs: CapabilityObservation = serde_json::from_str(&obs_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse observation: {}", e)))?;
        
        let key = (EntityId(entity_id), AttributeId(attribute_id));
        self.observations.entry(key).or_insert_with(Vec::new).push(obs);
        Ok(())
    }

    pub fn evaluate_operating_limits(&self, entity_id: String, attribute_id: String) -> PyResult<String> {
        let key = (EntityId(entity_id), AttributeId(attribute_id));
        
        let spec = match self.specifications.get(&key) {
            Some(s) => s,
            None => return Err(PyValueError::new_err("Specification not found for this entity/attribute")),
        };

        let obs_list = match self.observations.get(&key) {
            Some(o) => o,
            None => return Err(PyValueError::new_err("No observations found for this entity/attribute")),
        };

        let mut results = Vec::new();
        for obs in obs_list {
            let is_within_limits = match (&spec.operating_limits, &obs.observed_value) {
                (ValueRange { min: Some(min), max: Some(max) }, StateValue::Float { value, .. }) => {
                    value >= min && value <= max
                }
                (ValueRange { min: Some(min), max: None }, StateValue::Float { value, .. }) => {
                    value >= min
                }
                (ValueRange { min: None, max: Some(max) }, StateValue::Float { value, .. }) => {
                    value <= max
                }
                _ => true, // Fallback for non-float or unbounded limits
            };

            results.push(json!({
                "timestamp": obs.timestamp,
                "observed_value": obs.observed_value,
                "is_within_limits": is_within_limits,
                "uncertainty": obs.uncertainty
            }));
        }

        serde_json::to_string(&results)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize results: {}", e)))
    }
}
