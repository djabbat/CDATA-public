//! Ядро платформы симуляции клеточной дифференцировки

pub mod agent_population;
pub mod components;
pub mod error;
pub mod module;
pub mod population;
pub mod simulation;
pub mod world;
pub mod ze_validation;

pub use agent_population::*;
pub use components::*;
pub use error::*;
pub use module::{SimulationModule, CdataCollect};
pub use population::*;
pub use simulation::*;
pub use world::*;
pub use ze_validation::*;

pub use hecs;

/// Константы для тестов
#[cfg(test)]
pub mod test_constants {
    pub const TEST_CELL_COUNT: usize = 10;
    pub const TEST_STEPS: u64 = 100;
    pub const TEST_DT: f64 = 0.1;
}
