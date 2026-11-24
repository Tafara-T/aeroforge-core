import json
import math
import sys
from datetime import datetime, timezone

# --- 1. Contract Helper: Null Prediction Generator ---
def make_null_prediction(attribute, reason, model_id, model_version, input_obs):
    """Generates a contract-compliant Unvalidated/Null prediction."""
    return {
        "attribute": attribute,
        "predicted_value": {"type": "Null"},
        "uncertainty": {
            "representation_type": "Unknown",
            "confidence_level": 0.0,
            "basis": {"Uncalibrated": {"reason": "Insufficient valid input data"}},
            "correlations": []
        },
        "prediction_horizon_s": 0.0,
        "model_id": model_id,
        "model_version": model_version,
        "input_observations": input_obs,
        "known_limitations": [reason],
        "validation_status": {"Unvalidated": {"reason": reason}}
    }

# --- 2. Module A: Fuel Endurance Prediction ---
def predict_fuel_endurance(tank_contents_lbs, engine_flow_pps, reserve_fuel_lbs, input_obs):
    # Finite validation for tanks
    if not all(isinstance(x, (int, float)) and math.isfinite(x) for x in tank_contents_lbs):
        return make_null_prediction("remaining_flight_time_min", "Invalid or missing fuel tank data", "fuel_endurance_v1", "1.1.2", input_obs)
    
    # Finite validation for engines
    if not all(isinstance(x, (int, float)) and math.isfinite(x) and x >= 0 for x in engine_flow_pps):
        return make_null_prediction("remaining_flight_time_min", "Invalid or missing engine fuel flow data", "fuel_endurance_v1", "1.1.2", input_obs)
    
    # Reserve validation
    if not isinstance(reserve_fuel_lbs, (int, float)) or not math.isfinite(reserve_fuel_lbs) or reserve_fuel_lbs < 0:
        return make_null_prediction("remaining_flight_time_min", "Invalid reserve fuel configuration", "fuel_endurance_v1", "1.1.2", input_obs)
    
    total_fuel = sum(tank_contents_lbs)
    total_flow_pps = sum(engine_flow_pps)
    
    if total_flow_pps <= 0:
        return make_null_prediction("remaining_flight_time_min", "Invalid or missing engine fuel flow data", "fuel_endurance_v1", "1.1.2", input_obs)
        
    if total_fuel <= reserve_fuel_lbs:
        return make_null_prediction("remaining_flight_time_min", "Fuel at or below reserve minimum", "fuel_endurance_v1", "1.1.2", input_obs)
        
    usable_fuel = total_fuel - reserve_fuel_lbs
    endurance_min = usable_fuel / (total_flow_pps * 60.0)
    
    return {
        "attribute": "remaining_flight_time_min",
        "predicted_value": {"type": "Float", "value": {"magnitude": round(endurance_min, 2), "unit": "min"}},
        "uncertainty": {
            "representation_type": "BoundedInterval",
            "confidence_level": 0.85,
            "basis": {"Heuristic": {"reason": "Assumes constant flow profile; +/- 15% bounding box"}},
            "correlations": []
        },
        "prediction_horizon_s": endurance_min * 60.0,
        "model_id": "fuel_endurance_v1",
        "model_version": "1.1.2",
        "input_observations": input_obs,
        "known_limitations": ["Assumes constant flow profile. Does not model sudden headwinds or reserve fuel mandates."],
        "validation_status": {"Unvalidated": {"reason": "Heuristic bounding box, not statistically calibrated"}}
    }

