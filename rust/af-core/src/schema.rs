use crate::core_types::{EntitySchema, EntityType, AttributeId, AttributeDefinition};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
pub struct PySchemaRegistry {
    schemas: HashMap<EntityType, EntitySchema>,
}

#[pymethods]
impl PySchemaRegistry {
    #[new]
    pub fn new() -> Self {
        PySchemaRegistry {
            schemas: HashMap::new(),
        }
    }

    pub fn load_schema_from_file(&mut self, path: String) -> PyResult<()> {
        let content = fs::read_to_string(&path)
            .map_err(|e| PyValueError::new_err(format!("Failed to read schema file: {}", e)))?;
        
        let schema: EntitySchema = serde_json::from_str(&content)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse schema: {}", e)))?;
        
        self.schemas.insert(schema.entity_type.clone(), schema);
        Ok(())
    }

    pub fn load_schema_from_json(&mut self, json: String) -> PyResult<()> {
        let schema: EntitySchema = serde_json::from_str(&json)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse schema: {}", e)))?;
        
        self.schemas.insert(schema.entity_type.clone(), schema);
        Ok(())
    }

    pub fn get_schema(&self, entity_type: String) -> PyResult<Option<String>> {
        match self.schemas.get(&EntityType(entity_type)) {
            Some(schema) => {
                let json = serde_json::to_string(schema)
                    .map_err(|e| PyValueError::new_err(format!("Failed to serialize: {}", e)))?;
                Ok(Some(json))
            }
            None => Ok(None),
        }
    }

    pub fn validate_attribute(
        &self,
        entity_type: String,
        attribute_id: String,
        value_type: String,
        unit: Option<String>,
    ) -> PyResult<bool> {
        let schema = match self.schemas.get(&EntityType(entity_type)) {
            Some(s) => s,
            None => return Ok(false),
        };

        let attr_def = match schema.attributes.get(&AttributeId(attribute_id)) {
            Some(a) => a,
            None => return Ok(false),
        };

        if attr_def.value_type != value_type {
            return Ok(false);
        }

        if let Some(expected_unit) = &attr_def.unit {
            if let Some(actual_unit) = unit {
                if expected_unit != &actual_unit {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn list_entity_types(&self) -> PyResult<Vec<String>> {
        Ok(self.schemas.keys().map(|k| k.0.clone()).collect())
    }
}
