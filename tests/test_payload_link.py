import json
import sys
from datetime import datetime, timezone

import af_core

def main():
    print("=== Payload & Link Capability Modeling Validation ===\n")

    modeler = af_core.PyPayloadLinkModeler()

    # 1. Register a capability specification (e.g., battery voltage limits)
    print("[1] Registering capability specification...")
    now = datetime.now(timezone.utc).isoformat()
    spec = {
        "attribute": "battery_voltage",
        "nominal_value": {"type": "Float", "value": {"value": 12.0, "unit": "V"}},
        "operating_limits": {"min": 10.0, "max": 14.0},
        "specification_source": "Manufacturer Datasheet",
        "specification_version": "1.0"
    }
    modeler.register_specification("drone_alpha_01", "battery_voltage", json.dumps(spec))
    print("    PASS: Specification registered\n")

    # 2. Ingest a valid observation
    print("[2] Ingesting valid observation (12.4V)...")
    valid_obs = {
        "attribute": "battery_voltage",
        "observed_value": {"type": "Float", "value": {"value": 12.4, "unit": "V"}},
        "uncertainty": {
            "representation_type": {"BoundedInterval": {"lower": 12.3, "upper": 12.5}},
            "confidence_level": 0.95,
            "basis": {"CalibrationCertificate": {"certificate_id": "CAL-001", "valid_until": "2027-01-01T00:00:00Z"}},
            "correlations": []
        },
        "timestamp": now,
        "source": {"source_id": "sensor_1", "timestamp_observed": now, "provenance_hash": "hash1"}
    }
    modeler.ingest_observation("drone_alpha_01", "battery_voltage", json.dumps(valid_obs))
    print("    PASS: Valid observation ingested\n")

    # 3. Ingest an out-of-limits observation
    print("[3] Ingesting out-of-limits observation (9.5V)...")
    invalid_obs = valid_obs.copy()
    invalid_obs["observed_value"]["value"]["value"] = 9.5
    invalid_obs["source"]["source_id"] = "sensor_2"
    modeler.ingest_observation("drone_alpha_01", "battery_voltage", json.dumps(invalid_obs))
    print("    PASS: Out-of-limits observation ingested\n")

    # 4. Evaluate operating limits
    print("[4] Evaluating operating limits...")
    results_json = modeler.evaluate_operating_limits("drone_alpha_01", "battery_voltage")
    results = json.loads(results_json)

    if len(results) == 2:
        print("    PASS: Both observations evaluated")
    else:
        print(f"    FAIL: Expected 2 results, got {len(results)}")
        sys.exit(1)

    if results[0]["is_within_limits"] is True:
        print("    PASS: Valid observation correctly marked as within limits")
    else:
        print("    FAIL: Valid observation incorrectly marked as out of limits")
        sys.exit(1)

    if results[1]["is_within_limits"] is False:
        print("    PASS: Out-of-limits observation correctly marked as violating limits")
    else:
        print("    FAIL: Out-of-limits observation incorrectly marked as within limits")
        sys.exit(1)

    # 5. Verify uncertainty representation is preserved
    if "uncertainty" in results[0] and results[0]["uncertainty"]["confidence_level"] == 0.95:
        print("    PASS: Uncertainty representation correctly preserved")
    else:
        print("    FAIL: Uncertainty representation missing or malformed")
        sys.exit(1)

    print("\n" + "="*60)
    print("=== PAYLOAD & LINK MODELING VALIDATION: PASSED ===")
    print("="*60)

if __name__ == "__main__":
    main()