# --- 3. Module B: Link Reliability Modeling ---
def predict_link_reliability(ac_lat, ac_lon, ac_alt_ft, gs_lat, gs_lon, gs_alt_ft, freq_mhz, pt_dbm, gt_dbi, gr_dbi, sr_dbm, fade_margin_db, input_obs):
    # Validate finite inputs
    for val in [ac_lat, ac_lon, ac_alt_ft, gs_lat, gs_lon, gs_alt_ft, freq_mhz, pt_dbm, gt_dbi, gr_dbi, sr_dbm, fade_margin_db]:
        if not isinstance(val, (int, float)) or not math.isfinite(val):
            return make_null_prediction("link_degradation_risk_score", "Invalid or missing link geometry/radio data", "link_reliability_v1", "1.1.2", input_obs)
            
    if freq_mhz <= 0:
        return make_null_prediction("link_degradation_risk_score", "Frequency must be strictly positive", "link_reliability_v1", "1.1.2", input_obs)

    # 3D Slant Range (Spherical earth + altitude diff)
    R = 6371.0 # km
    dlat = math.radians(ac_lat - gs_lat)
    dlon = math.radians(ac_lon - gs_lon)
    a = math.sin(dlat/2)**2 + math.cos(math.radians(gs_lat)) * math.cos(math.radians(ac_lat)) * math.sin(dlon/2)**2
    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1-a))
    surface_dist_km = R * c
    
    alt_diff_km = (ac_alt_ft - gs_alt_ft) * 0.3048 / 1000.0
    slant_dist_km = math.sqrt(surface_dist_km**2 + alt_diff_km**2)
    
    if slant_dist_km < 0.001:
        return make_null_prediction("link_degradation_risk_score", "Aircraft and ground station co-located; FSPL model invalid", "link_reliability_v1", "1.1.2", input_obs)
        
    # Terrain screening (conservative geometric rule)
    if ac_alt_ft < gs_alt_ft + 32.8: # 10m in ft
        return make_null_prediction("link_degradation_risk_score", "Potential terrain obstruction; FSPL model outside valid operating conditions", "link_reliability_v1", "1.1.2", input_obs)

    # FSPL
    fspl_db = 20 * math.log10(slant_dist_km) + 20 * math.log10(freq_mhz) + 32.44
    
    # Link Margin
    margin_db = pt_dbm + gt_dbi + gr_dbi - fspl_db - sr_dbm
    
    # Heuristic Risk Score
    if margin_db >= fade_margin_db:
        risk_score = 0.05
    elif margin_db >= 0:
        risk_score = 0.05 + 0.90 * (1.0 - (margin_db / fade_margin_db))
    else:
        risk_score = 0.95
        
    return {
        "attribute": "link_degradation_risk_score",
        "predicted_value": {"type": "Float", "value": {"magnitude": round(risk_score, 3), "unit": "probability"}},
        "uncertainty": {
            "representation_type": "Unknown",
            "confidence_level": 0.0,
            "basis": {"Uncalibrated": {"reason": "Heuristic link-margin proxy, not empirically calibrated packet-loss probability"}},
            "correlations": []
        },
        "prediction_horizon_s": 60.0,
        "model_id": "link_reliability_v1",
        "model_version": "1.1.2",
        "input_observations": input_obs,
        "known_limitations": [
            "Assumes JSBSim HSL matches WGS84 MSL datum. Geoid undulation differences are not modeled.",
            "Terrain buffer is a conservative geometric screening rule, not a verified terrain clearance."
        ],
        "validation_status": {"Unvalidated": {"reason": "Heuristic risk score, not calibrated probability"}}
    }

