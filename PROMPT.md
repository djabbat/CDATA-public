# Claude Code — Промпты для генерации кода CDATA v3.0

## Инструкция

Скопируй и выполни эти промпты последовательно в Claude Code. Каждый промпт создает один крейт или модуль. После выполнения всех промптов запусти `cargo build --workspace` и `cargo test --workspace`.

---

## Prompt 1: Создание workspace и core крейта

Создай Rust workspace для CDATA v3.0 со следующей структурой:

Cargo.toml (workspace root):
```toml
[workspace]
members = [
    "crates/cell_dt_core",
    "crates/cell_dt_modules/mitochondrial",
    "crates/cell_dt_modules/inflammaging",
    "crates/cell_dt_modules/asymmetric_division",
    "crates/cell_dt_modules/tissue_specific",
    "crates/cell_dt_validation",
    "crates/cell_dt_gui",
    "crates/cell_dt_python",
]
resolver = "2"
```

Теперь создай крейт `crates/cell_dt_core`:

Cargo.toml для core:
```toml
[package]
name = "cell_dt_core"
version = "0.1.0"
edition = "2021"

[dependencies]
hecs = "0.10"
serde = { version = "1.0", features = ["derive"] }
rand = "0.8"
rand_chacha = "0.3"
anyhow = "1.0"
thiserror = "1.0"
float_eq = "1.0"

[dev-dependencies]
rand = "0.8"
```

Создай src/lib.rs:
```rust
pub mod components;
pub mod systems;
pub mod parameters;
pub mod prelude;

pub use components::*;
pub use systems::*;
pub use parameters::*;
```

Создай src/parameters/mod.rs:
```rust
mod fixed_params;
pub use fixed_params::*;
```

Создай src/parameters/fixed_params.rs с FixedParameters struct (32 параметра из CONCEPT.md).

Создай src/components/mod.rs:
```rust
mod tissue_state;
mod mitochondrial_state;
mod inflammaging_state;
mod youth_protection;
mod asymmetric_inheritance;
mod chip_state;

pub use tissue_state::*;
pub use mitochondrial_state::*;
pub use inflammaging_state::*;
pub use youth_protection::*;
pub use asymmetric_inheritance::*;
pub use chip_state::*;
```

Создай каждый компонент с соответствующими полями и методами.

---

## Prompt 2: Mitochondrial модуль

Создай крейт `crates/cell_dt_modules/mitochondrial`:

```toml
[package]
name = "cell_dt_mitochondrial"
version = "0.1.0"
edition = "2021"

[dependencies]
cell_dt_core = { path = "../../cell_dt_core" }
serde = { version = "1.0", features = ["derive"] }
rand = "0.8"
```

Создай src/components.rs:
- MitochondrialState struct с полями: mtdna_mutations, ros_level, mito_shield, mitophagy_efficiency, membrane_potential, fusion_frequency, base_ros
- impl MitochondrialState с методами:
  - update_ros(mtdna_damage, oxidative_input, dt) -> f64 (сигмоидальная формула)
  - update_mito_shield() -> f64
  - update_mitophagy(ros_level, age_years)
  - accumulate_mtdna_mutations(ros_level, dt) -> f64
- MitochondrialParams struct с параметрами: mitophagy_threshold, ros_steepness, max_ros, base_ros_young, hormesis_factor

Создай src/system.rs:
- MitochondrialSystem struct с полем params: MitochondrialParams
- impl MitochondrialSystem с методами:
  - new() -> Self
  - update(world, dt, age_years, inflammation_level)
  - calculate_oxygen_delivery(state, age_years) -> f64
  - check_mitochondrial_collapse(state) -> bool

Добавь тесты в tests/ для проверки сигмоидального поведения и порога митофагии.

---

## Prompt 3: Inflammaging модуль

Создай крейт `crates/cell_dt_modules/inflammaging`:

```toml
[package]
name = "cell_dt_inflammaging"
version = "0.1.0"
edition = "2021"

[dependencies]
cell_dt_core = { path = "../../cell_dt_core" }
serde = { version = "1.0", features = ["derive"] }
```

Создай src/components.rs:
- InflammagingState struct с полями: sasp_level, cgas_sting_activity, damps_level, nk_efficiency, fibrosis_level, senescent_cell_fraction
- impl InflammagingState с методами:
  - update_damps(senescent_fraction, dna_damage, dt)
  - update_cgas_sting(damps_level, mtdna_release)
  - update_sasp(cgas_sting, senescent_fraction, nfkb_activity, dt)
  - sasp_to_ros_contribution() -> f64
  - sasp_damage_multiplier() -> f64
  - nk_elimination(current_senescent, dt) -> f64
  - update_nk_efficiency(age_years, sasp_level)
  - update_fibrosis(sasp_level, dt)
