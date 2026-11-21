import jsbsim
import json
from datetime import datetime, timezone

print('='*70)
print('JSBSIM API INSPECTION FOR AF INTEGRATION')
print('='*70)

# 1. Initialize JSBSim with correct API
fdm = jsbsim.FGFDMExec()
aircraft = 'c172p'
print(f'
[1] Loading aircraft model: {aircraft}')
fdm.load_model(aircraft)

# 2. Set initial conditions
fdm['ic/h-sl-ft'] = 1000.0        # Altitude (feet)
fdm['ic/long-gc-deg'] = -122.0    # Longitude (degrees)
fdm['ic/lat-gc-deg'] = 45.0       # Latitude (degrees)
fdm['ic/psi-true-deg'] = 90.0     # Heading (degrees)
fdm['ic/vc-kts'] = 100.0          # Airspeed (knots)

print('[2] Initializing simulation...')
fdm.run_ic()

# 3. Run for a few steps and inspect properties
print('
[3] Running simulation and inspecting state variables:')
print('-' * 70)

mapping = {
    'gps_altitude': ('position/h-sl-ft', 'ft'),
    'gps_latitude': ('position/lat-gc-deg', 'deg'),
    'gps_longitude': ('position/long-gc-deg', 'deg'),
    'true_airspeed': ('velocities/vc-kts', 'kts'),
    'roll_angle': ('attitude/phi-deg', 'deg'),
    'pitch_angle': ('attitude/theta-deg', 'deg'),
    'heading': ('attitude/psi-deg', 'deg'),
}

for step in range(3):
    fdm.run()
    sim_time = fdm.get_sim_time()
    print(f'
Simulation Time: {sim_time:.2f} seconds')
    print(f'{"AF Target Attribute":<25} | {"JSBSim Property":<30} | {"Value":<12} | {"Unit"}')
    print('-' * 70)
    for af_attr, (jsb_prop, unit) in mapping.items():
        value = fdm[jsb_prop]
        print(f'{af_attr:<25} | {jsb_prop:<30} | {value:<12.4f} | {unit}')

# 4. Demonstrate AfStateUpdate JSON generation
print('
[4] Demonstrating AF AfStateUpdate JSON generation:')
now_iso = datetime.now(timezone.utc).isoformat()
sample_update = {
    'entity_id': 'sim_c172p_01',
    'variable_name': 'gps_altitude',
    'value': {
        'type': 'Numerical',
        'value': {
            'magnitude': float(fdm['position/h-sl-ft']),
            'unit': 'ft'
        }
    },
    'temporal': {
        'timestamp_observed': now_iso,
        'timestamp_ingested': now_iso,
        'validity_duration_ms': 1000,
        'freshness_threshold_ms': 500
    },
    'provenance': {
        'source_id': 'jsbsim_sim',
        'provenance_hash': 'sim_hash_001',
        'calibration_ref': None
    },
    'uncertainty': {
        'state': 'Predicted'
    }
}
print(json.dumps(sample_update, indent=2))

print('
' + '='*70)
print('INSPECTION COMPLETE')
print('='*70)
print('Key Findings:')
print('  1. JSBSim properties accessed via fdm["property/name"]')
print('  2. Units are mixed (ft, deg, kts). AF must tag units explicitly.')
print('  3. Simulation time via fdm.get_sim_time()')
print('  4. Mapping to AfStateUpdate is deterministic.')
