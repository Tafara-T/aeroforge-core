use crate::core_types::{
    EntityId, AttributeId, UncertaintyPolicy, ToleranceSource, ValidityPeriod,
    UncertaintyStatus, StateQuality,
};
use crate::types::AfStateUpdate;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[derive(Debug)]
pub struct UncertaintyManager {
    policies: HashMap<(Option<EntityId>, AttributeId), UncertaintyPolicy>,
    current_state: HashMap<(EntityId, AttributeId), (f64, DateTime<Utc>)>,
    contradictions: HashMap<(EntityId, AttributeId), Vec<String>>,
}

impl UncertaintyManager {
    pub fn new() -> Self {
        UncertaintyManager {
            policies: HashMap::new(),
            current_state: HashMap::new(),
            contradictions: HashMap::new(),
        }
    }

    pub fn register_policy(&mut self, policy: UncertaintyPolicy) {
        let key = (policy.entity_id.clone(), policy.attribute_id.clone());
        self.policies.insert(key, policy);
    }

    pub fn get_policy(&self, entity_id: &str, attribute_id: &str) -> Option<&UncertaintyPolicy> {
        let eid = EntityId(entity_id.to_string());
        let aid = AttributeId(attribute_id.to_string());
        
        // Try specific policy first
        if let Some(policy) = self.policies.get(&(Some(eid.clone()), aid.clone())) {
            return Some(policy);
        }
        
        // Fall back to generic policy
        self.policies.get(&(None, aid))
    }

    pub fn validate_temporal(&self, update: &AfStateUpdate, current_time: DateTime<Utc>) -> Result<(), String> {
        let age_observed = current_time.signed_duration_since(update.temporal.timestamp_observed);
        
        if age_observed.num_milliseconds() > update.temporal.validity_duration_ms {
            return Err(format!("Observation expired: age {}ms > validity {}ms",
                age_observed.num_milliseconds(), update.temporal.validity_duration_ms));
        }

        if update.temporal.timestamp_observed > update.temporal.timestamp_ingested {
            return Err("Out-of-order: observed timestamp is after ingestion timestamp".to_string());
        }

        Ok(())
    }

    pub fn process_update(&mut self, update: AfStateUpdate) -> Result<StateQuality, String> {
        let entity_id = EntityId(update.entity_id.clone());
        let attribute_id = AttributeId(update.variable_name.clone());
        let current_time = Utc::now();

        // Validate temporal constraints
        self.validate_temporal(&update, current_time)?;

        // Get policy
        let policy = match self.get_policy(&update.entity_id, &update.variable_name) {
            Some(p) => p,
            None => {
                // No policy = UNKNOWN uncertainty
                return Ok(StateQuality::Unknown);
            }
        };

        // Check policy validity
        if let Some(valid_until) = &policy.validity.valid_until {
            if current_time > *valid_until {
                return Ok(StateQuality::Invalid { reason: "Policy expired".to_string() });
            }
        }

        // Check if tolerance is known
        let tolerance = match policy.tolerance {
            Some(t) => t,
            None => {
                // Unknown tolerance = fail closed
                return Ok(StateQuality::Unknown);
            }
        };

        // Extract numerical value for comparison
        let value = match &update.value {
            crate::types::AfValue::Numerical { magnitude, .. } => *magnitude,
            _ => return Ok(StateQuality::Valid), // Non-numerical values don't need tolerance checking
        };

        let key = (entity_id.clone(), attribute_id.clone());

        // Check for contradiction
        if let Some((prev_value, _)) = self.current_state.get(&key) {
            if (value - prev_value).abs() > tolerance {
                // Contradiction detected
                let mut sources = vec![update.provenance.source_id.clone()];
                if let Some(existing_sources) = self.contradictions.get(&key) {
                    sources.extend(existing_sources.clone());
                }
                self.contradictions.insert(key.clone(), sources.clone());
                
                return Ok(StateQuality::Contradicted { sources });
            }
        }

        // Update current state
        self.current_state.insert(key.clone(), (value, current_time));
        self.contradictions.remove(&key);

        Ok(StateQuality::Valid)
    }
}

#[pyclass]
pub struct PyUncertaintyManager {
    inner: UncertaintyManager,
}

#[pymethods]
impl PyUncertaintyManager {
    #[new]
    pub fn new() -> Self {
        PyUncertaintyManager {
            inner: UncertaintyManager::new(),
        }
    }

    pub fn register_policy_json(&mut self, policy_json: String) -> PyResult<()> {
        let policy: UncertaintyPolicy = serde_json::from_str(&policy_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse policy: {}", e)))?;
        self.inner.register_policy(policy);
        Ok(())
    }

    pub fn process_update_json(&mut self, update_json: String) -> PyResult<String> {
        let update: AfStateUpdate = serde_json::from_str(&update_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse update: {}", e)))?;

        match self.inner.process_update(update) {
            Ok(quality) => {
                let result = serde_json::to_string(&quality)
                    .map_err(|e| PyValueError::new_err(format!("Failed to serialize: {}", e)))?;
                Ok(result)
            }
            Err(e) => Err(PyValueError::new_err(e)),
        }
    }
}
