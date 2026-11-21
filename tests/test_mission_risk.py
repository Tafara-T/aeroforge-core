import json
import sys
from datetime import datetime, timezone

import af_core

def main():
    print("=== Mission Risk Assessment Validation ===\n")

    assessor = af_core.PyRiskAssessor()
    now = datetime.now(timezone.utc).isoformat()

    # 1. Register a blocking hazard with explicit authority
    print("[1] Registering blocking hazard (Low Battery)...")
    blocking_hazard = {
        "hazard_id": "HZ-001",
        "description": "Battery voltage below safe operating limit",
        "hazard_authority": {
            "authority_type": "Design",
            "authority_reference": "Aircraft Flight Manual Sec 3.2",
            "approval_provenance": "ENG-APPROVAL-2023-001",
            "valid_from": "2023-01-01T00:00:00Z",
            "valid_until": None
        },
        "evidence": [{
            "evidence_type": "Analysis",
            "evidence_reference": "DOC-123",
            "evidence_summary": "Thermal runaway risk below 10V",
            "confidence_level": 0.95
        }],
        "triggering_conditions": ["low_battery_detected"],
        "affected_dimensions": ["Safety", "Operational"],
        "severity": "Critical",
        "is_blocking": True,
        "blocking_authority": {
            "authority_type": "Regulation",
            "authority_reference": "FAA Part 107",
            "approval_provenance": "REG-DOC-456",
            "approval_date": "2023-01-01T00:00:00Z",
            "rationale": "Regulatory mandate to prevent uncontrolled descent"
        },
        "blocking_rationale": "Safety-critical limit exceeded; mission must abort"
    }
    assessor.register_hazard(json.dumps(blocking_hazard))
    print("    PASS: Blocking hazard registered\n")

    # 2. Register a non-blocking hazard
    print("[2] Registering non-blocking hazard (High Wind)...")
    non_blocking_hazard = blocking_hazard.copy()
    non_blocking_hazard["hazard_id"] = "HZ-002"
    non_blocking_hazard["description"] = "Wind speed exceeds optimal payload stability"
    non_blocking_hazard["triggering_conditions"] = ["high_wind_detected"]
    non_blocking_hazard["is_blocking"] = False
    non_blocking_hazard["blocking_authority"] = None
    non_blocking_hazard["blocking_rationale"] = None
    assessor.register_hazard(json.dumps(non_blocking_hazard))
    print("    PASS: Non-blocking hazard registered\n")

    # 3. Assess risk with triggering conditions
    print("[3] Assessing risk with active hazards...")
    conditions = {
        "low_battery_detected": "true",
        "high_wind_detected": "true"
    }
    
    result_json = assessor.assess_risk("mission_001", json.dumps(conditions))
    result = json.loads(result_json)

    # 4. Verify Outcomes
    print("[4] Verifying Risk Assessment Outcomes...")
    
    if len(result["identified_hazards"]) == 2:
        print("    PASS: Both hazards correctly identified")
    else:
        print(f"    FAIL: Expected 2 identified hazards, got {len(result['identified_hazards'])}")
        sys.exit(1)

    if len(result["blocking_hazards"]) == 1 and result["blocking_hazards"][0]["hazard_id"] == "HZ-001":
        print("    PASS: Only the blocking hazard is in the blocking list")
    else:
        print(f"    FAIL: Blocking hazards incorrect: {result['blocking_hazards']}")
        sys.exit(1)

    # Verify blocking hazard has required authority fields
    hz1 = result["blocking_hazards"][0]
    if hz1["blocking_authority"] is not None and hz1["blocking_rationale"] is not None:
        print("    PASS: Blocking hazard contains required authority and rationale")
    else:
        print("    FAIL: Blocking hazard missing authority or rationale")
        sys.exit(1)

    # Verify mission success estimate is statistically unjustified (None)
    mse = result["mission_success_estimate"]
    if mse["estimate"] is None and mse["statistical_justification"]["is_justified"] is False:
        print("    PASS: Mission success estimate correctly set to None (unjustified)")
    else:
        print(f"    FAIL: Mission success estimate should be None: {mse}")
        sys.exit(1)

    # Verify confidence assessment is present and defined
    if result["assessment_confidence"]["confidence_definition"] != "":
        print("    PASS: Confidence assessment explicitly defined")
    else:
        print("    FAIL: Confidence assessment missing definition")
        sys.exit(1)

    print("\n" + "="*60)
    print("=== MISSION RISK ASSESSMENT VALIDATION: PASSED ===")
    print("="*60)

if __name__ == "__main__":
    main()