- InflammagingParams struct

Создай src/system.rs:
- InflammagingSystem struct
- impl InflammagingSystem с методами:
  - update(world, dt, age_years, dna_damage, nfkb_activity)
  - get_sasp_ros_contribution(world) -> f64
  - get_sasp_damage_multiplier(world) -> f64
  - eliminate_senescent_cells(world, senescent_fraction, dt) -> f64

Добавь тесты для проверки обратной связи SASP→ROS.

---

## Prompt 4: Asymmetric Division модуль

Создай крейт `crates/cell_dt_modules/asymmetric_division`:

```toml
[package]
name = "cell_dt_asymmetric_division"
version = "0.1.0"
edition = "2021"

[dependencies]
cell_dt_core = { path = "../../cell_dt_core" }
serde = { version = "1.0", features = ["derive"] }
rand = "0.8"
rand_chacha = "0.3"
```

Создай src/stochastic.rs:
- AsymmetricInheritance struct с полями: inheritance_probability, inherited_maternal, rng
- impl AsymmetricInheritance с методами:
  - new(seed) -> Self
  - calculate_probability(age_years, spindle_fidelity, niche_integrity, oxidative_stress) -> f64
  - roll_inheritance(...) -> bool
  - damage_accumulation_multiplier(&self) -> f64
- AsymmetryStatistics struct с методами record_division, asymmetry_fraction

Создай src/chip_drift.rs:
- ChipDriverMutation enum: DNMT3A, TET2, ASXL1, JAK2, Other
- impl ChipDriverMutation с методами:
  - fitness_advantage(age_years) -> f64
  - mutation_rate() -> f64
  - sasp_sensitivity() -> f64
- ChipClone struct
- ChipState struct с полями: clones, total_chip_frequency, dominant_clone, detection_age
- impl ChipState с методами:
  - update(division_rate, sasp_level, age_years, dt, rng)
  - generate_new_mutations(...)
  - expand_clones(...)
  - hematologic_risk() -> f64

Добавь тесты для проверки распределения вероятностей наследования.

---

## Prompt 5: Tissue Specific модуль

Создай крейт `crates/cell_dt_modules/tissue_specific`:

```toml
[package]
name = "cell_dt_tissue_specific"
version = "0.1.0"
edition = "2021"

[dependencies]
cell_dt_core = { path = "../../cell_dt_core" }
serde = { version = "1.0", features = ["derive"] }
```

Создай src/tissue_specific.rs:
- TissueType enum: Hematopoietic, Intestinal, Muscle, Neural
- TissueSpecificParams struct с полями: tissue_type, base_division_rate, damage_per_division_multiplier, centriole_repair_efficiency, youth_protection_duration, sasp_sensitivity, regenerative_potential, tolerance
- impl TissueSpecificParams:
  - for_tissue(tissue: TissueType) -> Self
    - HSC: base=12, damage=1.0, tolerance=0.3
    - ISC: base=70, damage=0.3, tolerance=0.8
    - Muscle: base=4, damage=1.2, tolerance=0.5
    - Neural: base=2, damage=1.5, tolerance=0.2
  - effective_division_rate(systemic_factors, local_sasp) -> f64
  - damage_accumulation_multiplier(age_years) -> f64

Создай src/system.rs (опционально) с AgingEngineV2, объединяющим все модули.

Добавь тесты для проверки эффективной скорости деления для каждой ткани.

---

## Prompt 6: Validation модуль

Создай крейт `crates/cell_dt_validation`:

```toml
[package]
name = "cell_dt_validation"
version = "0.1.0"
edition = "2021"

[dependencies]
cell_dt_core = { path = "../../cell_dt_core" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
csv = "1.3"
anyhow = "1.0"
thiserror = "1.0"
float_eq = "1.0"
ndarray = "0.15"
rand = "0.8"
rand_distr = "0.4"
```

Создай src/biomarkers.rs:
- BiomarkerType enum: RosLevel, MtdnaMutations, ChipFrequency, EpigeneticClock, StemCellPool, TelomereLength
- BiomarkerDataPoint struct с полями: age, value, std_dev, n_samples
- BiomarkerDataset struct с полями: name, biomarker_type, values, source_pmid
- impl BiomarkerDataset:
  - load_from_csv(path) -> Result<Self>
  - max_age() -> f64
  - min_age() -> f64