# --- 4. Module C: Mission Duration Estimation ---
def predict_mission_duration(task_graph, tas_kts, wind_speed_kts, wind_dir_deg, input_obs):
    # Validate finite inputs
    if not isinstance(tas_kts, (int, float)) or not math.isfinite(tas_kts) or tas_kts <= 0:
        return make_null_prediction("total_mission_duration_min", "Invalid or missing true airspeed", "mission_duration_v1", "1.1.2", input_obs)
    if not isinstance(wind_speed_kts, (int, float)) or not math.isfinite(wind_speed_kts) or wind_speed_kts < 0:
        return make_null_prediction("total_mission_duration_min", "Invalid or missing wind speed", "mission_duration_v1", "1.1.2", input_obs)
    if not isinstance(wind_dir_deg, (int, float)) or not math.isfinite(wind_dir_deg):
        return make_null_prediction("total_mission_duration_min", "Invalid or missing wind direction", "mission_duration_v1", "1.1.2", input_obs)

    total_duration_min = 0.0
    
    # Group tasks by resource_id to handle concurrency
    resources = {}
    for task in task_graph.get("tasks", []):
        rid = task.get("resource_id", "primary_aircraft")
        if rid not in resources:
            resources[rid] = []
        resources[rid].append(task)
        
    # For each resource, sum durations sequentially (simplified CPM for single capacity)
    for rid, tasks in resources.items():
        resource_time_min = 0.0
        for task in tasks:
            # Transit
            dist_nm = task.get("distance_nm", 0.0)
            track_deg = task.get("track_deg", 0.0)
            
            if dist_nm > 0:
                # Wind components (Meteorological convention: wind FROM direction)
                wind_rad = math.radians(wind_dir_deg)
                track_rad = math.radians(track_deg)
                
                v_wind_along = -wind_speed_kts * math.cos(wind_rad - track_rad)
                v_wind_cross = wind_speed_kts * math.sin(wind_rad - track_rad)
                
                if abs(v_wind_cross) >= tas_kts:
                    return make_null_prediction("total_mission_duration_min", "Crosswind exceeds true airspeed; track maintenance infeasible", "mission_duration_v1", "1.1.2", input_obs)
                    
                v_ground = math.sqrt(tas_kts**2 - v_wind_cross**2) + v_wind_along
                
                if v_ground <= 0:
                    return make_null_prediction("total_mission_duration_min", "Adverse wind exceeds true airspeed; forward progress infeasible", "mission_duration_v1", "1.1.2", input_obs)
                    
                transit_time_min = (dist_nm / v_ground) * 60.0
                resource_time_min += transit_time_min
                
            # Task execution
            exec_time_min = task.get("execution_time_min", 0.0)
            resource_time_min += exec_time_min
            
        # Critical path is the maximum time across all parallel resources
        total_duration_min = max(total_duration_min, resource_time_min)
        
    return {
        "attribute": "total_mission_duration_min",
        "predicted_value": {"type": "Float", "value": {"magnitude": round(total_duration_min, 2), "unit": "min"}},
        "uncertainty": {
            "representation_type": "BoundedInterval",
            "confidence_level": 0.80,
            "basis": {"Heuristic": {"reason": "+/- 10% bounding box for ATC delays and variable winds"}},
            "correlations": []
        },
        "prediction_horizon_s": total_duration_min * 60.0,
        "model_id": "mission_duration_v1",
        "model_version": "1.1.2",
        "input_observations": input_obs,
        "known_limitations": ["Assumes 1 unit of capacity per resource_id. Does not model complex resource pooling."],
        "validation_status": {"Unvalidated": {"reason": "Heuristic bounding box, not statistically calibrated"}}
    }

