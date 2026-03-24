# cell_dt_core

Core ECS engine and biological components for the **Cell Digital Twin (CDATA)** — a Rust simulation platform for the *Centriolar Damage Accumulation Theory of Aging* (Tkemaladze J., 2023).

## Overview

`cell_dt_core` provides:

- **ECS world** — entity-component storage via [`hecs`](https://crates.io/crates/hecs); each entity is a stem cell niche
- **`SimulationModule` trait** — plug-in interface for biological modules (`step`, `initialize`, `get_params`, `set_params`)
- **`SimulationManager`** — orchestrates module registration, ECS world, background simulation thread, mpsc snapshot channel
- **Biological components** — ready-to-use ECS components for centriolar aging:

| Component | Description |
|-----------|-------------|
| `CentriolePair` | Mother/daughter centriole with PTM signatures and inducer sets |
| `CentriolarDamageState` | 5 molecular + 4 appendage damage fields; derived ciliary/spindle metrics |
| `HumanDevelopmentComponent` | Full CDATA state: stage, age, damage, inducers, tissue |
| `MitochondrialState` | Track E: mtDNA mutations, ROS, mitophagy, mito_shield |
| `StemCellDivisionRateState` | Track F: division rate as product of cilia/spindle/age/ROS/mTOR drives |
| `TelomereState` | Track C: mean length, shortening per division, Hayflick G1 arrest |
| `EpigeneticClockState` | Track D: methylation age, clock acceleration |
| `SenescenceAccumulationState` | Senescent fraction → SASP feedback loop (P65) |
| `TrackABCrossState` | Cross-penalty when cilia and spindle both degrade (P66) |

## Usage

```rust
use cell_dt_core::{SimulationManager, SimulationConfig, SimulationModule};
use cell_dt_core::components::{CentriolePair, CellCycleStateExtended};

let config = SimulationConfig {
    max_steps: 36500,
    dt: 1.0,
    seed: Some(42),
    ..Default::default()
};

let mut sim = SimulationManager::new(config);
// sim.register_module(Box::new(MyModule::new()))?;
// sim.world_mut().spawn((CentriolePair::default(), CellCycleStateExtended::new()));
// sim.initialize()?;
// sim.step()?;
```

## Theory

Based on: Tkemaladze J. *The Centriolar Damage Accumulation Theory of Aging.* Mol Biol Reports (2023). [PMID 36583780](https://pubmed.ncbi.nlm.nih.gov/36583780/)

Preprint + data: [Zenodo DOI 10.5281/zenodo.19174506](https://doi.org/10.5281/zenodo.19174506)

## License

MIT OR Apache-2.0
