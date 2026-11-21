use pyo3::prelude::*;

pub mod core_types;
pub mod schema;
pub mod reality;
pub mod uncertainty;
pub mod types;
pub mod mission_model;
pub mod feasibility;
pub mod mission_contracts;
pub mod advanced_constraints;
pub mod payload_link;
pub mod mission_risk;
pub mod mission_planner;

#[pymodule]
fn af_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<schema::PySchemaRegistry>()?;
    m.add_class::<uncertainty::PyUncertaintyManager>()?;
    m.add_class::<reality::PyRealityModel>()?;
    m.add_class::<feasibility::PyFeasibilityAnalyzer>()?;
    m.add_class::<advanced_constraints::PyAdvancedConstraintEvaluator>()?;
    m.add_class::<payload_link::PyPayloadLinkModeler>()?;
    m.add_class::<mission_risk::PyRiskAssessor>()?;
    m.add_class::<mission_planner::PyMissionPlanner>()?;
    Ok(())
}
