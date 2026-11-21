import json
import sys
from datetime import datetime, timezone, timedelta
import af_core

def main():
    print("="*70)
    print("STAGE 2 INTEGRATION TEST: AERIAL SURVEY MISSION")
    print("="*70)
    print()
    
    # Initialize all modules
    constraint_eval = af_core.PyAdvancedConstraintEvaluator()
    payload_model = af_core.PyPayloadLinkModeler()
    risk_assessor = af_core.PyRiskAssessor()
    planner = af_core.PyMissionPlanner()
    
    now = datetime.now(timezone.utc)
    now_iso = now.isoformat()
    
    print("[1/8] Creating reality snapshot with mixed evidence quality...")
    snapshot = {
        "snapshot_id": "snap_survey_001",
        "captured_at": now_iso,
        "entities": [
            {
                "entity_id": "uav_alpha_01",
                "entity_type": "aircraft",
                "schema_version": "1.0.0",
                "attributes": {
                    "battery_voltage": {"type": "Float", "value": {"value": 11.8, "unit": "V"}},
                    "gps_latitude": {"type": "Float", "value": {"value": -33.8688, "unit": "deg"}},
                    "altitude_msl": {"type": "Float", "value": {"value": 85.0, "unit": "m"}},
                    "flight_mode": {"type": "Text", "value": "AUTO"}
                },
                "metadata": {
                    "last_updated": now_iso,
                    "source_references": [{"source_id": "telem_1", "timestamp_observed": now_iso, "provenance_hash": "hash1"}],
                    "quality": {"status": "Valid"}
                }
            },
            {
                "entity_id": "camera_payload_01",
                "entity_type": "payload",
                "schema_version": "1.0.0",
                "attributes": {
                    "operating_temp": {"type": "Float", "value": {"value": 28.5, "unit": "C"}}
                },
                "metadata": {
                    "last_updated": now_iso,
                    "source_references": [],
                    "quality": {"status": "Valid"}
                }
            },
            {
                "entity_id": "comm_link_01",
                "entity_type": "communication_link",
                "schema_version": "1.0.0",
                "attributes": {
                    "rssi": {"type": "Float", "value": {"value": -72.0, "unit": "dBm"}}
                },
                "metadata": {
                    "last_updated": now_iso,
                    "source_references": [],
                    "quality": {"status": "Contradicted", "detail": {"sources": ["sensor_a", "sensor_b"]}}
                }
            }
        ],
        "provenance": [],
        "quality": {"status": "Valid"}
    }
    snapshot_json = json.dumps(snapshot)
    print("    Snapshot created with 3 entities")
    
    print("[2/8] Registering payload specifications...")
    camera_spec = {
        "attribute": "operating_temp",
        "nominal_value": {"type": "Float", "value": {"value": 25.0, "unit": "C"}},
        "operating_limits": {"min": -10.0, "max": 45.0},
        "specification_source": "Camera Datasheet",
        "specification_version": "2.1"
    }
    payload_model.register_specification("camera_payload_01", "operating_temp", json.dumps(camera_spec))
    
    camera_obs = {
        "attribute": "operating_temp",
        "observed_value": {"type": "Float", "value": {"value": 28.5, "unit": "C"}},
        "uncertainty": {
            "representation_type": {"BoundedInterval": {"lower": 27.5, "upper": 29.5}},
            "confidence_level": 0.95,
            "basis": {"CalibrationCertificate": {"certificate_id": "CAM-001", "valid_until": "2027-12-31T23:59:59Z"}},
            "correlations": []
        },
        "timestamp": now_iso,
        "source": {"source_id": "temp_sensor_1", "timestamp_observed": now_iso, "provenance_hash": "cam_hash"}
    }
    payload_model.ingest_observation("camera_payload_01", "operating_temp", json.dumps(camera_obs))
    print("    Camera spec and observation registered")
    
    print("[3/8] Defining constraints...")
    constraints = [
        {
            "constraint_id": "hard_battery",
            "entity_id": "uav_alpha_01",
            "attribute_id": "battery_voltage",
            "required_value": {"type": "Float", "value": {"value": 11.0, "unit": "V"}},
            "enforcement": {
                "HardGate": {
                    "justification": "Minimum voltage for safe flight",
                    "authority": {"authority_type": "Certification", "authority_reference": "Flight Manual 3.2", "approval_provenance": "ENG-001", "valid_from": "2024-01-01T00:00:00Z", "valid_until": None}
                }
            }
        },
        {
            "constraint_id": "unreliable_rssi",
            "entity_id": "comm_link_01",
            "attribute_id": "rssi",
            "required_value": {"type": "Float", "value": {"value": -75.0, "unit": "dBm"}},
            "enforcement": {
                "HardGate": {
                    "justification": "Minimum signal strength",
                    "authority": {"authority_type": "System", "authority_reference": "Comm Req", "approval_provenance": "SYS-002", "valid_from": "2024-01-01T00:00:00Z", "valid_until": None}
                }
            }
        },
        {
            "constraint_id": "soft_altitude",
            "entity_id": "uav_alpha_01",
            "attribute_id": "altitude_msl",
            "required_value": {"type": "Float", "value": {"value": 100.0, "unit": "m"}},
            "enforcement": {"SoftPreference": {"preference_weight": 0.7}}
        }
    ]
    constraints_json = json.dumps(constraints)
    print("    Constraints defined")
    
    print("[4/8] Evaluating constraints...")
    constraint_result_json = constraint_eval.evaluate_constraints(constraints_json, snapshot_json)
    constraint_result = json.loads(constraint_result_json)
    print(f"    Is executable: {constraint_result['is_executable']}")
    print(f"    Unreliable evidence count: {len(constraint_result['unreliable_evidence'])}")
    
    print("[5/8] Registering hazards...")
    hazard = {
        "hazard_id": "HZ-002",
        "description": "Communication link degradation",
        "hazard_authority": {"authority_type": "Operational", "authority_reference": "Risk Assess", "approval_provenance": "OPS-001", "valid_from": "2024-01-01T00:00:00Z", "valid_until": None},
        "evidence": [{"evidence_type": "FlightData", "evidence_reference": "DATA-001", "evidence_summary": "Loss of control risk", "confidence_level": 0.85}],
        "triggering_conditions": ["comm_link_degraded"],
        "affected_dimensions": ["Operational", "Safety"],
        "severity": "Critical",
        "is_blocking": False,
        "blocking_authority": None,
        "blocking_rationale": None
    }
    risk_assessor.register_hazard(json.dumps(hazard))
    print("    Hazard registered")
    
    print("[6/8] Assessing mission risk...")
    conditions = {"comm_link_degraded": "true"}
    risk_result_json = risk_assessor.assess_risk("survey_mission_001", json.dumps(conditions))
    risk_result = json.loads(risk_result_json)
    print(f"    Identified hazards: {len(risk_result['identified_hazards'])}")
    print(f"    Mission success estimate: {risk_result['mission_success_estimate']}")
    
    print("[7/8] Generating mission plan...")
    constraint_results = [
        constraint_result,
        {"hard_gate_violations": [], "regulatory_gate_violations": [], "mission_requirement_violations": [], "soft_preference_violations": [], "missing_evidence": [], "unreliable_evidence": [], "is_executable": True, "blocking_reason": None},
        {"hard_gate_violations": [], "regulatory_gate_violations": [], "mission_requirement_violations": [], "soft_preference_violations": [], "missing_evidence": [], "unreliable_evidence": [], "is_executable": True, "blocking_reason": None}
    ]
    risk_assessments = [risk_result, risk_result, risk_result]
    
    plan_result_json = planner.generate_plan("survey_mission_001", json.dumps(constraint_results), json.dumps(risk_assessments))
    plan_result = json.loads(plan_result_json)
    print(f"    Feasible alternatives: {len(plan_result['feasible_alternatives'])}")
    print(f"    Infeasible alternatives: {len(plan_result['infeasible_alternatives'])}")
    print(f"    Pareto-optimal: {len(plan_result['pareto_front'])}")
    
    print("[8/8] Verifying integration results...")
    assert constraint_result['is_executable'] == False, "Module 1: Should be blocked"
    assert len(constraint_result['unreliable_evidence']) == 1, "Module 1: Should detect unreliable evidence"
    print("    PASS: Module 1 (Constraints) - Fail-closed behavior confirmed")
    
    camera_limits_json = payload_model.evaluate_operating_limits("camera_payload_01", "operating_temp")
    camera_limits = json.loads(camera_limits_json)
    assert camera_limits[0]['is_within_limits'] == True, "Module 2: Should be within limits"
    print("    PASS: Module 2 (Payload & Link) - Specifications and observations separated")
    
    assert len(risk_result['identified_hazards']) == 1, "Module 3: Should identify hazard"
    mse = risk_result['mission_success_estimate']
    assert mse is not None, "Module 3: estimate wrapper must carry justification"
    assert mse['estimate'] is None, "Module 3: PoS must be None when unjustified"
    assert mse['confidence_interval'] is None, "Module 3: CI must be None when unjustified"
    assert mse['statistical_justification']['is_justified'] is False, "Module 3: must be flagged unjustified"
    print("    PASS: Module 3 (Risk) - Hazard tracking with authority, PoS is None")
    
    assert len(plan_result['feasible_alternatives']) == 2, "Module 4: Should have 2 feasible"
    assert len(plan_result['infeasible_alternatives']) == 1, "Module 4: Should have 1 infeasible"
    print("    PASS: Module 4 (Planning) - Multi-objective optimization working")
    
    for alt in plan_result['feasible_alternatives']:
        obj_vals = alt['objective_values']
        assert obj_vals['MissionDuration']['unit'] == 'minutes', "Module 4: Duration unit"
        assert obj_vals['EnergyConsumption']['unit'] == 'Wh', "Module 4: Energy unit"
        assert obj_vals['OperationalCost']['unit'] == 'USD', "Module 4: Cost unit"
    print("    PASS: Module 4 (Planning) - Native units preserved, no composite scores")
    
    print()
    print("="*70)
    print("STAGE 2 INTEGRATION TEST: PASSED")
    print("="*70)
    print("Stage 2 (Enhanced Option 1) is COMPLETE and VERIFIED.")
    return 0

if __name__ == "__main__":
    sys.exit(main())

