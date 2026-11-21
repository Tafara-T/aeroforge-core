use crate::core_types::{
    EntityId, EntityType, AttributeId, StateValue, EntityState, StateMetadata,
    StateQuality, SourceReference, RealitySnapshot, SnapshotId,
};
use crate::types::AfStateUpdate;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[derive(Debug)]
pub struct RealityModel {
    entities: HashMap<EntityId, EntityState>,
    history: Vec<(DateTime<Utc>, EntityId, AttributeId, StateValue)>,
}

impl RealityModel {
    pub fn new() -> Self {
        RealityModel {
            entities: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn ingest_update(&mut self, update: AfStateUpdate, quality: StateQuality) {
        let entity_id = EntityId(update.entity_id.clone());
        let attribute_id = AttributeId(update.variable_name.clone());
        let timestamp = update.temporal.timestamp_observed;

        // Convert AfValue to StateValue
        let state_value = match update.value {
            crate::types::AfValue::Numerical { magnitude, unit } => {
                StateValue::Float { value: magnitude, unit }
            }
            crate::types::AfValue::Categorical(s) => StateValue::Text(s),
            crate::types::AfValue::Boolean(b) => StateValue::Boolean(b),
            crate::types::AfValue::Structured(_) => StateValue::Null, // TODO: handle structured
            crate::types::AfValue::Null => StateValue::Null,
        };

        // Record in history
        self.history.push((timestamp, entity_id.clone(), attribute_id.clone(), state_value.clone()));

        // Get or create entity
        let entity = self.entities.entry(entity_id.clone()).or_insert_with(|| {
            EntityState {
                entity_id: entity_id.clone(),
                entity_type: EntityType("unknown".to_string()),
                schema_version: "1.0.0".to_string(),
                attributes: HashMap::new(),
                metadata: StateMetadata {
                    last_updated: timestamp,
                    source_references: Vec::new(),
                    quality: StateQuality::Valid,
                },
            }
        });

        // Update attribute
        entity.attributes.insert(attribute_id, state_value);
        entity.metadata.last_updated = timestamp;
        entity.metadata.quality = quality;

        // Add source reference
        entity.metadata.source_references.push(SourceReference {
            source_id: update.provenance.source_id,
            timestamp_observed: update.temporal.timestamp_observed,
            provenance_hash: update.provenance.provenance_hash,
        });
    }

    pub fn set_entity_type(&mut self, entity_id: String, entity_type: String) {
        if let Some(entity) = self.entities.get_mut(&EntityId(entity_id)) {
            entity.entity_type = EntityType(entity_type);
        }
    }

    pub fn get_entity(&self, entity_id: &str) -> Option<&EntityState> {
        self.entities.get(&EntityId(entity_id.to_string()))
    }

    pub fn get_snapshot(&self) -> RealitySnapshot {
        let entities: Vec<EntityState> = self.entities.values().cloned().collect();
        
        let overall_quality = if entities.iter().all(|e| e.metadata.quality == StateQuality::Valid) {
            StateQuality::Valid
        } else if entities.iter().any(|e| matches!(e.metadata.quality, StateQuality::Contradicted { .. })) {
            StateQuality::Contradicted { sources: vec![] }
        } else {
            StateQuality::Unknown
        };

        let provenance: Vec<SourceReference> = entities.iter()
            .flat_map(|e| e.metadata.source_references.clone())
            .collect();

        RealitySnapshot {
            snapshot_id: SnapshotId(format!("snap_{}", Utc::now().timestamp())),
            captured_at: Utc::now(),
            entities,
            provenance,
            quality: overall_quality,
        }
    }

    pub fn get_history(&self, entity_id: &str, attribute_id: &str) -> Vec<&(DateTime<Utc>, EntityId, AttributeId, StateValue)> {
        let eid = EntityId(entity_id.to_string());
        let aid = AttributeId(attribute_id.to_string());
        self.history.iter()
            .filter(|(_, e, a, _)| e == &eid && a == &aid)
            .collect()
    }
}

#[pyclass]
pub struct PyRealityModel {
    inner: RealityModel,
}

#[pymethods]
impl PyRealityModel {
    #[new]
    pub fn new() -> Self {
        PyRealityModel {
            inner: RealityModel::new(),
        }
    }

    pub fn ingest_update_json(&mut self, update_json: String, quality_json: String) -> PyResult<()> {
        let update: AfStateUpdate = serde_json::from_str(&update_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse update: {}", e)))?;
        let quality: StateQuality = serde_json::from_str(&quality_json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse quality: {}", e)))?;
        self.inner.ingest_update(update, quality);
        Ok(())
    }

    pub fn set_entity_type(&mut self, entity_id: String, entity_type: String) {
        self.inner.set_entity_type(entity_id, entity_type);
    }

    pub fn get_entity_json(&self, entity_id: String) -> PyResult<Option<String>> {
        match self.inner.get_entity(&entity_id) {
            Some(entity) => {
                let json = serde_json::to_string(entity)
                    .map_err(|e| PyValueError::new_err(format!("Failed to serialize: {}", e)))?;
                Ok(Some(json))
            }
            None => Ok(None),
        }
    }

    pub fn get_snapshot_json(&self) -> PyResult<String> {
        let snapshot = self.inner.get_snapshot();
        let json = serde_json::to_string(&snapshot)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize: {}", e)))?;
        Ok(json)
    }

    pub fn get_history_json(&self, entity_id: String, attribute_id: String) -> PyResult<String> {
        let history = self.inner.get_history(&entity_id, &attribute_id);
        let json = serde_json::to_string(&history)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize: {}", e)))?;
        Ok(json)
    }
}
