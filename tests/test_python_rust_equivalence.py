import json
import sys
import math

# Import the reference models from the existing Stage 3 test file
# (Ensure this script is run from the ~/AF/aeroforge-core/tests directory)
from test_stage3_predictions import (
    predict_fuel_endurance as py_predict_fuel_endurance,
    predict_link_reliability as py_predict_link_reliability,
    predict_mission_duration as py_predict_mission_duration
)

# Import the Rust bindings
import af_core

def assert_float_equivalence(py_val, rust_val, tolerance=1e-6):
    """Asserts floating point equivalence within combined absolute/relative tolerance."""
    if py_val is None and rust_val is None:
        return True
    if py_val is None or rust_val is None:
        return False
    
    # Combined absolute and relative tolerance: |a - b| <= tol + tol * |b|
    allowed_diff = tolerance + tolerance * abs(py_val)
    return abs(py_val - rust_val) <= allowed_diff

def test_fuel_endurance_equivalence():
    print("[Test 1] Fuel Endurance Equivalence (Nominal)")
    input_obs = [{"source_id": "jsbsim_sim", "timestamp_observed": "2026-11-17T12:00:00Z", "provenance_hash": "test_hash"}]
    
    # Python reference uses "magnitude"
    py_res = py_predict_fuel_endurance([100.0], [0.1], 20.0, input_obs)
    py_val = py_res["predicted_value"]["value"]["magnitude"]
    
    # Rust uses "value"
    tanks = [{"tank_id": "t1", "contents_lbs": 100.0}]
    engines = [{"engine_id": "e1", "flow_pps": 0.1}]
    
    engine = af_core.PyPredictionEngine()
    rust_json = engine.predict_fuel_endurance_json(
        json.dumps(tanks), json.dumps(engines), 20.0, json.dumps(input_obs)
    )
    rust_res = json.loads(rust_json)
    rust_val = rust_res["predicted_value"]["value"]["value"]
    
    assert assert_float_equivalence(py_val, rust_val), f"Fuel mismatch: py={py_val}, rust={rust_val}"
    print("  PASS: Fuel endurance matches within tolerance.")

def test_link_reliability_equivalence():
    print("[Test 2] Link Reliability Equivalence (Nominal)")
    input_obs = [{"source_id": "jsbsim_sim", "timestamp_observed": "2026-11-17T12:00:00Z", "provenance_hash": "test_hash"}]
    
    # Python reference uses "magnitude"
    py_res = py_predict_link_reliability(45.0, -122.0, 1000.0, 45.0, -122.0, 500.0, 900.0, 20.0, 3.0, 3.0, -90.0, 10.0, input_obs)
    py_val = py_res["predicted_value"]["value"]["magnitude"]
    
    # Rust uses "value"
    ac_pos = {"latitude": 45.0, "longitude": -122.0, "altitude": 1000.0}
    gs_pos = {"latitude": 45.0, "longitude": -122.0, "altitude": 500.0}
    radio = {
        "freq_mhz": 900.0, "tx_power_dbm": 20.0, "tx_gain_dbi": 3.0,
        "rx_gain_dbi": 3.0, "rx_sensitivity_dbm": -90.0, "fade_margin_db": 10.0
    }
    
    engine = af_core.PyPredictionEngine()
    rust_json = engine.predict_link_reliability_json(
        json.dumps(ac_pos), json.dumps(gs_pos), json.dumps(radio), json.dumps(input_obs)
    )
    rust_res = json.loads(rust_json)
    rust_val = rust_res["predicted_value"]["value"]["value"]
    
    assert assert_float_equivalence(py_val, rust_val), f"Link mismatch: py={py_val}, rust={rust_val}"
    print("  PASS: Link reliability matches within tolerance.")

def test_mission_duration_equivalence():
    print("[Test 3] Mission Duration Equivalence (Nominal)")
    input_obs = [{"source_id": "jsbsim_sim", "timestamp_observed": "2026-11-17T12:00:00Z", "provenance_hash": "test_hash"}]
    
    # Python reference expects distance/track inside the "tasks" array
    task_graph = {
        "tasks": [
            {"resource_id": "uav1", "distance_nm": 10.0, "track_deg": 0.0, "execution_time_min": 0.0}
        ]
    }
    py_res = py_predict_mission_duration(task_graph, tas_kts=100.0, wind_speed_kts=10.0, wind_dir_deg=0.0, input_obs=input_obs)
    
    # DEBUG: Print the entire Python response to see why it's 0.0
    print("DEBUG py_res:", json.dumps(py_res, indent=2))
    
    py_val = py_res["predicted_value"]["value"]["magnitude"]
    
    # Rust expects structured JSON for tasks, transits, schedule, and wind
    tasks = []
    transits = [{
        "segment_id": "s1", "origin_task_id": None, "destination_task_id": None, 
        "distance_nm": 10.0, "track_deg": 0.0, "resource_id": "uav1", "dependencies": []
    }]
    schedule = []
    wind = {"speed_kts": 10.0, "direction_deg": 0.0}
    
    engine = af_core.PyPredictionEngine()
    rust_json = engine.predict_mission_duration_json(
        json.dumps(tasks), json.dumps(transits), json.dumps(schedule), 100.0, json.dumps(wind), json.dumps(input_obs)
    )
    rust_res = json.loads(rust_json)
    rust_val = rust_res["predicted_value"]["value"]["value"]
    
    assert assert_float_equivalence(py_val, rust_val), f"Duration mismatch: py={py_val}, rust={rust_val}"
    print("  PASS: Mission duration matches within tolerance.")
    
def run_equivalence_tests():
    print("="*70)
    print("PYTHON-RUST EQUIVALENCE TESTS")
    print("="*70)
    
    try:
        test_fuel_endurance_equivalence()
        test_link_reliability_equivalence()
        test_mission_duration_equivalence()
        
        print("\n" + "="*70)
        print("ALL EQUIVALENCE TESTS PASSED")
        print("="*70)
        return 0
    except Exception as e:
        print(f"\nFAIL: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(run_equivalence_tests())