# --- 5. Test Scenarios ---
def run_tests():
    print("="*70)
    print("STAGE 3 PREDICTIVE MODULE REFERENCE MODEL TESTS")
    print("="*70)
    
    input_obs = [{"source_id": "jsbsim_sim", "timestamp_observed": "2026-11-17T12:00:00Z", "provenance_hash": "test_hash"}]
    
    # Test 1: Nominal Fuel
    print("\n[Test 1] Nominal Fuel Endurance")
    res = predict_fuel_endurance([50.0, 50.0], [0.02, 0.02], 20.0, input_obs)
    assert res["predicted_value"]["type"] == "Float", f"Expected Float, got {res}"
    assert res["predicted_value"]["value"]["magnitude"] > 0, "Expected positive endurance"
    print("  PASS: Nominal fuel calculated correctly.")
    
    # Test 2: Degraded Fuel (NaN)
    print("\n[Test 2] Degraded Fuel Input (NaN)")
    res = predict_fuel_endurance([float('nan'), 50.0], [0.02], 20.0, input_obs)
    assert res["predicted_value"]["type"] == "Null", f"Expected Null, got {res}"
    assert "Invalid or missing fuel tank data" in res["known_limitations"][0]
    print("  PASS: NaN fuel correctly rejected (fail-closed).")
    
    # Test 3: Nominal Link
    print("\n[Test 3] Nominal Link Reliability")
    res = predict_link_reliability(45.0, -122.0, 1000.0, 45.0, -122.0, 500.0, 900.0, 20.0, 3.0, 3.0, -90.0, 10.0, input_obs)
    assert res["predicted_value"]["type"] == "Float", f"Expected Float, got {res}"
    assert 0.0 <= res["predicted_value"]["value"]["magnitude"] <= 1.0, "Risk score out of bounds"
    print("  PASS: Nominal link risk score calculated.")
    
    # Test 4: Extrapolation Link (Terrain Screening)
    print("\n[Test 4] Link Extrapolation (Terrain Screening)")
    res = predict_link_reliability(45.0, -122.0, 400.0, 45.0, -122.0, 500.0, 900.0, 20.0, 3.0, 3.0, -90.0, 10.0, input_obs)
    assert res["predicted_value"]["type"] == "Null", f"Expected Null, got {res}"
    assert "Potential terrain obstruction" in res["known_limitations"][0]
    print("  PASS: Terrain screening correctly triggered.")
    
    # Test 5: Nominal Mission Duration
    print("\n[Test 5] Nominal Mission Duration")
    graph = {
        "tasks": [
            {"resource_id": "uav_1", "distance_nm": 10.0, "track_deg": 0.0, "execution_time_min": 5.0},
            {"resource_id": "uav_1", "distance_nm": 10.0, "track_deg": 90.0, "execution_time_min": 5.0}
        ]
    }
    res = predict_mission_duration(graph, tas_kts=100.0, wind_speed_kts=10.0, wind_dir_deg=0.0, input_obs=input_obs)
    assert res["predicted_value"]["type"] == "Float", f"Expected Float, got {res}"
    print("  PASS: Nominal mission duration calculated.")
    
    # Test 6: Adversarial Mission Duration (Crosswind > TAS)
    print("\n[Test 6] Adversarial Mission Duration (Crosswind > TAS)")
    graph_adv = {
        "tasks": [
            {"resource_id": "uav_1", "distance_nm": 10.0, "track_deg": 0.0, "execution_time_min": 5.0}
        ]
    }
    # Wind from 90 deg (East), track 0 deg (North). Crosswind = 100 kts. TAS = 50 kts.
    res = predict_mission_duration(graph_adv, tas_kts=50.0, wind_speed_kts=100.0, wind_dir_deg=90.0, input_obs=input_obs)
    assert res["predicted_value"]["type"] == "Null", f"Expected Null, got {res}"
    assert "Crosswind exceeds true airspeed" in res["known_limitations"][0]
    print("  PASS: Crosswind infeasibility correctly detected (no artificial speed capping).")
    
    # Test 7: Concurrent Resources
    print("\n[Test 7] Concurrent Resource Scheduling")
    graph_concurrent = {
        "tasks": [
            {"resource_id": "uav_1", "distance_nm": 10.0, "track_deg": 0.0, "execution_time_min": 10.0},
            {"resource_id": "uav_2", "distance_nm": 10.0, "track_deg": 0.0, "execution_time_min": 10.0}
        ]
    }
    # Both take 10 min transit (10nm/100kts*60 = 6 min) + 10 min exec = 16 min. 
    # Max of resources = 16 min. (If sequential, it would be 32 min).
    res = predict_mission_duration(graph_concurrent, tas_kts=100.0, wind_speed_kts=0.0, wind_dir_deg=0.0, input_obs=input_obs)
    assert abs(res["predicted_value"]["value"]["magnitude"] - 16.0) < 0.1, f"Expected ~16.0, got {res['predicted_value']['value']['magnitude']}"
    print("  PASS: Concurrent resource scheduling correctly calculated (max parallel time).")

    print("\n" + "="*70)
    print("ALL STAGE 3 REFERENCE MODEL TESTS PASSED")
    print("="*70)
    return 0

if __name__ == "__main__":
    sys.exit(run_tests())
