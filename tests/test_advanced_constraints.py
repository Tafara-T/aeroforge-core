import json
import sys
from datetime import datetime, timezone

import af_core

def main():
    print("=== Advanced Mission Constraints Validation ===\n")

    evaluator = af_core.PyAdvancedConstraintEvaluator()

    # 1. Define a Reality Snapshot with mixed quality states
    print("[1] Defining Reality Snapshot...")
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
                    # Note: 'gps_latitude' is intentionally MISSING to test MissingEvidence
                },
                "metadata": {
                    "last_updated": now,
                    "source_references": [{"source_id": "telem_1", "timestamp_observed": now, "provenance_hash": "hash1"}],
                    "quality": {"status": "Valid"}
                }
            },
            {
                "entity_id": "drone_beta_02",
                "entity_type": "aircraft",
                "schema_version": "1.0.0",
                "attributes": {
                    "battery_voltage": {"type": "Float", "value": {"value": 11.0, "unit": "V"}}
                },
                "metadata": {
                    "last_updated": now,
                    "source_references": [],
                    "quality": {"status": "Contradicted", "detail": {"sources": ["sensor_a", "sensor_b"]}}
                }
            }
        ],
        "provenance": [],
        "quality": {"status": "Valid"}
    }
    snapshot_json = json.dumps(snapshot)
    print("    PASS: Snapshot defined\n")

    # 2. Define Constraints testing all four outcomes
    print("[2] Defining Constraints...")
    constraints = [
        # Test 1: Satisfied
        {
            "constraint_id": "c1",
            "entity_id": "drone_alpha_01",
            "attribute_id": "battery_voltage",
            "required_value": {"type": "Float", "value": {"value": 12.4, "unit": "V"}},
            "enforcement": {"HardGate": {"justification": "Test", "authority": {"authority_type": "System", "authority_reference": "ref", "approval_provenance": "prov", "valid_from": now, "valid_until": None}}}
        },
        # Test 2: Missing Evidence (attribute missing)
        {
            "constraint_id": "c2",
            "entity_id": "drone_alpha_01",
            "attribute_id": "gps_latitude",
            "required_value": {"type": "Float", "value": {"value": -33.8, "unit": "deg"}},
            "enforcement": {"HardGate": {"justification": "Test", "authority": {"authority_type": "System", "authority_reference": "ref", "approval_provenance": "prov", "valid_from": now, "valid_until": None}}}
        },
        # Test 3: Unreliable Evidence (Contradicted state)
        {
            "constraint_id": "c3",
            "entity_id": "drone_beta_02",
            "attribute_id": "battery_voltage",
            "required_value": {"type": "Float", "value": {"value": 11.0, "unit": "V"}},
            "enforcement": {"HardGate": {"justification": "Test", "authority": {"authority_type": "System", "authority_reference": "ref", "approval_provenance": "prov", "valid_from": now, "valid_until": None}}}
        },
        # Test 4: Violated (Soft preference, so it shouldn't block)
        {
            "constraint_id": "c4",
            "entity_id": "drone_alpha_01",
            "attribute_id": "flight_mode",
            "required_value": {"type": "Text", "value": "AUTO"},
            "enforcement": {"SoftPreference": {"preference_weight": 0.5}}
        }
    ]
    constraints_json = json.dumps(constraints)
    print("    PASS: Constraints defined\n")

    # 3. Evaluate
    print("[3] Evaluating Constraints...")
    result_json = evaluator.evaluate_constraints(constraints_json, snapshot_json)
    result = json.loads(result_json)

    # 4. Verify Outcomes
    print("[4] Verifying Outcomes...")
    
    if result["is_executable"]:
        print("    FAIL: Mission should NOT be executable due to missing/unreliable evidence")
        sys.exit(1)
    else:
        print("    PASS: Mission correctly marked as non-executable")

    if len(result["missing_evidence"]) == 1 and result["missing_evidence"][0]["constraint_id"] == "c2":
        print("    PASS: Missing evidence correctly identified (c2)")
    else:
        print(f"    FAIL: Missing evidence not correctly identified: {result['missing_evidence']}")
        sys.exit(1)

    if len(result["unreliable_evidence"]) == 1 and result["unreliable_evidence"][0]["constraint_id"] == "c3":
        print("    PASS: Unreliable evidence correctly identified (c3)")
    else:
        print(f"    FAIL: Unreliable evidence not correctly identified: {result['unreliable_evidence']}")
        sys.exit(1)

    if len(result["soft_preference_violations"]) == 1 and result["soft_preference_violations"][0]["constraint_id"] == "c4":
        print("    PASS: Soft preference violation correctly identified (c4)")
    else:
        print(f"    FAIL: Soft preference violation not correctly identified: {result['soft_preference_violations']}")
        sys.exit(1)

    if result["blocking_reason"] is not None:
        print(f"    PASS: Blocking reason provided: {result['blocking_reason']}")
    else:
        print("    FAIL: Blocking reason missing")
        sys.exit(1)

    print("\n" + "="*60)
    print("=== ADVANCED CONSTRAINTS VALIDATION: PASSED ===")
    print("="*60)

if __name__ == "__main__":
    main()