Создай src/calibration.rs:
- Calibrator struct
- CalibrationParams struct с 32 параметрами
- impl Calibrator:
  - new() -> Self
  - calibrate(simulation, datasets, age_range) -> Result<CalibrationParams>
  - mcmc_nuts(...) -> Result<Posterior>
  - calculate_r2(...) -> f64
  - calculate_rmse(...) -> f64

Создай src/validation.rs:
- Validator struct
- ValidationResult struct: r_squared, rmse, auc, mae
- ValidationMetrics struct
- impl Validator:
  - validate(simulation, datasets, params) -> Result<ValidationResult>
  - cross_validate(...) -> Vec<ValidationResult>
  - blind_predict(...) -> PredictionResult
  - sensitivity_analysis(...) -> Vec<SensitivityResult>

Создай examples/validation_example.rs для демонстрации.

---

## Prompt 7: GUI модуль

Создай крейт `crates/cell_dt_gui`:

```toml
[package]
name = "cell_dt_gui"
version = "0.1.0"
edition = "2021"

[dependencies]
cell_dt_core = { path = "../../cell_dt_core" }
cell_dt_mitochondrial = { path = "../../cell_dt_modules/mitochondrial" }
cell_dt_inflammaging = { path = "../../cell_dt_modules/inflammaging" }
cell_dt_asymmetric_division = { path = "../../cell_dt_modules/asymmetric_division" }
cell_dt_tissue_specific = { path = "../../cell_dt_modules/tissue_specific" }
cell_dt_validation = { path = "../../cell_dt_validation" }
eframe = "0.24"
egui = "0.24"
egui_plot = "0.24"
serde = { version = "1.0", features = ["derive"] }
rfd = "0.12"
chrono = "0.4"
```

Создай src/main.rs:
```rust
mod app;
mod simulation_tab;
mod parameters_tab;
mod validation_tab;
mod results_tab;
mod interventions_tab;

use app::CdataApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "CDATA Cell-DT v3.0",
        options,
        Box::new(|_cc| Box::new(CdataApp::default())),
    )
}
```

Создай src/app.rs:
- CdataApp struct с полями: simulation_state, params, validation_results, current_tab
- impl eframe::App for CdataApp:
  - update(&mut self, ctx, frame)
  - 4 пресета: Control, Longevity, Progeria, CentrosomeTransplant@50yr
  - 8 интервенций: Senolytics, NAD+, CR, TERT, Antioxidant, CafdRetainer, CafdReleaser, CentrosomeTransplant
  - Экспорт CSV
  - 3 языка (English, Русский, Georgian)

Создай simulation_tab.rs с графиками frailty, stem_cell_pool, ROS, CAII, epigenetic_clock, CHIP_frequency.

---

## Prompt 8: Финальная сборка и тесты

Создай недостающие файлы и тесты:

1. Создай `crates/cell_dt_python/Cargo.toml` для PyO3 биндингов:
```toml
[package]
name = "cell_dt_python"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
cell_dt_core = { path = "../../cell_dt_core" }
pyo3 = { version = "0.20", features = ["extension-module"] }
```

2. Создай `crates/cell_dt_python/src/lib.rs` с pyfunction для основных функций.

3. Создай `examples/basic_simulation.rs`:
   - Инициализация мира
   - Добавление тканей
   - Запуск симуляции на 100 лет
   - Вывод результатов

4. Создай `examples/interventions.rs`:
   - Симуляция Control
   - Симуляция с Senolytics
   - Сравнение lifespan

5. Создай `tests/integration_tests.rs`:
   - test_full_simulation
   - test_interventions
   - test_validation_pipeline

6. Создай `datasets/` с CSV файлами:
   - datasets/ros_by_age.csv
   - datasets/chip_frequency.csv
   - datasets/epigenetic_clock.csv
   - datasets/stem_cell_pool.csv
   - datasets/telomere_length.csv

7. Запусти `cargo build --workspace` и убедись, что все компилируется.
8. Запусти `cargo test --workspace` и убедись, что все тесты проходят.

---

## После генерации

1. **Проверь сборку:** `cargo build --workspace`
2. **Запусти тесты:** `cargo test --workspace`
3. **Запусти GUI:** `cargo run -p cell_dt_gui`
4. **Запусти пример:** `cargo run --example basic_simulation`

Если есть ошибки, сообщи их для исправления.
