# AeroForge Core (AF)
**Aerospace Intelligence, Autonomous Systems & Engineering Innovation**  
*Blueprint: AF Production Blueprint v2.0 | Stage 1: Aerospace Intelligence Foundation*

## Overview
AeroForge Core is the foundational intelligence layer for aerospace mission execution. It provides:
- **Generic, Extensible State Modeling**: Schema-driven entity representation without hardcoded aircraft/sensor structs.
- **Policy-Based Uncertainty Management**: Metadata-driven tolerance validation with fail-closed safety for unknown parameters.
- **Dynamic Reality Snapshots**: Real-time aggregation of validated aerospace state with full provenance tracking.
- **Tamper-Evident Archiving**: Cryptographic hash-chain logging for operational integrity and post-mission forensics.

## Architecture
- **Rust (`rust/af-core`)**: High-performance, memory-safe core logic (state management, uncertainty validation, schema parsing). Exposed to Python via PyO3.
- **Python (`python/af_orchestrator`)**: Mission orchestration, external data ingestion, and tamper-evident SQLite archiving.
- **Schemas (`schemas/`)**: JSON-defined entity schemas (e.g., `aircraft.json`, `environment.json`) loaded at runtime.

## Build & Installation
Requires Rust 1.70+ and Python 3.10+.


# 1. Create and activate a Python virtual environment
python3 -m venv .venv
source .venv/bin/activate

# 2. Install build dependencies
pip install maturin

# 3. Build and install the Rust extension
cd rust/af-core
maturin develop --release
cd ../..

# 4. Run validation tests
python3 -m pytest tests/  # (Tests to be added in Stage 2)

# AEROFORGE (AF) Core

**Governing Architecture:** AF Production Blueprint v2.0  
**Current Status:** Stage 2 (Integration) & Stage 3 (Predictive Reference Models) Validated.

AEROFORGE (AF) is a high-integrity, fail-closed mission intelligence and constraint-evaluation engine designed for aerospace autonomy. It strictly separates measured reality from predictive models, enforces hard/soft constraint hierarchies, and provides defensible, auditable capability predictions.

## 🏗️ Architecture Overview

- **`rust/af-core/`**: The high-performance, safety-critical Rust core. Implements the `AfStateUpdate`, `RealityModel`, `UncertaintyManager`, and `CapabilityPrediction` contracts. Exposed to Python via PyO3.
- **`python/af_orchestrator/`**: The Python integration layer that bridges external data sources (simulators, telemetry) to the Rust core.
- **`tests/`**: Comprehensive integration and reference-model tests validating contract adherence, temporal logic, and mathematical defensibility.
- **`schemas/`**: JSON schema definitions for aircraft and environment state.

## ✅ Validated Capabilities

### Stage 2: Reality Model & Constraint Integration
- **Fail-Closed Execution**: Hard gates strictly block mission execution on missing or unreliable evidence.
- **Evidence Quality Tracking**: Distinguishes between `Valid`, `Contradicted`, and `Unknown` data sources.
- **High-Frequency Ingestion**: Python-to-Rust ingestion path tested at ~870 steps/sec (~1.15ms/step) on local development hardware. In a controlled 600-step JSBSim simulation, history retention resulted in ~0.02 MB memory growth. (Note: Long-term production deployments require active history pruning, as the current reference implementation retains an unbounded history vector).
- **Numerical Contradiction Detection**: Automatically flags state deviations exceeding registered tolerance policies.

### Stage 3: Predictive Intelligence (Production Implementation Validated)
- **Production Rust Implementation**: Three mathematically rigorous prediction functions (`predict_fuel_endurance`, `predict_link_reliability`, `predict_mission_duration`) implemented in `rust/af-core/src/prediction.rs` with PyO3 bindings.
- **Fail-Closed Design**: Models explicitly return `StateValue::Null` and `ValidationStatus::Unvalidated` when inputs are missing, NaN, or physically infeasible (e.g., crosswind exceeding true airspeed, dependency cycles detected).
- **DAG-Based Critical Path Scheduling**: Mission duration prediction implements topological sort with cycle detection and resource serialization, correctly handling parallel and sequential task execution across multiple resources.
- **Python-Rust Equivalence Validated**: All three prediction models produce mathematically equivalent outputs (within `1e-6` tolerance) when given identical inputs, verified by dedicated equivalence tests.
- **Comprehensive Test Coverage**: 15 Rust unit tests covering nominal, boundary, adversarial, and complex scheduling scenarios, all passing.
- **Backward Compatibility**: Existing `Task` contract remains completely untouched. New scheduling metadata isolated in separate `TransitSegment` and `TaskScheduleEntry` structs.

### Stage 3: Predictive Intelligence (Reference Models)
- **Fuel Endurance Prediction**: Mass-flow-based calculation with strict reserve handling and fail-closed rejection of invalid/NaN inputs.
- **Link Reliability Modeling**: 3D slant-range Free Space Path Loss (FSPL) calculation outputting a heuristic risk score (explicitly not a calibrated probability).
- **Mission Duration Estimation**: Critical-path task graph evaluation with meteorological wind-triangle ground speed calculations and crosswind infeasibility detection.

## 🚀 Getting Started

### Prerequisites
- Rust (1.70+)
- Python 3.10+
- `jsbsim` Python package (for simulation testing)

### Setup
```bash
# Clone the repository
git clone https://github.com/Tafara-T/aeroforge-core.git
cd aeroforge-core

# Build the Rust core
cd rust/af-core
cargo build --release

# (Optional) Set up Python virtual environment
cd ../..
python3 -m venv .venv
source .venv/bin/activate
pip install jsbsim