import json
import sys
from datetime import datetime, timezone

import af_core

def main():
    print("=== Stage 2: Mission Feasibility Validation ===\n")

    analyzer = af_core.PyFeasibilityAnalyzer()

    # 1. Define a feasible mission (requires battery > 10.0V)
    print("[1] Testing FEASIBLE mission...")
    feasible_mission = {
        "mission_id": "mission_001",
        "objective": {
            "description": "Survey area",
            "success_criteria": {}
        },
        "tasks": [
            {
                "task_id": "task_1",
                "description": "Fly to survey area",
                "dependencies": [],
                "constraints": [
                    {
                        "attribute": "battery_voltage",
                        "operator": "GreaterThan",
                        "value": {"type": "Float", "value": {"value": 10.0, "unit": "V"}}
                    }
                ]
            }
        ],
        "assigned_entity": "drone_alpha_01",
        "deadline": None
    }

    # 2. Define an infeasible mission (requires battery > 15.0V)
    print("[2] Testing INFEASIBLE mission...")
    infeasible_mission = {
        "mission_id": "mission_002",
        "objective": {
            "description": "Heavy payload delivery",
            "success_criteria": {}
        },
        "tasks": [
            {
                "task_id": "task_1",
                "description": "Lift heavy payload",
                "dependencies": [],
                "constraints": [
                    {
                        "attribute": "battery_voltage",
                        "operator": "GreaterThan",
                        "value": {"type": "Float", "value": {"value": 15.0, "unit": "V"}}
                    }
                ]
            }
        ],
        "assigned_entity": "drone_alpha_01",
        "deadline": None
    }

    # 3. Define the Reality Snapshot (drone has 12.4V battery)
    print("[3] Defining Reality Snapshot (drone has 12.4V)...")
    now = datetime.now(timezone.utc).isoformat()
    snapshot = {
        "snapshot_id": "snap_test",
        "captured_at": now,
        "entities": [
            {
                "entity_id": "drone_alpha_01",
                "entity_type": "aircraft",
                "schema_version": "1.0.0",
                "attributes": {
                    "battery_voltage": {"type": "Float", "value": {"value": 12.4, "unit": "V"}},
                    "flight_mode": {"type": "Text", "value": "STABILIZE"}
                },
                "metadata": {
                    "last_updated": now,
                    "source_references": [],
                    "quality": {"status": "Valid"}
                }
            }
        ],
        "provenance": [],
        "quality": {"status": "Valid"}
    }

    snapshot_json = json.dumps(snapshot)

    # 4. Evaluate Feasible Mission
    print("[4] Evaluating feasible mission...")
    result_feasible = json.loads(analyzer.evaluate_feasibility(json.dumps(feasible_mission), snapshot_json))
    if result_feasible["is_feasible"] and len(result_feasible["violations"]) == 0:
        print("    PASS: Feasible mission correctly approved")
    else:
        print(f"    FAIL: Feasible mission rejected: {result_feasible}")
        sys.exit(1)

    # 5. Evaluate Infeasible Mission
    print("[5] Evaluating infeasible mission...")
    result_infeasible = json.loads(analyzer.evaluate_feasibility(json.dumps(infeasible_mission), snapshot_json))
    if not result_infeasible["is_feasible"] and len(result_infeasible["violations"]) > 0:
        print(f"    PASS: Infeasible mission correctly rejected")
        print(f"    Violation details: {result_infeasible['violations'][0]['reason']}")
    else:
        print(f"    FAIL: Infeasible mission incorrectly approved: {result_infeasible}")
        sys.exit(1)

    # 6. Test Missing Entity
    print("[6] Testing missing entity in snapshot...")
    missing_entity_mission = feasible_mission.copy()
    missing_entity_mission["assigned_entity"] = "nonexistent_drone"
    
    result_missing = json.loads(analyzer.evaluate_feasibility(json.dumps(missing_entity_mission), snapshot_json))
    if not result_missing["is_feasible"] and len(result_missing["warnings"]) > 0:
        print(f"    PASS: Missing entity correctly flagged: {result_missing['warnings'][0]}")
    else:
        print(f"    FAIL: Missing entity not handled correctly: {result_missing}")
        sys.exit(1)

    print("\n" + "="*60)
    print("=== STAGE 2 FEASIBILITY VALIDATION: PASSED ===")
    print("="*60)
    print("\nVerified capabilities:")
    print("  ✓ Mission model parsing and constraint definition")
    print("  ✓ Feasible missions correctly approved")
    print("  ✓ Infeasible missions correctly rejected with specific violations")
    print("  ✓ Missing entity handling with appropriate warnings")

if __name__ == "__main__":
    main()
