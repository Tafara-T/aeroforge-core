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