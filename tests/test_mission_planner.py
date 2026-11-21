import json
import sys
from datetime import datetime, timezone

import af_core

def main():
    print("=== Mission Planning & Multi-Objective Optimization Validation ===\n")

    planner = af_core.PyMissionPlanner()

    # 1. Define constraint results (one feasible, one infeasible, one feasible)
    print("[1] Defining constraint evaluation results...")
    constraint_results = [
        {
            "hard_gate_violations": [],
            "regulatory_gate_violations": [],
            "mission_requirement_violations": [],
            "soft_preference_violations": [],
            "missing_evidence": [],
            "unreliable_evidence": [],
            "is_executable": True,
            "blocking_reason": None
        },
        {
            "hard_gate_violations": [{"constraint_id": "c1", "outcome": {"Violated": {"actual_value": {"type": "Float", "value": {"value": 9.0, "unit": "V"}}, "required_value": {"type": "Float", "value": {"value": 10.0, "unit": "V"}}, "evidence": []}}, "enforcement": {"HardGate": {"justification": "Test", "authority": {"authority_type": "System", "authority_reference": "ref", "approval_provenance": "prov", "valid_from": datetime.now(timezone.utc).isoformat(), "valid_until": None}}}}],
            "regulatory_gate_violations": [],
            "mission_requirement_violations": [],
            "soft_preference_violations": [],
            "missing_evidence": [],
            "unreliable_evidence": [],
            "is_executable": False,
            "blocking_reason": {"HardConstraintViolated": {"count": 1}}
        },
        {
            "hard_gate_violations": [],
            "regulatory_gate_violations": [],
            "mission_requirement_violations": [],
            "soft_preference_violations": [],
            "missing_evidence": [],
            "unreliable_evidence": [],
            "is_executable": True,
            "blocking_reason": None
        }
    ]

    # 2. Define risk assessments (simplified)
    print("[2] Defining risk assessments...")
    risk_assessments = [
        {
            "mission_id": "mission_001",
            "assessment_timestamp": datetime.now(timezone.utc).isoformat(),
            "safety_risk": "Low",
            "operational_risk": "Low",
            "technical_risk": "Low",
            "regulatory_risk": "Negligible",
            "effectiveness_risk": "Low",
            "identified_hazards": [],
            "blocking_hazards": [],
            "mission_success_estimate": None,
            "assessment_confidence": {
                "confidence_level": 0.8,
                "confidence_definition": "Demo",
                "calibration_method": {"Uncalibrated": {"reason": "Demo"}},
                "confidence_reducers": []
            },
            "known_hazard_gaps": [],
            "model_limitations": []
        }
    ] * 3  # Repeat for all 3 alternatives

    # 3. Generate plan
    print("[3] Generating mission plan...")
    result_json = planner.generate_plan("mission_001", json.dumps(constraint_results), json.dumps(risk_assessments))
    result = json.loads(result_json)

    # 4. Verify outcomes
    print("[4] Verifying planning outcomes...")

    # Check that 2 alternatives are feasible (alt_1 and alt_3)
    if len(result["feasible_alternatives"]) == 2:
        print("    PASS: 2 feasible alternatives identified")
    else:
        print(f"    FAIL: Expected 2 feasible alternatives, got {len(result['feasible_alternatives'])}")
        sys.exit(1)

    # Check that 1 alternative is infeasible (alt_2)
    if len(result["infeasible_alternatives"]) == 1:
        print("    PASS: 1 infeasible alternative correctly filtered")
    else:
        print(f"    FAIL: Expected 1 infeasible alternative, got {len(result['infeasible_alternatives'])}")
        sys.exit(1)

    # Check Pareto front (should contain alt_1 and alt_3, as alt_2 is infeasible)
    # alt_1: fast (30 min), expensive (500 Wh, $150)
    # alt_3: slow (60 min), efficient (250 Wh, $80)
    # Neither dominates the other, so both should be in Pareto front
    if len(result["pareto_front"]) == 2:
        print("    PASS: Pareto front contains 2 non-dominated alternatives")
    else:
        print(f"    FAIL: Expected 2 Pareto-optimal alternatives, got {len(result['pareto_front'])}")
        sys.exit(1)

    # Verify no composite score is used
    for alt in result["feasible_alternatives"]:
        obj_vals = alt["objective_values"]
        if "MissionDuration" in obj_vals and "EnergyConsumption" in obj_vals and "OperationalCost" in obj_vals:
            # Check that each objective has its own unit
            if obj_vals["MissionDuration"]["unit"] == "minutes" and \
               obj_vals["EnergyConsumption"]["unit"] == "Wh" and \
               obj_vals["OperationalCost"]["unit"] == "USD":
                print(f"    PASS: Alternative {alt['alternative_id']} has objectives in native units (no composite score)")
            else:
                print(f"    FAIL: Alternative {alt['alternative_id']} has incorrect units")
                sys.exit(1)
        else:
            print(f"    FAIL: Alternative {alt['alternative_id']} missing objectives")
            sys.exit(1)

    # Verify planning metadata
    if result["planning_metadata"]["candidates_generated"] == 3 and \
       result["planning_metadata"]["candidates_filtered_by_hard_constraints"] == 1 and \
       result["planning_metadata"]["pareto_optimal_count"] == 2:
        print("    PASS: Planning metadata correctly recorded")
    else:
        print(f"    FAIL: Planning metadata incorrect: {result['planning_metadata']}")
        sys.exit(1)

    print("\n" + "="*60)
    print("=== MISSION PLANNING VALIDATION: PASSED ===")
    print("="*60)

if __name__ == "__main__":
    main()
