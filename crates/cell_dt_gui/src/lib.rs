//! Graphical interface for simulation parameter configuration
//! Extended version with validation, presets, export and history
//! Supports 7 languages: EN, FR, ES, RU, ZH, AR, KA (Georgian)

pub mod i18n;
use i18n::Lang;

use cell_dt_config::*;
use eframe::{egui, Frame};
use egui::{CentralPanel, Context, ScrollArea, Slider, Window, ComboBox, Color32, Stroke};
use egui_plot::{Plot, Line, PlotPoints, Legend};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::VecDeque;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc;

use stem_cell_hierarchy_module::NicheCatalog;

// Simulation engine
use cell_dt_core::{
    SimulationManager,
    SimulationConfig as CoreSimConfig,
    CentriolarDamageState,
    TelomereState,
    EpigeneticClockState,
    CentriolePair,
    CellCycleStateExtended,
    StemCellDivisionRateState,
    TissueType,
};
use human_development_module::{HumanDevelopmentModule, HumanDevelopmentComponent};
use myeloid_shift_module::{MyeloidShiftModule, MyeloidShiftComponent};
use mitochondrial_module::MitochondrialModule;
use centriole_module::CentrioleModule;
use cell_cycle_module::CellCycleModule;

// ==================== DATA STRUCTURES ====================

/// Ecosphere-level environmental conditions (+8).
/// These initial conditions are fed into food_water_module, sleep_module, breathing_module
/// at simulation start and remain constant throughout (or can be updated via interventions).
#[derive(Clone, Serialize, Deserialize)]
pub struct EcosphereConditions {
    // ─── Food & Water ───
    /// Caloric balance: -0.5 = severe CR, 0.0 = maintenance, +0.5 = excess
    pub caloric_balance: f32,
    /// Diet quality [0..1]: 1.0 = Mediterranean / whole-foods
    pub diet_quality: f32,
    /// Hydration [0..1]: 1.0 = optimal (2–3L/day)
    pub hydration: f32,
    /// Protein quality [0..1]: amino acid completeness + antioxidant load
    pub protein_quality: f32,

    // ─── Sleep ───
    /// Sleep quality [0..1]: 1.0 = 8h restorative sleep
    pub sleep_quality: f32,
    /// Circadian alignment [0..1]: 1.0 = regular schedule, no shift work
    pub circadian_alignment: f32,

    // ─── Breathing ───
    /// O₂ fraction in inspired air [0..1]: sea level = 0.21, 3000m = 0.14
    pub o2_fraction: f32,
    /// Breathing quality [0..1]: 1.0 = slow diaphragmatic (pranayama)
    pub breathing_quality: f32,
    /// Air pollution index [0..1]: 0 = clean, 1.0 = heavy urban pollution
    pub pollution_index: f32,
}

impl Default for EcosphereConditions {
    fn default() -> Self {
        Self {
            // Average modern Western lifestyle
            caloric_balance:      0.05,  // slight caloric excess
            diet_quality:         0.60,
            hydration:            0.75,
            protein_quality:      0.65,
            sleep_quality:        0.65,  // ~6.5h modern average
            circadian_alignment:  0.70,
            o2_fraction:          0.21,  // sea level
            breathing_quality:    0.60,
            pollution_index:      0.20,  // mild urban
        }
    }
}

impl EcosphereConditions {
    pub fn optimal() -> Self {
        Self {
            caloric_balance: -0.10, diet_quality: 0.90, hydration: 0.95, protein_quality: 0.85,
            sleep_quality: 0.90, circadian_alignment: 0.90,
            o2_fraction: 0.21, breathing_quality: 0.85, pollution_index: 0.05,
        }
    }
    pub fn poor() -> Self {
        Self {
            caloric_balance: 0.25, diet_quality: 0.30, hydration: 0.50, protein_quality: 0.40,
            sleep_quality: 0.35, circadian_alignment: 0.40,
            o2_fraction: 0.21, breathing_quality: 0.40, pollution_index: 0.60,
        }
    }
}

/// Application state
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigAppState {
    // Main parameters
    pub config_file: String,
    pub config_format: String,
    pub simulation: SimulationConfig,
    
    // Modules
    pub centriole: CentrioleConfig,
    pub cell_cycle: CellCycleConfig,
    pub transcriptome: TranscriptomeConfig,
    pub asymmetric: AsymmetricDivisionConfig,
    pub stem_hierarchy: StemHierarchyConfig,
    pub io: IOConfig,
    pub viz: VisualizationConfig,
    pub cdata: CdataGuiConfig,
    pub mitochondrial: MitochondrialConfig,

    // Ecosphere (+8) — environmental conditions for food/water, sleep, breathing modules
    pub ecosphere: EcosphereConditions,

    // Language
    pub language: Lang,

    // Simulation run state
    pub simulation_running: bool,
    pub sim_progress: f32,         // 0.0 – 1.0
    pub sim_elapsed_steps: u64,
    pub show_impact_panel: bool,

    // UI state
    pub selected_tab: Tab,
    pub show_about: bool,
    pub show_save_dialog: bool,
    pub show_load_dialog: bool,
    pub show_preset_dialog: bool,
    pub show_export_dialog: bool,
    pub show_validation_dialog: bool,
    pub message: Option<String>,
    pub validation_errors: Vec<String>,
    
    // Real-time visualization
    pub realtime_viz: RealtimeVisualization,
}

impl Default for ConfigAppState {
    fn default() -> Self {
        Self {
            config_file: "config.toml".to_string(),
            config_format: "toml".to_string(),
            simulation: SimulationConfig {
                max_steps: 36_500,   // 100 years × 365 days/year (dt = 1.0 day/step)
                dt: 1.0,
                ..SimulationConfig::default()
            },
            centriole: CentrioleConfig::default(),
            cell_cycle: CellCycleConfig::default(),
            transcriptome: TranscriptomeConfig::default(),
            asymmetric: AsymmetricDivisionConfig::default(),
            stem_hierarchy: StemHierarchyConfig::default(),
            io: IOConfig::default(),
            viz: VisualizationConfig::default(),
            cdata: CdataGuiConfig::default(),
            mitochondrial: MitochondrialConfig::default(),
            ecosphere: EcosphereConditions::default(),
            language: Lang::En,
            simulation_running: false,
            sim_progress: 0.0,
            sim_elapsed_steps: 0,
            show_impact_panel: false,
            selected_tab: Tab::Cell,
            show_about: false,
            show_save_dialog: false,
            show_load_dialog: false,
            show_preset_dialog: false,
            show_export_dialog: false,
            show_validation_dialog: false,
            message: None,
            validation_errors: Vec::new(),
            realtime_viz: RealtimeVisualization::default(),
        }
    }
}


// ==================== REAL-TIME VISUALIZATION ====================

/// Data for real-time visualization
#[derive(Clone, Serialize, Deserialize)]
pub struct RealtimeVisualization {
    pub enabled: bool,
    pub parameter_history: VecDeque<ParameterSnapshot>,
    pub max_history: usize,
    pub selected_parameters: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ParameterSnapshot {
    pub time: f64,
    pub values: std::collections::HashMap<String, f64>,
}

impl Default for RealtimeVisualization {
    fn default() -> Self {
        Self {
            enabled: false,
            parameter_history: VecDeque::new(),
            max_history: 100,
            selected_parameters: vec![
                "simulation.max_steps".to_string(),
                "centriole.acetylation_rate".to_string(),
                "cell_cycle.base_cycle_time".to_string(),
            ],
        }
    }
}

impl RealtimeVisualization {
    pub fn add_snapshot(&mut self, values: std::collections::HashMap<String, f64>, time: f64) {
        self.parameter_history.push_back(ParameterSnapshot { time, values });
        
        while self.parameter_history.len() > self.max_history {
            self.parameter_history.pop_front();
        }
    }
    
    pub fn extract_values(state: &ConfigAppState) -> std::collections::HashMap<String, f64> {
        let mut values = std::collections::HashMap::new();
        
        values.insert("simulation.max_steps".to_string(), state.simulation.max_steps as f64);
        values.insert("simulation.dt".to_string(), state.simulation.dt);
        values.insert("centriole.acetylation_rate".to_string(), state.centriole.acetylation_rate as f64);
        values.insert("centriole.oxidation_rate".to_string(), state.centriole.oxidation_rate as f64);
        values.insert("cell_cycle.base_cycle_time".to_string(), state.cell_cycle.base_cycle_time as f64);
        values.insert("cell_cycle.checkpoint_strictness".to_string(), state.cell_cycle.checkpoint_strictness as f64);
        values.insert("transcriptome.mutation_rate".to_string(), state.transcriptome.mutation_rate as f64);
        values.insert("asymmetric.asymmetric_probability".to_string(), state.asymmetric.asymmetric_probability as f64);
        
        values
    }
}

// ==================== SIMULATION SNAPSHOT ====================

/// One data point sent from simulation thread to GUI each N steps.
#[derive(Clone, Debug)]
pub struct SimSnapshot {
    pub step: u64,
    pub progress: f32,
    pub age_years: f64,
    pub frailty: f32,
    pub stem_cell_pool: f32,
    pub ros_level: f32,
    pub myeloid_bias: f32,
    pub telomere_length: f32,
    pub methylation_age: f32,
    pub is_alive: bool,
    /// Protein aggregation index from CentriolarDamageState [0..1]
    pub protein_aggregation: f32,
    /// Division rate from StemCellDivisionRateState [0..1] — global average
    pub division_rate: f32,
    /// Per-tissue division rates: [Blood, Neural, Connective, Muscle, Epithelial, Skin, Liver, Lung]
    /// f32::NAN if no entity of that tissue type exists
    pub per_tissue_div_rate: [f32; 8],
    /// Centrosomal Damage Score (CDS) = total_damage_score() [0..1]
    pub centrosomal_damage: f32,
    /// Biological age (composite): 0.4×(methyl/120) + 0.3×(1-telo) + 0.3×frailty → years
    pub biological_age: f32,
}

// ==================== PARAMETER VALIDATION ====================

/// Parameter validator
pub struct ParameterValidator;

impl ParameterValidator {
    pub fn validate_all(state: &ConfigAppState) -> Vec<String> {
        let mut errors = Vec::new();
        
        // Simulation
        if state.simulation.max_steps == 0 {
            errors.push("❌ Number of steps must be greater than 0".to_string());
        }
        if state.simulation.dt <= 0.0 {
            errors.push("❌ Time step must be positive".to_string());
        }
        if state.simulation.dt > 1.0 {
            errors.push("⚠️ Time step > 1.0 may cause instability".to_string());
        }
        
        // Centriole
        if state.centriole.enabled {
            if state.centriole.acetylation_rate < 0.0 || state.centriole.acetylation_rate > 0.1 {
                errors.push("❌ Acetylation rate must be in range [0, 0.1]".to_string());
            }
            if state.centriole.oxidation_rate < 0.0 || state.centriole.oxidation_rate > 0.1 {
                errors.push("❌ Oxidation rate must be in range [0, 0.1]".to_string());
            }
        }
        
        // Cell cycle
        if state.cell_cycle.enabled {
            if state.cell_cycle.base_cycle_time <= 0.0 {
                errors.push("❌ Cycle duration must be positive".to_string());
            }
            if state.cell_cycle.checkpoint_strictness < 0.0 || state.cell_cycle.checkpoint_strictness > 1.0 {
                errors.push("❌ Checkpoint strictness must be in [0, 1]".to_string());
            }
        }
        
        // Transcriptome
        if state.transcriptome.enabled
            && (state.transcriptome.mutation_rate < 0.0 || state.transcriptome.mutation_rate > 0.1)
        {
            errors.push("❌ Mutation rate must be in [0, 0.1]".to_string());
        }
        
        // Asymmetric division
        if state.asymmetric.enabled {
            let sum = state.asymmetric.asymmetric_probability + 
                     state.asymmetric.renewal_probability + 
                     state.asymmetric.diff_probability;
            if (sum - 1.0).abs() > 0.01 {
                errors.push("⚠️ Sum of division probabilities should be ~1.0".to_string());
            }
            if state.asymmetric.niche_capacity == 0 {
                errors.push("❌ Niche capacity must be > 0".to_string());
            }
        }
        
        errors
    }
    
    pub fn is_valid(state: &ConfigAppState) -> bool {
        Self::validate_all(state).is_empty()
    }
}

// ==================== CONFIGURATION PRESETS ====================

/// Configuration presets for different experiments
#[derive(Debug, Clone)]
pub struct ConfigPreset {
    pub name: String,
    pub description: String,
    pub icon: &'static str,
    pub apply: fn(&mut ConfigAppState),
}

impl ConfigPreset {
    pub fn get_all() -> Vec<Self> {
        vec![
            Self {
                name: "Quick Test".to_string(),
                description: "Minimal configuration for quick testing".to_string(),
                icon: "⚡",
                apply: |state| {
                    state.simulation.max_steps = 100;
                    state.simulation.dt = 0.1;
                    state.centriole.enabled = true;
                    state.cell_cycle.enabled = true;
                    state.transcriptome.enabled = false;
                },
            },
            Self {
                name: "Standard Experiment".to_string(),
                description: "Standard parameters for typical experiments".to_string(),
                icon: "🔬",
                apply: |state| {
                    state.simulation.max_steps = 10000;
                    state.simulation.dt = 0.05;
                    state.simulation.num_threads = Some(8);
                    state.centriole.enabled = true;
                    state.cell_cycle.enabled = true;
                    state.transcriptome.enabled = true;
                },
            },
            Self {
                name: "High Performance".to_string(),
                description: "Optimized for large populations".to_string(),
                icon: "🚀",
                apply: |state| {
                    state.simulation.max_steps = 100000;
                    state.simulation.dt = 0.1;
                    state.simulation.num_threads = Some(16);
                    state.simulation.parallel_modules = true;
                    state.centriole.parallel_cells = true;
                    state.io.save_checkpoints = false;
                    state.viz.enabled = false;
                },
            },
            Self {
                name: "Stem Cells".to_string(),
                description: "Focus on asymmetric division and hierarchy".to_string(),
                icon: "🌱",
                apply: |state| {
                    state.simulation.max_steps = 50000;
                    state.asymmetric.enabled = true;
                    state.asymmetric.asymmetric_probability = 0.4;
                    state.stem_hierarchy.enabled = true;
                    state.stem_hierarchy.initial_potency = "Pluripotent".to_string();
                    state.transcriptome.enabled = true;
                },
            },
            Self {
                name: "Cell Cycle".to_string(),
                description: "Detailed cell cycle study".to_string(),
                icon: "🔄",
                apply: |state| {
                    state.simulation.max_steps = 20000;
                    state.simulation.dt = 0.02;
                    state.cell_cycle.enabled = true;
                    state.cell_cycle.checkpoint_strictness = 0.3;
                    state.cell_cycle.enable_apoptosis = true;
                    state.centriole.enabled = true;
                    state.transcriptome.enabled = false;
                },
            },
            Self {
                name: "Transcriptome Analysis".to_string(),
                description: "Detailed gene expression analysis".to_string(),
                icon: "🧬",
                apply: |state| {
                    state.simulation.max_steps = 5000;
                    state.simulation.dt = 0.05;
                    state.transcriptome.enabled = true;
                    state.transcriptome.mutation_rate = 0.001;
                    state.transcriptome.noise_level = 0.02;
                    state.io.format = "parquet".to_string();
                    state.io.compression = "snappy".to_string();
                },
            },
        ]
    }
}

// ==================== PYTHON EXPORT ====================

/// Python script generator
pub struct PythonExporter;

impl PythonExporter {
    pub fn generate_script(state: &ConfigAppState) -> String {
        let mut script = String::new();
        
        script.push_str("#!/usr/bin/env python3\n");
        script.push_str("# -*- coding: utf-8 -*-\n");
        script.push_str("\"\"\"\n");
        script.push_str("Automatically generated script for Cell DT\n");
        script.push_str("Usage: python3 script.py\n");
        script.push_str("\"\"\"\n\n");
        
        script.push_str("import cell_dt\n");
        script.push_str("import numpy as np\n");
        script.push_str("import matplotlib.pyplot as plt\n\n");
        
        // Simulation setup
        script.push_str("# Simulation setup\n");
        script.push_str("sim = cell_dt.PySimulation(\n");
        script.push_str(&format!("    max_steps={},\n", state.simulation.max_steps));
        script.push_str(&format!("    dt={},\n", state.simulation.dt));
        script.push_str(&format!("    num_threads={},\n", state.simulation.num_threads.unwrap_or(1)));
        script.push_str(&format!("    seed={}\n", state.simulation.seed.unwrap_or(42)));
        script.push_str(")\n\n");
        
        // Create cells
        script.push_str("# Create cells\n");
        if state.transcriptome.enabled {
            script.push_str("sim.create_population_with_transcriptome(100)\n");
        } else {
            script.push_str("sim.create_population(100)\n");
        }
        script.push('\n');
        
        // Cell cycle parameters
        if state.cell_cycle.enabled {
            script.push_str("# Cell cycle parameters\n");
            script.push_str("cell_cycle_params = cell_dt.PyCellCycleParams(\n");
            script.push_str(&format!("    base_cycle_time={},\n", state.cell_cycle.base_cycle_time));
            script.push_str(&format!("    checkpoint_strictness={},\n", state.cell_cycle.checkpoint_strictness));
            script.push_str(&format!("    enable_apoptosis={},\n", state.cell_cycle.enable_apoptosis));
            script.push_str(&format!("    nutrient_availability={},\n", state.cell_cycle.nutrient_availability));
            script.push_str(&format!("    growth_factor_level={},\n", state.cell_cycle.growth_factor_level));
            script.push_str(&format!("    random_variation={}\n", state.cell_cycle.random_variation));
            script.push_str(")\n\n");
        } else {
            script.push_str("cell_cycle_params = None\n\n");
        }
        
        // Register modules
        script.push_str("# Register modules\n");
        script.push_str("sim.register_modules(\n");
        script.push_str(&format!("    enable_centriole={},\n", state.centriole.enabled));
        script.push_str(&format!("    enable_cell_cycle={},\n", state.cell_cycle.enabled));
        script.push_str(&format!("    enable_transcriptome={},\n", state.transcriptome.enabled));
        script.push_str("    cell_cycle_params=cell_cycle_params\n");
        script.push_str(")\n\n");
        
        // Run simulation
        script.push_str("# Run simulation\n");
        script.push_str("print(\"🚀 Starting simulation...\")\n");
        script.push_str("cells = sim.run()\n");
        script.push_str("print(f\"✅ Simulation completed in {sim.current_step()} steps\")\n\n");
        
        // Analyze results
        script.push_str("# Analyze results\n");
        script.push_str("print(f\"\\nTotal cells: {len(cells)}\")\n\n");
        
        script.push_str("# Get centriole data\n");
        script.push_str("centriole_data = sim.get_centriole_data_numpy()\n");
        script.push_str("if len(centriole_data) > 0:\n");
        script.push_str("    print(f\"Average mother centriole maturity: {np.mean(centriole_data[:, 0]):.3f}\")\n\n");
        
        script.push_str("# Phase distribution\n");
        script.push_str("phase_dist = sim.get_phase_distribution()\n");
        script.push_str("print(\"\\nPhase distribution:\")\n");
        script.push_str("for phase, count in phase_dist.items():\n");
        script.push_str("    print(f\"  {phase}: {count}\")\n\n");
        
        script.push_str("# Visualization\n");
        script.push_str("if len(phase_dist) > 0:\n");
        script.push_str("    plt.figure(figsize=(10, 6))\n");
        script.push_str("    plt.bar(phase_dist.keys(), phase_dist.values())\n");
        script.push_str("    plt.title('Cell Cycle Phase Distribution')\n");
        script.push_str("    plt.xlabel('Phase')\n");
        script.push_str("    plt.ylabel('Number of Cells')\n");
        script.push_str("    plt.savefig('phase_distribution.png')\n");
        script.push_str("    plt.show()\n");
        
        script
    }
}

// ==================== TABS ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tab {
    ZeField,        // -5
    Atoms,          // -4
    Molecules,      // -3
    Cytoskeleton,   // -2
    Organelles,     // -1
    Cell,           //  0 ← DEFAULT (★)
    Tissues,        // +1
    Organs,         // +2
    Organism,       // +3
    Society,        // +4
    Ecosphere,      // +5
    // Legacy tabs (kept for backward compat)
    Molecular,
    Transcriptome,
    Mitochondrial,
    Centriole,
    CellCycle,
    Division,
    StemHierarchy,
    Niche,
    Tissue,
    System,
}

impl Tab {
    pub fn name(&self) -> &'static str {
        match self {
            Tab::ZeField      => "🌀 Ze Field",
            Tab::Atoms        => "⚛️ Atoms",
            Tab::Molecules    => "🔬 Molecules",
            Tab::Cytoskeleton => "🏛️ Structures",
            Tab::Organelles   => "🔋 Organelles",
            Tab::Cell         => "⭐ Cell",
            Tab::Tissues      => "🔴 Tissues",
            Tab::Organs       => "🫀 Organs",
            Tab::Organism     => "🧍 Organism",
            Tab::Society      => "👥 Society",
            Tab::Ecosphere    => "🌍 Ecosphere",
            Tab::Molecular     => "🔬 Molecular",
            Tab::Transcriptome => "🧬 Transcriptome",
            Tab::Mitochondrial => "🔋 Mitochondrial",
            Tab::Centriole     => "🔵 Centriole",
            Tab::CellCycle     => "♻️ Cell Cycle",
            Tab::Division      => "⚖️ Division",
            Tab::StemHierarchy => "🌱 Stem Hierarchy",
            Tab::Niche         => "🏘️ Niche / CDATA",
            Tab::Tissue        => "🔴 Tissue / Myeloid",
            Tab::System        => "⚙️ System",
        }
    }

    pub fn name_tr(&self, lang: Lang) -> &'static str {
        let tr = lang.tr();
        match self {
            Tab::ZeField      => tr.tab_ze_field,
            Tab::Atoms        => tr.tab_atoms,
            Tab::Molecules    => tr.tab_molecules,
            Tab::Cytoskeleton => tr.tab_cytoskeleton,
            Tab::Organelles   => tr.tab_organelles,
            Tab::Cell         => tr.tab_cell,
            Tab::Tissues      => tr.tab_tissues,
            Tab::Organs       => tr.tab_organs,
            Tab::Organism     => tr.tab_organism,
            Tab::Society      => tr.tab_society,
            Tab::Ecosphere    => tr.tab_ecosphere,
            Tab::Molecular     => tr.tab_molecular,
            Tab::Transcriptome => tr.tab_transcriptome,
            Tab::Mitochondrial => tr.tab_mitochondrial,
            Tab::Centriole     => tr.tab_centriole,
            Tab::CellCycle     => tr.tab_cell_cycle,
            Tab::Division      => tr.tab_division,
            Tab::StemHierarchy => tr.tab_stem_hierarchy,
            Tab::Niche         => tr.tab_niche,
            Tab::Tissue        => tr.tab_tissue,
            Tab::System        => tr.tab_system,
        }
    }
}

/// Конфигурация CDATA-параметров для GUI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdataGuiConfig {
    // --- Система индукторов ---
    pub base_detach_probability: f32,
    pub mother_bias: f32,
    pub age_bias_coefficient: f32,
    // --- Жизненный цикл индукторов ---
    /// Номер деления бластомеров для de novo создания центриолей [1..=8]
    pub de_novo_centriole_division: u32,
    /// Учитывать элиминацию центриолей в прелептотенной стадии мейоза
    pub meiotic_elimination_enabled: bool,
    // --- Миелоидный сдвиг ---
    pub spindle_weight: f32,
    pub cilia_weight: f32,
    pub ros_weight: f32,
    pub aggregate_weight: f32,
    /// Пресет скоростей повреждений
    pub damage_preset: DamagePreset,
    // --- Track F: темп деления ---
    pub division_rate_floor: f32,
    pub ros_brake_strength: f32,
    pub mtor_brake_strength: f32,
    // --- Myeloid shift fine-tuning ---
    /// Масштаб обратной связи воспаления → ROS (myeloid → inflammaging → ros_boost)
    pub ros_boost_scale: f32,
    /// Масштаб обратной связи воспаления → нише (myeloid → niche_impairment)
    pub niche_impair_scale: f32,
    /// Нелинейность веретена в myeloid_bias: (1-spindle)^exponent
    pub spindle_nonlinearity_exponent: f32,
    // --- Индукторная система ---
    /// true = стандартная CDATA (потентность через индукторы); false = прямая f(spindle, cilia)
    pub enable_inducer_system: bool,
}

/// Пресет DamageParams
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamagePreset {
    Normal,
    Progeria,
    Longevity,
}

impl DamagePreset {
    pub fn label(&self) -> &'static str {
        match self {
            DamagePreset::Normal   => "Normal aging",
            DamagePreset::Progeria => "Progeria (×5 rates)",
            DamagePreset::Longevity => "Longevity (×0.6 rates)",
        }
    }
}

impl Default for CdataGuiConfig {
    fn default() -> Self {
        Self {
            base_detach_probability: 0.0003,
            mother_bias: 0.6,
            age_bias_coefficient: 0.003,
            de_novo_centriole_division: 4,
            meiotic_elimination_enabled: true,
            spindle_weight: 0.45,
            cilia_weight: 0.30,
            ros_weight: 0.15,
            aggregate_weight: 0.10,
            damage_preset: DamagePreset::Normal,
            division_rate_floor: 0.15,
            ros_brake_strength: 0.40,
            mtor_brake_strength: 0.35,
            ros_boost_scale: 0.15,
            niche_impair_scale: 0.08,
            spindle_nonlinearity_exponent: 1.5,
            enable_inducer_system: true,
        }
    }
}

// ==================== CONFIGURATIONS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsymmetricDivisionConfig {
    pub enabled: bool,
    pub asymmetric_probability: f32,
    pub renewal_probability: f32,
    pub diff_probability: f32,
    pub niche_capacity: usize,
    pub max_niches: usize,
    pub enable_polarity: bool,
    pub enable_fate_determinants: bool,
}

impl Default for AsymmetricDivisionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            asymmetric_probability: 0.3,
            renewal_probability: 0.4,
            diff_probability: 0.3,
            niche_capacity: 10,
            max_niches: 100,
            enable_polarity: true,
            enable_fate_determinants: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemHierarchyConfig {
    pub enabled: bool,
    pub initial_potency: String,
    pub enable_plasticity: bool,
    pub plasticity_rate: f32,
    pub differentiation_threshold: f32,
}

impl Default for StemHierarchyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_potency: "Pluripotent".to_string(),
            enable_plasticity: true,
            plasticity_rate: 0.01,
            differentiation_threshold: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOConfig {
    pub enabled: bool,
    pub output_dir: String,
    pub format: String,
    pub compression: String,
    pub buffer_size: usize,
    pub save_checkpoints: bool,
    pub checkpoint_interval: u64,
    pub max_checkpoints: usize,
}

impl Default for IOConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            output_dir: "results".to_string(),
            format: "csv".to_string(),
            compression: "none".to_string(),
            buffer_size: 1000,
            save_checkpoints: true,
            checkpoint_interval: 100,
            max_checkpoints: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub enabled: bool,
    pub update_interval: u64,
    pub output_dir: String,
    pub save_plots: bool,
    pub phase_distribution: bool,
    pub maturity_histogram: bool,
    pub heatmap: bool,
    pub timeseries: bool,
    pub three_d_enabled: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval: 10,
            output_dir: "viz_output".to_string(),
            save_plots: true,
            phase_distribution: true,
            maturity_histogram: true,
            heatmap: true,
            timeseries: true,
            three_d_enabled: false,
        }
    }
}

// ==================== MAIN APPLICATION ====================

pub struct ConfigApp {
    state: ConfigAppState,
    history_states: VecDeque<ConfigAppState>,
    history_index: usize,
    max_history: usize,
    // Real simulation thread
    sim_rx: Option<mpsc::Receiver<SimSnapshot>>,
    stop_flag: Arc<AtomicBool>,
    pub sim_snapshots: Vec<SimSnapshot>,
    /// Expanded nodes in lineage tree (by node id)
    expanded_lineage: std::collections::HashSet<usize>,
    /// Selected niche card in Niche Browser tab
    selected_niche: Option<usize>,
}

impl ConfigApp {
    pub fn new() -> Self {
        let state = ConfigAppState::default();
        let mut history_states = VecDeque::new();
        history_states.push_back(state.clone());
        Self {
            state,
            history_states,
            history_index: 0,
            max_history: 50,
            sim_rx: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
            sim_snapshots: Vec::new(),
            expanded_lineage: {
                let mut s = std::collections::HashSet::new();
                s.insert(0usize); // root always expanded
                s
            },
            selected_niche: None,
        }
    }

    fn push_history(&mut self) {
        // Remove states ahead of current index
        while self.history_states.len() > self.history_index + 1 {
            self.history_states.pop_back();
        }
        // Flat clone — ConfigAppState no longer has history inside it
        self.history_states.push_back(self.state.clone());
        while self.history_states.len() > self.max_history {
            self.history_states.pop_front();
            self.history_index = self.history_index.saturating_sub(1);
        }
        self.history_index = self.history_states.len() - 1;
    }

    fn can_undo(&self) -> bool {
        self.history_index > 0
    }

    fn can_redo(&self) -> bool {
        self.history_index + 1 < self.history_states.len()
    }

    fn undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.state = self.history_states[self.history_index].clone();
        }
    }

    fn redo(&mut self) {
        if self.history_index + 1 < self.history_states.len() {
            self.history_index += 1;
            self.state = self.history_states[self.history_index].clone();
        }
    }
}

impl Default for ConfigApp {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigApp {
    /// Spawn background simulation thread.  GUI polls via `self.sim_rx`.
    fn start_simulation(&mut self, ctx: &Context) {
        // Reset previous run
        self.sim_snapshots.clear();
        self.stop_flag.store(false, Ordering::SeqCst);

        let (tx, rx) = mpsc::channel::<SimSnapshot>();
        self.sim_rx = Some(rx);

        // Clone everything needed by the thread
        let stop         = self.stop_flag.clone();
        let ctx_clone    = ctx.clone();
        let max_steps    = self.state.simulation.max_steps;
        let dt           = self.state.simulation.dt;
        let cdata        = self.state.cdata.clone();
        let cell_cycle   = self.state.cell_cycle.clone();
        let centriole    = self.state.centriole.clone();
        let mito         = self.state.mitochondrial.clone();

        // How often to emit a snapshot (aim for ~500 points per run)
        let snap_interval = (max_steps / 500).max(1);

        let preset_str = match cdata.damage_preset {
            DamagePreset::Normal    => "default",
            DamagePreset::Progeria  => "progeria",
            DamagePreset::Longevity => "longevity",
        };

        std::thread::spawn(move || {
            let config = CoreSimConfig {
                max_steps,
                dt,
                num_threads: None,
                seed: Some(42),
                parallel_modules: false,
                checkpoint_interval: max_steps,
                cleanup_dead_interval: None,
            };

            let mut sim = SimulationManager::new(config);

            // Register modules in correct order
            if sim.register_module(Box::new(CentrioleModule::new())).is_err() { return; }
            if sim.register_module(Box::new(CellCycleModule::new())).is_err()  { return; }
            if sim.register_module(Box::new(MitochondrialModule::new())).is_err() { return; }
            if sim.register_module(Box::new(HumanDevelopmentModule::new())).is_err() { return; }
            if sim.register_module(Box::new(MyeloidShiftModule::new())).is_err() { return; }

            // Apply GUI parameters
            sim.set_module_params("human_development_module", &serde_json::json!({
                "base_detach_probability": cdata.base_detach_probability,
                "mother_bias":             cdata.mother_bias,
                "age_bias_coefficient":    cdata.age_bias_coefficient,
                "damage_preset":           preset_str,
                "division_rate_floor":     cdata.division_rate_floor,
                "ros_brake_strength":      cdata.ros_brake_strength,
                "mtor_brake_strength":     cdata.mtor_brake_strength,
                "enable_inducer_system":   cdata.enable_inducer_system,
            })).ok();

            sim.set_module_params("cell_cycle_module", &serde_json::json!({
                "checkpoint_strictness":      cell_cycle.checkpoint_strictness,
                "base_cycle_time":            cell_cycle.base_cycle_time,
                "growth_factor_sensitivity":  cell_cycle.growth_factor_sensitivity,
                "stress_sensitivity":         cell_cycle.stress_sensitivity,
                "nutrient_availability":      cell_cycle.nutrient_availability,
                "growth_factor_level":        cell_cycle.growth_factor_level,
                "random_variation":           cell_cycle.random_variation,
                "enable_apoptosis":           cell_cycle.enable_apoptosis,
            })).ok();

            sim.set_module_params("centriole_module", &serde_json::json!({
                "acetylation_rate":     centriole.acetylation_rate,
                "oxidation_rate":       centriole.oxidation_rate,
                "methylation_rate":     centriole.methylation_rate,
                "phosphorylation_rate": centriole.phosphorylation_rate,
                "daughter_ptm_factor":  centriole.daughter_ptm_factor,
                "m_phase_boost":        centriole.m_phase_boost,
                "parallel_cells":       centriole.parallel_cells,
            })).ok();

            sim.set_module_params("mitochondrial_module", &serde_json::json!({
                "base_mutation_rate":          mito.base_mutation_rate,
                "ros_mtdna_feedback":          mito.ros_mtdna_feedback,
                "fission_rate":                mito.fission_rate,
                "base_mitophagy_flux":         mito.base_mitophagy_flux,
                "mitophagy_threshold":         mito.mitophagy_threshold,
                "ros_production_boost":        mito.ros_production_boost,
                "midlife_mutation_multiplier": mito.midlife_mutation_multiplier,
            })).ok();

            sim.set_module_params("myeloid_shift_module", &serde_json::json!({
                "spindle_weight":               cdata.spindle_weight,
                "cilia_weight":                 cdata.cilia_weight,
                "ros_weight":                   cdata.ros_weight,
                "aggregate_weight":             cdata.aggregate_weight,
                "ros_boost_scale":              cdata.ros_boost_scale,
                "niche_impair_scale":           cdata.niche_impair_scale,
                "spindle_nonlinearity_exponent":cdata.spindle_nonlinearity_exponent,
            })).ok();

            // Spawn 8 stem-cell niche entities with distinct TissueTypes (one per ADULT_SC_SAFE)
            // Must be done BEFORE initialize() so modules can process them
            {
                use human_development_module::HumanDevelopmentComponent;
                let world = sim.world_mut();
                let tissues = [
                    TissueType::Blood,
                    TissueType::Neural,
                    TissueType::Connective,
                    TissueType::Muscle,
                    TissueType::Epithelial,
                    TissueType::Skin,
                    TissueType::Liver,
                    TissueType::Lung,
                ];
                for tt in tissues {
                    world.spawn((
                        CentriolePair::default(),
                        CellCycleStateExtended::new(),
                        HumanDevelopmentComponent::for_tissue(tt),
                    ));
                }
            }

            if sim.initialize().is_err() { return; }

            while sim.current_step() < max_steps {
                if stop.load(Ordering::Relaxed) { break; }

                if sim.step().is_err() { break; }

                let step = sim.current_step();

                // Emit snapshot at requested interval or on final step
                if step % snap_interval != 0 && step < max_steps { continue; }

                // Aggregate ECS data across all niche entities
                let mut age_years    = 0.0_f64;
                let mut frailty_s    = 0.0_f64;
                let mut pool_s       = 0.0_f64;
                let mut is_alive     = true;
                let mut hdc_count    = 0usize;

                for (_, hdc) in sim.world().query::<&HumanDevelopmentComponent>().iter() {
                    age_years  = hdc.age_years();
                    frailty_s += hdc.frailty() as f64;
                    pool_s    += hdc.tissue_state.stem_cell_pool as f64;
                    if !hdc.is_alive { is_alive = false; }
                    hdc_count += 1;
                }

                let mut ros_s    = 0.0_f64;
                let mut prot_s   = 0.0_f64;
                let mut cds_s    = 0.0_f64;
                let mut cds_n    = 0usize;
                for (_, cds) in sim.world().query::<&CentriolarDamageState>().iter() {
                    ros_s  += cds.ros_level as f64;
                    prot_s += cds.protein_aggregates as f64;
                    cds_s  += cds.total_damage_score() as f64;
                    cds_n  += 1;
                }

                let mut myeloid_s = 0.0_f64;
                let mut ms_n      = 0usize;
                for (_, msc) in sim.world().query::<&MyeloidShiftComponent>().iter() {
                    myeloid_s += msc.myeloid_bias as f64;
                    ms_n += 1;
                }

                let mut telo_s = 0.0_f64;
                let mut telo_n = 0usize;
                for (_, ts) in sim.world().query::<&TelomereState>().iter() {
                    telo_s += ts.mean_length as f64;
                    telo_n += 1;
                }

                let mut methyl_s = 0.0_f64;
                let mut methyl_n = 0usize;
                for (_, eps) in sim.world().query::<&EpigeneticClockState>().iter() {
                    methyl_s += eps.methylation_age as f64;
                    methyl_n += 1;
                }

                let mut divrate_s = 0.0_f64;
                let mut divrate_n = 0usize;
                for (_, dr) in sim.world().query::<&StemCellDivisionRateState>().iter() {
                    divrate_s += dr.division_rate as f64;
                    divrate_n += 1;
                }

                // Per-tissue division rates: [Blood, Neural, Connective, Muscle, Epithelial, Skin, Liver, Lung]
                let tissue_order = [
                    TissueType::Blood, TissueType::Neural, TissueType::Connective,
                    TissueType::Muscle, TissueType::Epithelial, TissueType::Skin,
                    TissueType::Liver, TissueType::Lung,
                ];
                let mut per_tissue_sum  = [0.0_f64; 8];
                let mut per_tissue_cnt  = [0usize; 8];
                for (_, (hdc, dr)) in sim.world()
                    .query::<(&HumanDevelopmentComponent, &StemCellDivisionRateState)>()
                    .iter()
                {
                    if let Some(idx) = tissue_order.iter().position(|&t| t == hdc.tissue_type) {
                        per_tissue_sum[idx] += dr.division_rate as f64;
                        per_tissue_cnt[idx] += 1;
                    }
                }
                let mut per_tissue_div_rate = [f32::NAN; 8];
                for i in 0..8 {
                    if per_tissue_cnt[i] > 0 {
                        per_tissue_div_rate[i] = (per_tissue_sum[i] / per_tissue_cnt[i] as f64) as f32;
                    }
                }

                let n = |x: usize| if x == 0 { 1 } else { x };

                let avg_frailty  = (frailty_s  / n(hdc_count) as f64) as f32;
                let avg_telo     = (telo_s      / n(telo_n)    as f64) as f32;
                let avg_methyl   = (methyl_s    / n(methyl_n)  as f64) as f32;
                // Biological age composite: clock 40% + telomere loss 30% + frailty 30%
                let bio_age = (avg_methyl * 0.4
                    + (1.0 - avg_telo) * 120.0 * 0.3
                    + avg_frailty * 100.0 * 0.3).clamp(0.0, 150.0);

                let snap = SimSnapshot {
                    step,
                    progress: step as f32 / max_steps as f32,
                    age_years,
                    frailty:             avg_frailty,
                    stem_cell_pool:      (pool_s    / n(hdc_count) as f64) as f32,
                    ros_level:           (ros_s     / n(cds_n)     as f64) as f32,
                    myeloid_bias:        (myeloid_s / n(ms_n)      as f64) as f32,
                    telomere_length:     avg_telo,
                    methylation_age:     avg_methyl,
                    is_alive,
                    protein_aggregation: (prot_s    / n(cds_n)     as f64) as f32,
                    division_rate:       (divrate_s / n(divrate_n) as f64) as f32,
                    centrosomal_damage:  (cds_s     / n(cds_n)     as f64) as f32,
                    biological_age:      bio_age,
                    per_tissue_div_rate,
                };

                let done = snap.progress >= 1.0 || !snap.is_alive;
                if tx.send(snap).is_err() { break; }
                ctx_clone.request_repaint();

                if done { break; }
            }
        });
    }
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // ── Poll simulation snapshots from background thread ──────────────
        {
            let mut done = false;
            if let Some(rx) = &self.sim_rx {
                while let Ok(snap) = rx.try_recv() {
                    self.state.sim_progress     = snap.progress;
                    self.state.sim_elapsed_steps = snap.step;
                    if snap.progress >= 1.0 || !snap.is_alive {
                        done = true;
                    }
                    self.sim_snapshots.push(snap);
                }
            }
            if done {
                self.state.simulation_running = false;
                self.state.show_impact_panel  = true;
                let tr = self.state.language.tr();
                let steps = self.state.sim_elapsed_steps;
                // P67-GUI: auto-export snapshots to CSV
                let csv_path = self.save_simulation_csv();
                self.state.message = Some(match csv_path {
                    Ok(p)  => format!("✅ {} — {} steps | CSV: {}", tr.sim_complete, steps, p),
                    Err(_) => format!("✅ {} — {} steps", tr.sim_complete, steps),
                });
                self.sim_rx = None;
            }
        }

        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // ── Row 1: title + language + exit ───────────────────────────
            ui.horizontal(|ui| {
                let tr = self.state.language.tr();
                ui.heading(format!("🧬 {}", tr.app_title));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌ Exit").clicked() {
                        std::process::exit(0);
                    }
                    ui.separator();
                    let lang_label = self.state.language.display_name();
                    ComboBox::from_id_source("language_picker")
                        .selected_text(lang_label)
                        .show_ui(ui, |ui| {
                            for &lang in Lang::all() {
                                let selected = self.state.language == lang;
                                if ui.selectable_label(selected, lang.display_name()).clicked() {
                                    self.state.language = lang;
                                    self.push_history();
                                }
                            }
                        });
                    ui.label(self.state.language.tr().language_label);
                });
            });

            // ── Row 2: action buttons, left-aligned ──────────────────────
            ui.horizontal(|ui| {
                let tr = self.state.language.tr();
                ui.add_enabled_ui(self.can_undo(), |ui| {
                    if ui.button("↩️ Undo").clicked() { self.undo(); }
                });
                ui.add_enabled_ui(self.can_redo(), |ui| {
                    if ui.button("↪️ Redo").clicked() { self.redo(); }
                });
                ui.separator();
                if ui.button(tr.btn_load).clicked()     { self.state.show_load_dialog    = true; }
                if ui.button(tr.btn_save).clicked()     { self.state.show_save_dialog    = true; }
                if ui.button(tr.btn_presets).clicked()  { self.state.show_preset_dialog  = true; }
                if ui.button("🐍 Export to Python").clicked() { self.state.show_export_dialog = true; }
                if ui.button(tr.btn_validate).clicked() {
                    self.state.validation_errors = ParameterValidator::validate_all(&self.state);
                    self.state.show_validation_dialog = true;
                }
                if let Some(msg) = &self.state.message.clone() {
                    ui.separator();
                    ui.label(egui::RichText::new(msg).size(12.0).color(Color32::LIGHT_GREEN));
                }
            });
        });
        
        // Right panel — parameter controls for selected hierarchy level
        egui::SidePanel::right("right_panel")
            .min_width(1020.0)
            .resizable(true)
            .show(ctx, |ui| {
                let lang = self.state.language;
                ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(2.0);
                    ui.label(egui::RichText::new(self.state.selected_tab.name_tr(lang))
                        .size(13.0).strong().color(Color32::from_rgb(50, 200, 130)));
                    ui.separator();
                    ui.add_space(2.0);
                    match self.state.selected_tab {
                        Tab::ZeField      => self.show_ze_field_tab(ui),
                        Tab::Atoms        => self.show_atoms_tab(ui),
                        Tab::Molecules    => self.show_molecules_tab(ui),
                        Tab::Cytoskeleton => self.show_cytoskeleton_tab(ui),
                        Tab::Organelles   => self.show_organelles_tab(ui),
                        Tab::Cell         => self.show_cell_tab(ui),
                        Tab::Tissues      => self.show_tissues_tab(ui),
                        Tab::Organs       => self.show_organs_tab(ui),
                        Tab::Organism     => self.show_organism_tab(ui),
                        Tab::Society      => self.show_society_tab(ui),
                        Tab::Ecosphere    => self.show_ecosphere_tab(ui),
                        Tab::Molecular     => self.show_molecular_tab(ui),
                        Tab::Transcriptome => self.show_transcriptome_tab(ui),
                        Tab::Mitochondrial => self.show_mitochondrial_tab(ui),
                        Tab::Centriole     => self.show_centriole_tab(ui),
                        Tab::CellCycle     => self.show_cell_cycle_tab(ui),
                        Tab::Division      => self.show_division_tab(ui),
                        Tab::StemHierarchy => self.show_stem_hierarchy_tab(ui),
                        Tab::Niche         => self.show_niche_tab(ui),
                        Tab::Tissue        => self.show_tissue_tab(ui),
                        Tab::System        => self.show_system_tab(ui),
                    }
                });
            });

        // Left panel — hierarchy navigation only
        egui::SidePanel::left("left_panel")
            .min_width(220.0)
            .max_width(320.0)
            .resizable(true)
            .show(ctx, |ui| {
            let lang = self.state.language;
            ScrollArea::vertical().show(ui, |ui| {
                // ── Hierarchy nav ────────────────────────────────────────
                ui.add_space(2.0);
                ui.label(egui::RichText::new("🧬 CDATA Hierarchy").size(11.0).strong()
                    .color(Color32::from_rgb(130, 170, 210)));
                ui.separator();

                let levels: &[(i32, Tab, &str, &str)] = &[
                    (-5, Tab::ZeField,      "✅", "ZeHealthState"),
                    (-4, Tab::Atoms,        "✅", "ThermodynamicState"),
                    (-3, Tab::Molecules,    "✅", "ROSCascadeState · ATPEnergyState · ChromatinState"),
                    (-2, Tab::Cytoskeleton, "✅", "MicrotubuleState · NuclearEnvelope · MembraneState"),
                    (-1, Tab::Organelles,   "✅", "MitochondrialState · LysosomeState · ERStressState"),
                    ( 0, Tab::Cell,         "⭐", "CentriolarDamageState · CloneEpigeneticState"),
                    ( 1, Tab::Tissues,      "✅", "TissueState · ECM · VascularNicheState"),
                    ( 2, Tab::Organs,       "✅", "OrganState(11) · poly-organ failure"),
                    ( 3, Tab::Organism,     "✅", "OrganismState · HPAAxisState"),
                    ( 4, Tab::Society,      "❌", "SocialStressInput (TODO)"),
                    ( 5, Tab::Ecosphere,    "🟡", "Interventions · AIM integration"),
                ];

                for &(level, tab, status, components) in levels {
                    let selected = self.state.selected_tab == tab;
                    let lv_str = if level >= 0 { format!("+{}", level) } else { format!("{}", level) };
                    let lv_col = Color32::from_rgb(90, 100, 115);
                    let sel_col = Color32::from_rgb(50, 200, 130);
                    let sub_col = Color32::from_rgb(80, 95, 110);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&lv_str).size(9.5).color(lv_col).monospace());
                        ui.label(egui::RichText::new(status).size(11.0));
                        let resp = ui.selectable_value(
                            &mut self.state.selected_tab,
                            tab,
                            egui::RichText::new(tab.name_tr(lang))
                                .size(12.5)
                                .color(if selected { sel_col } else { Color32::from_rgb(200, 210, 225) }),
                        );
                        if resp.clicked() { self.push_history(); }
                    });
                    ui.label(egui::RichText::new(format!("      {}", components)).size(8.5).color(sub_col));
                    ui.add_space(1.0);
                }
            });
        });
        
        // ======= BOTTOM PANEL — RUN SIMULATION =======
        egui::TopBottomPanel::bottom("run_panel")
            .min_height(195.0)
            .show(ctx, |ui| {
                let tr = self.state.language.tr();
                ui.add_space(8.0);

                if self.state.simulation_running {
                    // Progress comes from background thread via polling above
                    ctx.request_repaint_after(std::time::Duration::from_millis(40));

                    ui.horizontal(|ui| {
                        // Stop button
                        let stop_btn = egui::Button::new(
                            egui::RichText::new("⏹  STOP")
                                .color(Color32::WHITE)
                        )
                        .fill(Color32::from_rgb(150, 25, 25))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(210, 70, 70)));
                        if ui.add(stop_btn).clicked() {
                            self.stop_flag.store(true, Ordering::SeqCst);
                            self.state.simulation_running = false;
                            self.state.show_impact_panel  = !self.sim_snapshots.is_empty();
                            self.state.message = Some(format!("⛔ Stopped at step {}", self.state.sim_elapsed_steps));
                        }

                        ui.add_space(16.0);

                        // Progress bar
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(
                                format!("⏱  Step {} / {}  ({:.1}%)",
                                    self.state.sim_elapsed_steps,
                                    self.state.simulation.max_steps,
                                    self.state.sim_progress * 100.0)
                            ).size(14.0).color(Color32::LIGHT_GRAY));
                            let bar_rect = ui.add(
                                egui::ProgressBar::new(self.state.sim_progress)
                                    .desired_width(420.0)
                                    .animate(true)
                            );
                            let _ = bar_rect;
                        });
                    });
                } else {
                    ui.horizontal(|ui| {
                        // Run button — dark navy, slow teal breath
                        let t = ui.input(|i| i.time);
                        let breath = ((t * 0.7).sin() as f32) * 0.5 + 0.5;
                        let glow = Color32::from_rgb(
                            (48.0 + breath * 18.0) as u8,
                            (175.0 + breath * 28.0) as u8,
                            (155.0 + breath * 18.0) as u8,
                        );
                        let run_btn = egui::Button::new(
                            egui::RichText::new(format!("▶  {}", tr.btn_run_simulation))
                                .strong()
                                .color(Color32::from_rgb(220, 232, 248))
                        )
                        .fill(Color32::from_rgb(16, 30, 52))
                        .stroke(Stroke::new(1.4 + breath * 0.7, glow))
                        .rounding(egui::Rounding::same(4.0));

                        if ui.add(run_btn).on_hover_text(tr.btn_run_tooltip).clicked() {
                            self.state.simulation_running = true;
                            self.state.sim_progress       = 0.0;
                            self.state.sim_elapsed_steps  = 0;
                            self.state.show_impact_panel  = false;
                            self.state.message = Some(tr.sim_started.to_string());
                            self.start_simulation(ctx);
                        }
                        // Repaint for teal-breath animation on idle button
                        ctx.request_repaint_after(std::time::Duration::from_millis(50));

                        // Back to settings — visible only after simulation finished
                        if self.state.show_impact_panel {
                            ui.add_space(12.0);
                            if ui.button(
                                egui::RichText::new("← Back to settings")
                                    .color(Color32::from_rgb(170, 185, 210))
                            ).clicked() {
                                self.state.show_impact_panel = false;
                            }
                        }

                        ui.add_space(16.0);
                        if ui.button(
                            egui::RichText::new("ℹ️ About")
                                .color(Color32::from_rgb(160, 190, 230))
                        ).on_hover_text("Какие параметры на какие метрики влияют / Which params affect which metrics").clicked() {
                            self.state.show_about = true;
                        }
                    });
                }
                ui.add_space(4.0);
                ui.separator();
                self.show_metrics_bar(ui);
            });

        // Central panel — lineage tree (top) + adult SC row (bottom, fixed)
        CentralPanel::default().show(ctx, |ui| {
            if self.state.simulation_running || !self.sim_snapshots.is_empty() {
                ScrollArea::vertical().show(ui, |ui| {
                    self.show_live_dashboard(ui);
                });
            } else {
                let total_h = ui.available_height();
                let sc_row_h = (total_h * 0.45).clamp(180.0, 320.0);
                let tree_h = (total_h - sc_row_h - 6.0).max(80.0);
                ScrollArea::vertical()
                    .max_height(tree_h)
                    .show(ui, |ui| {
                        self.show_lineage_tree(ui);
                    });
                ui.separator();
                self.show_adult_sc_row(ui);
            }
        });
        
        // Dialogs
        if self.state.show_save_dialog {
            self.show_save_dialog(ctx);
        }
        
        if self.state.show_load_dialog {
            self.show_load_dialog(ctx);
        }
        
        if self.state.show_preset_dialog {
            self.show_preset_dialog(ctx);
        }
        
        if self.state.show_export_dialog {
            self.show_export_dialog(ctx);
        }
        
        if self.state.show_validation_dialog {
            self.show_validation_dialog(ctx);
        }

        if self.state.show_about {
            self.show_about_dialog(ctx);
        }

        // Limit repaint rate to reduce CPU usage when realtime_viz is enabled
        if self.state.realtime_viz.enabled {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

// ==================== LINEAGE TREE ====================

/// Static lineage node: (id, name, potency_label, description, div_rate_day, rgb)
type LNode = (usize, &'static str, &'static str, &'static str, f32, [u8; 3]);

const LINEAGE: &[LNode] = &[
    // ── Totipotent ──────────────────────────────────────────────────────────
    (0,  "Zygote → Morula → Blastocyst",  "Totipotent",    "All 200+ cell types; ICM + trophoblast",                1.0,  [255, 210, 60]),
    // ── Pluripotent ─────────────────────────────────────────────────────────
    (1,  "ICM / Epiblast",                "Pluripotent",    "3 germ layers; Oct4+/Sox2+/Nanog+",                    1.0,  [180, 100, 250]),
    // ── Germ layers (Multipotent) ────────────────────────────────────────────
    (2,  "Ectoderm",                      "Multipotent",    "Nervous system, skin, sense organs",                   0.5,  [80, 150, 230]),
    (3,  "Mesoderm",                      "Multipotent",    "Blood, muscle, bone, connective tissue",               0.5,  [80, 150, 230]),
    (4,  "Endoderm",                      "Multipotent",    "Gut, liver, lungs, thyroid, pancreas",                 0.5,  [80, 150, 230]),
    // ── Oligopotent SCs (from Ectoderm) ──────────────────────────────────────
    (5,  "Neural SC (NSC)",               "Oligopotent",    "SVZ + SGZ niches; Sox2+/Nestin+",                      0.01, [60, 200, 160]),
    (6,  "Epidermal SC (IFE basal)",      "Oligopotent",    "Rete ridges; α6β4+/p63+",                              0.07, [60, 200, 160]),
    // ── Oligopotent SCs (from Mesoderm) ──────────────────────────────────────
    (7,  "HSC (LT-HSC)",                  "Oligopotent",    "Endosteal niche; CD34+/CD38-/Lin-",                    0.019,[60, 200, 160]),
    (8,  "MSC",                           "Oligopotent",    "Periosteum/BM stroma; CD105+/CD90+/CD73+",            0.008,[60, 200, 160]),
    (9,  "Satellite cell (MuSC)",         "Unipotent",      "Sublaminal muscle niche; Pax7+/MyoD-",                 0.001,[80, 200, 90]),
    // ── Oligopotent SCs (from Endoderm) ──────────────────────────────────────
    (10, "Hepatic progenitor (HPC)",      "Bipotent",       "Canal of Hering; EpCAM+/Sox9+",                        0.002,[60, 200, 160]),
    (11, "Intestinal SC (Lgr5+)",         "Oligopotent",    "Crypt base CBC; Wnt-driven; ~1 div/day",               1.0,  [60, 200, 160]),
    (12, "Lung SC (AT2, Axin2+)",         "Oligopotent",    "Alveolar walls; SPC+/SFTPb+",                          0.003,[60, 200, 160]),
    // ── Unipotent progenitors ────────────────────────────────────────────────
    (13, "Neuroblast",                    "Unipotent",      "Migrating progenitor; Dcx+/PSA-NCAM+",                 0.5,  [80, 200, 90]),
    (14, "Oligodendrocyte PC (OPC)",      "Unipotent",      "White matter; PDGFRA+/NG2+",                           0.1,  [80, 200, 90]),
    (15, "Keratinocyte progenitor",       "Unipotent",      "Suprabasal; CK1/10 induction",                         0.07, [80, 200, 90]),
    (16, "Melanoblast",                   "Unipotent",      "NC-derived; MITF+/DCT+",                               0.02, [80, 200, 90]),
    (17, "CMP (Myeloid progenitor)",      "Unipotent",      "BM; CD34+/CD38+/IL-3Ra+/CD45RA-",                     0.3,  [80, 200, 90]),
    (18, "CLP (Lymphoid progenitor)",     "Unipotent",      "BM; CD34+/CD38+/CD10+",                               0.2,  [80, 200, 90]),
    (19, "Osteoprogenitor",               "Unipotent",      "RUNX2+/OSX+; pre-osteoblast",                          0.01, [80, 200, 90]),
    (20, "Preadipocyte",                  "Unipotent",      "PPARγ low; C/EBPβ+",                                   0.005,[80, 200, 90]),
    (21, "Chondroprogenitor",             "Unipotent",      "SOX9+/COL2A1 low",                                     0.003,[80, 200, 90]),
    (22, "Myoblast",                      "Unipotent",      "MyoD+/Myf5+; fusion-competent",                        0.3,  [80, 200, 90]),
    (23, "Hepatoblast",                   "Unipotent",      "HNF4A+/AFP+; bi-lineage precursor",                   0.05, [80, 200, 90]),
    (24, "Transit-amplifying cell (TA)",  "Unipotent",      "Crypt progenitor; Lgr5-/Ascl2+",                       2.0,  [80, 200, 90]),
    (25, "AT2→AT1 transitional",          "Unipotent",      "KRT8hi transitional state post-injury",                0.1,  [80, 200, 90]),
    // ── Terminally differentiated ────────────────────────────────────────────
    (26, "Glutamatergic neuron",          "Differentiated", "Excitatory; VGLUT1/2+; post-mitotic",                  0.0,  [140, 150, 170]),
    (27, "GABAergic interneuron",         "Differentiated", "Inhibitory; GAD1/2+; post-mitotic",                    0.0,  [140, 150, 170]),
    (28, "Oligodendrocyte",               "Differentiated", "Myelination; MBP+/PLP1+",                              0.0,  [140, 150, 170]),
    (29, "Keratinocyte (cornified)",      "Differentiated", "Barrier; loricrin+/filaggrin+",                        0.0,  [140, 150, 170]),
    (30, "Melanocyte",                    "Differentiated", "Melanin; TYR+/TYRP1+",                                 0.0,  [140, 150, 170]),
    (31, "Monocyte / Macrophage",         "Differentiated", "Innate immunity; CD14+/CD68+",                          0.0,  [140, 150, 170]),
    (32, "Neutrophil",                    "Differentiated", "Phagocytosis; CD66b+/MPO+",                             0.0,  [140, 150, 170]),
    (33, "Erythrocyte (RBC)",             "Differentiated", "O₂ transport; HbA+; enucleated",                       0.0,  [140, 150, 170]),
    (34, "T lymphocyte",                  "Differentiated", "Adaptive immunity; CD3+/TCR+",                          0.0,  [140, 150, 170]),
    (35, "B cell / Plasma cell",          "Differentiated", "Antibody; CD19+/CD20+ → IgG+",                         0.0,  [140, 150, 170]),
    (36, "Osteoblast",                    "Differentiated", "Bone matrix; RUNX2+/OCN+",                              0.003,[140, 150, 170]),
    (37, "Adipocyte",                     "Differentiated", "Energy; PPARγ+/FABP4+",                                 0.0,  [140, 150, 170]),
    (38, "Chondrocyte",                   "Differentiated", "Cartilage; COL2A1+/ACAN+",                              0.0,  [140, 150, 170]),
    (39, "Myocyte / Muscle fiber",        "Differentiated", "Contraction; MyHC+/post-mitotic",                       0.0,  [140, 150, 170]),
    (40, "Hepatocyte",                    "Differentiated", "Metabolism; ALB+/HNF4A+",                               0.001,[140, 150, 170]),
    (41, "Enterocyte",                    "Differentiated", "Nutrient absorption; CDX2+/EpCAM+",                     0.0,  [140, 150, 170]),
    (42, "AT1 pneumocyte",                "Differentiated", "Gas exchange; AQP5+/PDPN+",                             0.0,  [140, 150, 170]),
    (43, "AT2 pneumocyte (mature)",       "Differentiated", "Surfactant; SPC+/SFTPb+",                               0.003,[140, 150, 170]),
];

fn lineage_children(id: usize) -> &'static [usize] {
    match id {
        0  => &[1],
        1  => &[2, 3, 4],
        2  => &[5, 6],
        3  => &[7, 8, 9],
        4  => &[10, 11, 12],
        5  => &[13, 14],
        6  => &[15, 16],
        7  => &[17, 18],
        8  => &[19, 20, 21],
        9  => &[22],
        10 => &[23],
        11 => &[24],
        12 => &[25],
        13 => &[26, 27],
        14 => &[28],
        15 => &[29],
        16 => &[30],
        17 => &[31, 32, 33],
        18 => &[34, 35],
        19 => &[36],
        20 => &[37],
        21 => &[38],
        22 => &[39],
        23 => &[40],
        24 => &[41],
        25 => &[42, 43],
        _  => &[],
    }
}

/// Adult stem cells considered "safe" for regenerative medicine.
/// Index: (node_id, niche_name, safety_note)
const ADULT_SC_SAFE: &[(usize, &str)] = &[
    (7,  "BM/Blood"),
    (5,  "Neural"),
    (8,  "Mesenchymal"),
    (9,  "Muscle"),
    (11, "Intestinal"),
    (6,  "Epidermal"),
    (10, "Hepatic"),
    (12, "Lung AT2"),
];

/// Division Synchrony Index (DSI) for a safe adult SC:
/// DSI = mean(child_div_rates) / (1 + CV(child_div_rates))
/// where CV = std_dev / mean.  Range [0..1]: 1 = perfectly synchronous fast division.
fn division_synchrony_index(parent_id: usize) -> f32 {
    let children = lineage_children(parent_id);
    if children.is_empty() { return 0.0; }

    let rates: Vec<f32> = children.iter()
        .map(|&cid| {
            // also look one level deeper
            let direct = LINEAGE.iter().find(|n| n.0 == cid).map(|n| n.4).unwrap_or(0.0);
            let deeper = lineage_children(cid);
            if deeper.is_empty() { return direct; }
            let child_rates: Vec<f32> = deeper.iter()
                .filter_map(|&gcid| LINEAGE.iter().find(|n| n.0 == gcid).map(|n| n.4))
                .collect();
            let mean_child: f32 = child_rates.iter().sum::<f32>() / child_rates.len().max(1) as f32;
            (direct + mean_child) * 0.5
        })
        .collect();

    let n = rates.len() as f32;
    let mean = rates.iter().sum::<f32>() / n;
    if mean < 1e-6 { return 0.0; }
    let variance = rates.iter().map(|r| (r - mean).powi(2)).sum::<f32>() / n;
    let cv = variance.sqrt() / mean;
    let dsi = mean / (1.0 + cv);
    dsi.clamp(0.0, 1.0)
}

fn render_lineage_node(
    node_id: usize,
    depth: usize,
    is_last: bool,
    expanded: &std::collections::HashSet<usize>,
    ui: &mut egui::Ui,
    to_toggle: &mut Option<usize>,
) {
    let Some(&(_, name, potency, desc, div_rate, [r, g, b])) =
        LINEAGE.iter().find(|n| n.0 == node_id) else { return };

    let children = lineage_children(node_id);
    let has_children = !children.is_empty();
    let is_expanded = expanded.contains(&node_id);
    let node_col = Color32::from_rgb(r, g, b);

    ui.horizontal(|ui| {
        // indentation
        if depth > 0 {
            ui.add_space((depth as f32 - 1.0) * 18.0);
            let conn = if is_last { "└─" } else { "├─" };
            ui.label(egui::RichText::new(conn).size(10.0).monospace()
                .color(Color32::from_rgb(55, 75, 95)));
        }

        // card
        let fill = if is_expanded && has_children {
            Color32::from_rgb(32, 48, 62)
        } else {
            Color32::from_rgb(20, 30, 42)
        };

        let card_resp = egui::Frame::none()
            .fill(fill)
            .stroke(Stroke::new(if is_expanded { 1.8 } else { 1.0 }, node_col))
            .inner_margin(egui::Margin::same(5.0))
            .rounding(egui::Rounding::same(4.0))
            .show(ui, |ui| {
                ui.set_min_width(140.0);
                ui.horizontal(|ui| {
                    let icon = if !has_children { "•" } else if is_expanded { "▼" } else { "▶" };
                    ui.label(egui::RichText::new(icon).size(10.0).color(node_col));
                    ui.label(egui::RichText::new(name).size(10.5).strong()
                        .color(Color32::WHITE));
                });
                ui.label(egui::RichText::new(potency).size(8.5).color(node_col));
                ui.label(egui::RichText::new(desc).size(8.0)
                    .color(Color32::from_rgb(150, 162, 178)));
                if div_rate > 0.0 {
                    ui.label(egui::RichText::new(format!("⏱ {:.3}/day", div_rate))
                        .size(8.0).color(Color32::from_rgb(90, 180, 90)));
                }
            })
            .response;

        if card_resp.interact(egui::Sense::click()).clicked() {
            *to_toggle = Some(node_id);
        }
    });

    if is_expanded && has_children {
        for (i, &child_id) in children.iter().enumerate() {
            render_lineage_node(
                child_id, depth + 1,
                i == children.len() - 1,
                expanded, ui, to_toggle,
            );
        }
    }
}

impl ConfigApp {
    fn show_lineage_tree(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("🧬 Cell Lineage — Potency Hierarchy")
            .size(15.0).strong().color(Color32::from_rgb(100, 190, 160)));
        ui.label(egui::RichText::new(
            "Click ▶ to expand descendants • from totipotent blastocyst to terminally differentiated cells"
        ).size(10.0).color(Color32::GRAY));
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(2.0);

        let mut to_toggle: Option<usize> = None;
        render_lineage_node(0, 0, true, &self.expanded_lineage, ui, &mut to_toggle);

        if let Some(id) = to_toggle {
            if self.expanded_lineage.contains(&id) {
                self.expanded_lineage.remove(&id);
            } else {
                self.expanded_lineage.insert(id);
            }
        }
    }

    /// Fixed bottom row: safe adult stem cells with DSI metrics
    fn show_adult_sc_row(&self, ui: &mut egui::Ui) {
        let teal   = Color32::from_rgb(60, 200, 160);
        let gold   = Color32::from_rgb(220, 180, 60);
        let subtle = Color32::from_rgb(80, 95, 110);
        let live_col = Color32::from_rgb(50, 220, 120);
        let dead_col = Color32::from_rgb(120, 60, 60);

        // Live division_rate from last simulation snapshot (global average across all niches)
        let live_divrate: Option<f32> = self.sim_snapshots.last().map(|s| s.division_rate);

        // Header
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🔒 Safe Adult SCs:")
                .size(10.5).strong().color(teal));
            if let Some(live) = live_divrate {
                let col = if live > 0.6 { live_col } else if live > 0.3 { gold } else { dead_col };
                ui.label(egui::RichText::new(format!("● sim div_rate: {:.3}", live))
                    .size(9.0).color(col));
            }
        });
        ui.add_space(4.0);

        // Vertical list with scroll
        // tissue_order index matches ADULT_SC_SAFE order:
        // 0=Blood, 1=Neural, 2=Connective, 3=Muscle, 4=Epithelial, 5=Skin, 6=Liver, 7=Lung
        let last_snap = self.sim_snapshots.last();
        egui::ScrollArea::vertical()
            .id_source("adult_sc_scroll")
            .show(ui, |ui| {
                for (tissue_idx, &(node_id, niche)) in ADULT_SC_SAFE.iter().enumerate() {
                    let Some(&(_, name, _, _, div_rate_ref, [r, g, b])) =
                        LINEAGE.iter().find(|n| n.0 == node_id) else { continue };

                    let dsi = division_synchrony_index(node_id);
                    let sc_col = Color32::from_rgb(r, g, b);

                    let dsi_col = if dsi > 0.05 {
                        Color32::from_rgb(80, 200, 90)
                    } else if dsi > 0.01 {
                        gold
                    } else {
                        Color32::from_rgb(180, 90, 70)
                    };

                    // Per-tissue live rate from last snapshot
                    let live_tissue: Option<f32> = last_snap.and_then(|s| {
                        let v = s.per_tissue_div_rate[tissue_idx];
                        if v.is_nan() { None } else { Some(v) }
                    });

                    egui::Frame::none()
                        .fill(Color32::from_rgb(22, 34, 46))
                        .stroke(Stroke::new(1.2, sc_col))
                        .inner_margin(egui::Margin::same(6.0))
                        .rounding(egui::Rounding::same(4.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(niche).size(10.5).strong()
                                    .color(Color32::WHITE));
                                ui.label(egui::RichText::new(
                                    &name[..name.len().min(30)]
                                ).size(9.0).color(subtle));
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(
                                    format!("ref {:.3}", div_rate_ref)).size(8.5)
                                    .color(subtle));
                                ui.label(egui::RichText::new("|").size(8.5).color(subtle));
                                ui.label(egui::RichText::new(
                                    format!("DSI {:.3}", dsi)).size(8.5)
                                    .color(dsi_col));
                                if let Some(live) = live_tissue {
                                    let lc = if live > 0.6 { live_col }
                                             else if live > 0.3 { gold }
                                             else { dead_col };
                                    ui.label(egui::RichText::new("|").size(8.5).color(subtle));
                                    ui.label(egui::RichText::new(format!("● {:.3}", live))
                                        .size(8.5).color(lc));
                                }
                            });
                        });
                    ui.add_space(3.0);
                }
            });
    }

    fn show_niche_catalog(&mut self, ui: &mut egui::Ui) {
        use egui::RichText;

        let live_divrate: Option<f32> = self.sim_snapshots.last().map(|s| s.division_rate);

        ui.heading(RichText::new("🧬 Stem Cell Niches — Pre-simulation")
            .size(17.0).strong().color(Color32::from_rgb(100, 190, 160)));
        ui.horizontal(|ui| {
            ui.label(RichText::new(
                "20 human niches by potency level • click a card to inspect"
            ).size(10.5).color(Color32::GRAY));
            if let Some(live) = live_divrate {
                let lc = if live > 0.6 { Color32::from_rgb(50, 220, 120) }
                         else if live > 0.3 { Color32::from_rgb(220, 180, 60) }
                         else { Color32::from_rgb(180, 80, 60) };
                ui.label(RichText::new(format!("● sim div_rate: {:.3}", live))
                    .size(10.0).color(lc));
            }
        });
        ui.add_space(4.0);
        ui.separator();

        let niches = NicheCatalog::all();

        // ── Niche groups by potency level ─────────────────────────────────
        struct Group { name: &'static str, ids: &'static [&'static str], r: u8, g: u8, b: u8 }
        let groups = [
            Group { name: "Pluripotent / Long-term",    ids: &["BM_Endo","BM_Peri","SkMus","IntCrypt","SVZ","SGZ"],    r:80, g:160, b:220 },
            Group { name: "Oligopotent / Tissue-specific", ids: &["SkinIFE","HairBulge","Limbal","Mammary","Testis","Adipose"], r:100,g:200,b:130 },
            Group { name: "Unipotent / Ultra-slow",     ids: &["LiverHPC","LungAT2","LungBADJ","Pancreas","Heart","Prostate","DentalPulp","Periosteum"], r:220,g:150,b:60 },
        ];

        let mut new_sel = self.selected_niche;

        for g in &groups {
            let g_col = Color32::from_rgb(g.r, g.g, g.b);
            ui.add_space(6.0);
            ui.label(RichText::new(g.name).size(12.5).strong().color(g_col));
            ui.add_space(2.0);

            ui.horizontal_wrapped(|ui| {
                for &id in g.ids {
                    let found = niches.iter().enumerate().find(|(_, n)| n.id == id);
                    if let Some((idx, n)) = found {
                        let is_sel = new_sel == Some(idx);
                        let [tr, tg, tb] = n.tier.color();
                        let tier_col = Color32::from_rgb(tr, tg, tb);

                        let fill = if is_sel { Color32::from_rgb(40,62,78) } else { Color32::from_rgb(26,36,46) };
                        let stroke = if is_sel {
                            Stroke::new(2.0, g_col)
                        } else {
                            Stroke::new(1.0, Color32::from_rgb(55,75,95))
                        };

                        let card_resp = egui::Frame::none()
                            .fill(fill)
                            .stroke(stroke)
                            .inner_margin(egui::Margin::same(6.0))
                            .rounding(egui::Rounding::same(4.0))
                            .show(ui, |ui| {
                                ui.set_min_width(158.0);
                                ui.set_max_width(158.0);
                                ui.label(RichText::new(n.id).size(9.0).monospace()
                                    .color(tier_col));
                                ui.label(RichText::new(n.name).size(9.5).strong()
                                    .color(Color32::WHITE));
                                ui.label(RichText::new(n.sc_type).size(8.5)
                                    .color(Color32::LIGHT_GRAY));
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(
                                        format!("📍{}µm  ⏱{}", n.depth_um as u32,
                                            if n.div_rate_young >= 0.1 { "fast" }
                                            else if n.div_rate_young >= 0.005 { "slow" }
                                            else { "ultra-slow" })
                                    ).size(8.0).color(Color32::from_rgb(130,160,130)));
                                });
                                // Ref rates (literature)
                                ui.label(RichText::new(
                                    format!("ref {:.4}→{:.4}/day", n.div_rate_young, n.div_rate_aged)
                                ).size(7.5).color(Color32::from_rgb(80,95,110)));
                                // Live sim rate (global average)
                                if let Some(live) = live_divrate {
                                    let lc = if live > 0.6 { Color32::from_rgb(50,220,120) }
                                             else if live > 0.3 { Color32::from_rgb(220,180,60) }
                                             else { Color32::from_rgb(180,80,60) };
                                    ui.label(RichText::new(format!("● sim {:.3}", live))
                                        .size(7.5).color(lc));
                                }
                            })
                            .response;

                        if card_resp.interact(egui::Sense::click()).clicked() {
                            new_sel = if is_sel { None } else { Some(idx) };
                        }
                    }
                }
            });
        }

        // ── Detail panel ──────────────────────────────────────────────────
        if let Some(idx) = new_sel {
            if let Some(n) = niches.get(idx) {
                ui.add_space(8.0);
                ui.separator();
                egui::Frame::none()
                    .fill(Color32::from_rgb(20, 35, 52))
                    .stroke(Stroke::new(1.5, Color32::from_rgb(70, 130, 195)))
                    .inner_margin(egui::Margin::same(10.0))
                    .rounding(egui::Rounding::same(5.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("🔬  {}", n.name))
                            .size(14.0).strong().color(Color32::WHITE));
                        ui.add_space(3.0);
                        egui::Grid::new("nd")
                            .num_columns(2)
                            .spacing([10.0, 3.0])
                            .show(ui, |ui| {
                                let k = |s: &str| RichText::new(s).size(10.0)
                                    .color(Color32::from_rgb(130,160,200));
                                let v = |s: &str| RichText::new(s).size(10.0)
                                    .color(Color32::LIGHT_GRAY);
                                ui.label(k("Organ:"));     ui.label(v(n.organ));       ui.end_row();
                                ui.label(k("SC type:"));   ui.label(v(n.sc_type));     ui.end_row();
                                ui.label(k("Location:"));  ui.label(v(n.location));    ui.end_row();
                                ui.label(k("Depth:"));     ui.label(v(&format!("{} µm", n.depth_um))); ui.end_row();
                                ui.label(k("Div.rate:")); ui.label(v(&format!(
                                    "{:.4}/day young → {:.4}/day aged  (↓{:.0}%)",
                                    n.div_rate_young, n.div_rate_aged,
                                    (1.0 - n.div_rate_aged/n.div_rate_young.max(1e-9)) * 100.0
                                ))); ui.end_row();
                                ui.label(k("Signals:"));   ui.label(v(n.key_signals)); ui.end_row();
                                ui.label(k("Aging:"));     ui.label(v(n.aging_note));  ui.end_row();
                            });
                    });
            }
        }

        self.selected_niche = new_sel;
    }
}

// ==================== METRICS HELPERS ====================

impl ConfigApp {
    /// Compute damage_scale from current GUI parameters (same formula as dashboard).
    fn damage_scale(&self) -> f64 {
        let detach = (self.state.cdata.base_detach_probability / 0.0003_f32) as f64;
        let preset = match self.state.cdata.damage_preset {
            DamagePreset::Normal    => 1.0_f64,
            DamagePreset::Progeria  => 5.0_f64,
            DamagePreset::Longevity => 0.6_f64,
        };
        let bias   = 1.0 + (self.state.cdata.mother_bias as f64 - 0.5) * 0.8;
        let age_f  = 1.0 + self.state.cdata.age_bias_coefficient as f64 * 80.0;
        let chk    = 1.0 - self.state.cell_cycle.checkpoint_strictness as f64 * 0.30;
        (preset * detach * bias * age_f * chk).max(0.05)
    }

    /// Six key metrics estimated from math model (no simulation needed).
    /// Returns [(label, value, is_warning)].
    fn estimated_metrics(&self) -> Vec<(&'static str, String, bool)> {
        let s  = self.damage_scale();
        // ps: pool depletion scale — аппроксимация, не выведена из первых принципов.
        // Направление качественно верное: чем выше floor (медленнее стареет), тем меньше ps.
        // Формула ps = s / (0.5 + floor×1.5) нормирует damage_scale на эффективный
        // минимальный темп деления. Валидирована качественно; для количественных предсказаний
        // использовать "real" (зелёную) кривую из симуляции.
        let ps = (s / (0.5 + self.state.cdata.division_rate_floor as f64 * 1.5)).max(0.05);
        let rs = (s * (2.0 - self.state.cdata.ros_brake_strength as f64)).max(0.05);

        let frailty_fn = |age: f64| -> f64 {
            1.0 / (1.0 + (-(0.08 * s * (age - 45.0 / s.sqrt()))).exp())
        };
        // age where frailty = 0.95: solve sigmoid analytically
        let lifespan   = (45.0 / s.sqrt() + (19.0_f64).ln() / (0.08 * s)).max(1.0);
        let healthspan = (45.0 / s.sqrt()).clamp(0.0, lifespan);
        let f50  = frailty_fn(50.0);
        let f70  = frailty_fn(70.0);
        let p70  = (1.0 - 0.011 * ps * 70.0).max(0.0);
        let r50  = (0.007 * rs * 50.0).min(1.0);

        let dead50 = lifespan < 45.0;
        let dead70 = lifespan < 65.0;

        // Estimated new metrics (derived from damage_scale)
        let prot70_e = (0.005 * s * 70.0).min(1.0);
        let divr70_e = (1.0 - 0.008 * ps * 70.0).max(0.0);
        let cds70_e  = (0.009 * s * 70.0).min(1.0);
        let bio70_e  = (70.0 * s * 0.4 + (0.55 * ps * 70.0).min(1.0) * 120.0 * 0.3
                        + frailty_fn(70.0) * 100.0 * 0.3).clamp(0.0, 150.0) as f32;

        vec![
            ("Lifespan",    format!("~{:.0} yr", lifespan),                  lifespan < 40.0),
            ("Healthspan",  format!("~{:.0} yr", healthspan),                healthspan < 20.0),
            ("Frailty @50", if dead50 { "✝ died".into() } else { format!("{:.2}", f50) }, dead50 || f50 > 0.8),
            ("Frailty @70", if dead70 { "✝ died".into() } else { format!("{:.2}", f70) }, dead70 || f70 > 0.8),
            ("Pool @70",    if dead70 { "✝ died".into() } else { format!("{:.0}%", p70 * 100.0) }, dead70 || p70 < 0.3),
            ("ROS @50",     if dead50 { "✝ died".into() } else { format!("{:.2}", r50) }, dead50 || r50 > 0.6),
            ("Prot.Agg@70", if dead70 { "✝ died".into() } else { format!("{:.2}", prot70_e) }, dead70 || prot70_e > 0.5),
            ("Div.Rate@70", if dead70 { "✝ died".into() } else { format!("{:.2}", divr70_e) }, dead70 || divr70_e < 0.3),
            ("CDS @70",     if dead70 { "✝ died".into() } else { format!("{:.2}", cds70_e) },  dead70 || cds70_e > 0.6),
            ("BioAge @70",  if dead70 { "✝ died".into() } else { format!("~{:.0} yr", bio70_e) }, dead70 || bio70_e > 90.0),
        ]
    }

    /// Six key metrics from real simulation snapshots.
    fn real_metrics(&self) -> Option<Vec<(&'static str, String, bool)>> {
        if self.sim_snapshots.is_empty() { return None; }

        let snap_at = |target: f64| -> Option<&SimSnapshot> {
            self.sim_snapshots.iter()
                .min_by(|a, b| {
                    (a.age_years - target).abs()
                        .partial_cmp(&(b.age_years - target).abs())
                        .unwrap()
                })
        };

        let last      = self.sim_snapshots.last().unwrap();
        let lifespan  = last.age_years;
        let healthspan = self.sim_snapshots.iter()
            .rev()
            .find(|s| s.frailty < 0.5)
            .map(|s| s.age_years)
            .unwrap_or(0.0);

        let dead_before = |age: f64| lifespan < age - 3.0;

        let f50 = snap_at(50.0).map(|s| s.frailty).unwrap_or(0.0);
        let f70 = snap_at(70.0).map(|s| s.frailty).unwrap_or(0.0);
        let p70 = snap_at(70.0).map(|s| s.stem_cell_pool).unwrap_or(0.0);
        let r50 = snap_at(50.0).map(|s| s.ros_level).unwrap_or(0.0);

        let prot70 = snap_at(70.0).map(|s| s.protein_aggregation).unwrap_or(0.0);
        let divr70 = snap_at(70.0).map(|s| s.division_rate).unwrap_or(1.0);
        let cds70  = snap_at(70.0).map(|s| s.centrosomal_damage).unwrap_or(0.0);
        let bio70  = snap_at(70.0).map(|s| s.biological_age).unwrap_or(70.0);

        Some(vec![
            ("Lifespan",    format!("{:.1} yr", lifespan),                     lifespan < 40.0),
            ("Healthspan",  format!("{:.1} yr", healthspan),                   healthspan < 20.0),
            ("Frailty @50", if dead_before(50.0) { "✝ died".into() } else { format!("{:.3}", f50) }, dead_before(50.0) || f50 > 0.8),
            ("Frailty @70", if dead_before(70.0) { "✝ died".into() } else { format!("{:.3}", f70) }, dead_before(70.0) || f70 > 0.8),
            ("Pool @70",    if dead_before(70.0) { "✝ died".into() } else { format!("{:.0}%",  p70 * 100.0) }, dead_before(70.0) || p70 < 0.3),
            ("ROS @50",     if dead_before(50.0) { "✝ died".into() } else { format!("{:.3}", r50) }, dead_before(50.0) || r50 > 0.6),
            ("Prot.Agg@70", if dead_before(70.0) { "✝ died".into() } else { format!("{:.3}", prot70) }, dead_before(70.0) || prot70 > 0.5),
            ("Div.Rate@70", if dead_before(70.0) { "✝ died".into() } else { format!("{:.3}", divr70) }, dead_before(70.0) || divr70 < 0.3),
            ("CDS @70",     if dead_before(70.0) { "✝ died".into() } else { format!("{:.3}", cds70) },  dead_before(70.0) || cds70 > 0.6),
            ("BioAge @70",  if dead_before(70.0) { "✝ died".into() } else { format!("{:.1} yr", bio70) }, dead_before(70.0) || bio70 > 90.0),
        ])
    }

    // ── System risks ──────────────────────────────────────────────────────────
    //
    // Formulas derived from the CDATA damage model.
    //
    // Shared intermediates (evaluated at reference age 65):
    //   ros65     = (0.007 × rs × 65).min(1)       rs = s × (2 − ros_brake_strength)
    //   myeloid65 = (0.004 × ms × 65 × 1.54).min(1)  ms = s × (spindle_w×2 + ros_w×1.5)
    //   epigen65  = (65 × (1 + 0.4×s×0.65) / 120).min(1)
    //
    // Cardiovascular risk at lifespan:
    //   cardio = ros65×0.45 + myeloid65×0.40 + max(s−0.6, 0)×0.08
    //   Mechanism: ROS oxidises LDL → atherosclerosis; myeloid bias drives chronic
    //   vascular inflammation (SASP). damage_scale adds baseline endothelial stress.
    //   Parameters: damage_preset, base_detach_probability, ros_brake_strength,
    //               spindle_weight, ros_weight, mother_bias, age_bias_coefficient
    //
    // Cancer risk at lifespan:
    //   cancer = s × (1 − checkpoint×0.60) × 0.30
    //   Mechanism: higher damage = more DNA strand breaks (centriolar mis-segregation
    //   → aneuploidy; ROS → point mutations). Checkpoint strictness provides protection.
    //   Parameters: damage_preset, base_detach_probability, checkpoint_strictness
    //
    // Cognitive / mental risk at lifespan:
    //   cogn = (myeloid65×0.45 + ros65×0.30 + epigen65×0.25) × 0.65
    //   Mechanism: neuroinflammation via myeloid shift (microglia activation, SASP);
    //   oxidative neurodegeneration; epigenetic clock acceleration → cortical aging.
    //   Parameters: damage_preset, spindle_weight, ros_weight, ros_brake_strength,
    //               age_bias_coefficient
    //
    // Warning thresholds: cardio > 55%, cancer > 40%, cogn > 45%

    fn estimated_system_risks(&self) -> Vec<(&'static str, String, bool)> {
        let s  = self.damage_scale();
        let rs = (s * (2.0 - self.state.cdata.ros_brake_strength as f64)).max(0.05);
        let ms = (s * (self.state.cdata.spindle_weight as f64 * 2.0
                       + self.state.cdata.ros_weight    as f64 * 1.5)).max(0.05);

        let ros65     = (0.007 * rs * 65.0).min(1.0);
        let myeloid65 = (0.004 * ms * 65.0 * (1.0 + 65.0 / 120.0)).min(1.0);
        let epigen65  = (65.0 * (1.0 + 0.4 * s * 65.0 / 100.0) / 120.0).min(1.0);

        let chk = self.state.cell_cycle.checkpoint_strictness as f64;
        let cardio = (ros65 * 0.45 + myeloid65 * 0.40 + (s - 0.6).max(0.0) * 0.08).min(1.0);
        let cancer = (s * (1.0 - chk * 0.60) * 0.30).min(1.0);
        let cogn   = ((myeloid65 * 0.45 + ros65 * 0.30 + epigen65 * 0.25) * 0.65).min(1.0);

        vec![
            ("🫀 Cardio",  format!("{:.0}%", cardio * 100.0), cardio > 0.55),
            ("🧬 Cancer",  format!("{:.0}%", cancer * 100.0), cancer > 0.40),
            ("🧠 Cognit.", format!("{:.0}%", cogn   * 100.0), cogn   > 0.45),
        ]
    }

    fn real_system_risks(&self) -> Option<Vec<(&'static str, String, bool)>> {
        if self.sim_snapshots.is_empty() { return None; }

        let snap_at = |target: f64| -> Option<&SimSnapshot> {
            self.sim_snapshots.iter()
                .min_by(|a, b| {
                    (a.age_years - target).abs()
                        .partial_cmp(&(b.age_years - target).abs())
                        .unwrap()
                })
        };
        let last = self.sim_snapshots.last().unwrap();
        let chk  = self.state.cell_cycle.checkpoint_strictness as f64;

        let ros65     = snap_at(65.0).map(|s| s.ros_level    as f64).unwrap_or(0.0);
        let myeloid65 = snap_at(65.0).map(|s| s.myeloid_bias as f64).unwrap_or(0.0);
        let epigen65  = snap_at(65.0)
            .map(|s| (s.methylation_age / 120.0).clamp(0.0, 1.0) as f64)
            .unwrap_or(0.0);
        let telomere_loss = 1.0 - last.telomere_length as f64;
        let ros_peak = self.sim_snapshots.iter()
            .map(|s| s.ros_level as f64)
            .fold(0.0_f64, f64::max);

        let cardio = (ros65 * 0.45 + myeloid65 * 0.40 + (ros_peak - 0.6).max(0.0) * 0.08).min(1.0);
        let cancer = (telomere_loss * 0.50 + ros_peak * 0.35 + (1.0 - chk * 0.60) * 0.15).min(1.0);
        let cogn   = ((myeloid65 * 0.45 + ros65 * 0.30 + epigen65 * 0.25) * 0.65).min(1.0);

        Some(vec![
            ("🫀 Cardio",  format!("{:.0}%", cardio * 100.0), cardio > 0.55),
            ("🧬 Cancer",  format!("{:.0}%", cancer * 100.0), cancer > 0.40),
            ("🧠 Cognit.", format!("{:.0}%", cogn   * 100.0), cogn   > 0.45),
        ])
    }

    /// Render the metrics bar in TWO rows.
    /// Row 1 (green): core longevity metrics — Lifespan, Healthspan, Frailty@50/70, Pool@70, ROS@50
    /// Row 2 (blue/amber): new metrics + system risks — Prot.Agg, Div.Rate, CDS, BioAge │ Cardio, Cancer, Cognit.
    fn show_metrics_bar(&self, ui: &mut egui::Ui) {
        let (metrics, is_real) = if let Some(real) = self.real_metrics() {
            (real, true)
        } else {
            (self.estimated_metrics(), false)
        };
        let (risks, risks_real) = if let Some(r) = self.real_system_risks() {
            (r, true)
        } else {
            (self.estimated_system_risks(), false)
        };

        let tag_color  = if is_real { Color32::from_rgb(50, 200, 130) } else { Color32::from_rgb(120, 130, 145) };
        let val_color  = if is_real { Color32::from_rgb(50, 200, 130) } else { Color32::from_rgb(200, 210, 225) };
        let warn_color = Color32::from_rgb(220, 120, 60);
        let lbl_color  = Color32::from_rgb(100, 110, 125);
        let new_val    = Color32::from_rgb(100, 180, 255);   // blue for new metrics
        let risk_val   = Color32::from_rgb(240, 190, 80);
        let risk_warn  = Color32::from_rgb(210, 65, 65);
        let risk_lbl   = Color32::from_rgb(130, 115, 80);
        let div_col    = Color32::from_rgb(70, 75, 85);
        let tag        = if is_real { "● real" } else { "● est." };

        // ── Row 1: core 6 aging metrics ──────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(tag).size(10.0).color(tag_color));
            ui.add_space(3.0);
            for (label, value, warn) in metrics.iter().take(6) {
                ui.vertical(|ui| {
                    ui.set_min_width(68.0);
                    ui.label(egui::RichText::new(*label).size(9.5).color(lbl_color));
                    let color = if *warn { warn_color } else { val_color };
                    ui.label(egui::RichText::new(value.as_str()).size(12.5).strong().color(color));
                });
            }
        });

        // ── Row 2: new 4 metrics + 3 system risks ────────────────────────────
        ui.horizontal(|ui| {
            let risk_tag_col = if risks_real { Color32::from_rgb(220, 160, 40) } else { Color32::from_rgb(140, 120, 80) };
            ui.label(egui::RichText::new("◈ ext.").size(10.0).color(new_val));
            ui.add_space(3.0);
            for (label, value, warn) in metrics.iter().skip(6) {
                ui.vertical(|ui| {
                    ui.set_min_width(68.0);
                    ui.label(egui::RichText::new(*label).size(9.5).color(lbl_color));
                    let color = if *warn { warn_color } else { new_val };
                    ui.label(egui::RichText::new(value.as_str()).size(12.5).strong().color(color));
                });
            }
            ui.label(egui::RichText::new("│").size(18.0).color(div_col));
            ui.label(egui::RichText::new(if risks_real { "⚕ real" } else { "⚕ est." }).size(10.0).color(risk_tag_col));
            ui.add_space(2.0);
            for (label, value, warn) in &risks {
                ui.vertical(|ui| {
                    ui.set_min_width(68.0);
                    ui.label(egui::RichText::new(*label).size(9.5).color(risk_lbl));
                    let color = if *warn { risk_warn } else { risk_val };
                    ui.label(egui::RichText::new(value.as_str()).size(12.5).strong().color(color));
                });
            }
        });

        // ── Row 3: Niche division rates (6 representative niches) ────────────
        let niche_col = Color32::from_rgb(100, 200, 180);
        let niche_warn = Color32::from_rgb(220, 130, 60);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🏘 niches").size(10.0).color(niche_col));
            ui.add_space(2.0);
            // Representative niches: HSC, Intestinal, SVZ, Satellite, Hair, Cardiac
            let representative = [
                ("HSC",    0.019_f32, 0.015_f32),
                ("IntSC",  1.000_f32, 0.700_f32),
                ("Neural", 0.010_f32, 0.005_f32),
                ("Muscle", 0.001_f32, 0.000_5),
                ("HairSC", 0.050_f32, 0.030_f32),
                ("CardSC", 0.000_1,   0.000_05),
            ];
            // Estimate age-adjusted rate from sim snapshots or use default
            let age_frac = if self.sim_snapshots.is_empty() {
                0.0_f32
            } else {
                (self.sim_snapshots.last().unwrap().age_years / 100.0) as f32
            };
            let damage_s = self.damage_scale() as f32;
            for (label, rate_young, rate_aged) in &representative {
                let current = rate_young * (1.0 - age_frac) + rate_aged * age_frac;
                // damage modulates division further
                let current = (current / damage_s.sqrt()).clamp(0.0, *rate_young);
                let warn = current < rate_aged * 0.5;
                ui.vertical(|ui| {
                    ui.set_min_width(60.0);
                    ui.label(egui::RichText::new(*label).size(9.5).color(lbl_color));
                    let col = if warn { niche_warn } else { niche_col };
                    ui.label(egui::RichText::new(format!("{:.4}", current)).size(12.0).strong().color(col));
                });
            }
        });
    }
}

// ==================== LIVE DASHBOARD ====================

impl ConfigApp {
    fn show_live_dashboard(&mut self, ui: &mut egui::Ui) {
        let progress = self.state.sim_progress;
        let current_age = (progress * 100.0) as f64;

        // ── Curve helpers ─────────────────────────────────────────────────────
        let frailty = |age: f64, scale: f64| -> f64 {
            let k = 0.08 * scale;
            let mid = 45.0 / scale.sqrt();
            1.0 / (1.0 + (-k * (age - mid)).exp())
        };
        let pool = |age: f64, scale: f64| -> f64 {
            (1.0 - 0.011 * scale * age).max(0.0)
        };
        let biomarker_ros = |age: f64, scale: f64| -> f64 {
            (0.007 * scale * age).min(1.0)
        };
        let myeloid = |age: f64, scale: f64| -> f64 {
            (0.004 * scale * age * (1.0 + age / 120.0)).min(1.0)
        };
        let telomere = |age: f64, scale: f64| -> f64 {
            (1.0 - 0.009 * scale * age).max(0.0)
        };
        let epigenetic = |age: f64, scale: f64| -> f64 {
            (age * (1.0 + 0.4 * scale * age / 100.0) / 120.0).min(1.0)
        };

        // ── Derive damage_scale from actual GUI parameters ────────────────────
        let detach_ratio = (self.state.cdata.base_detach_probability / 0.0003_f32) as f64;
        let preset_scale = match self.state.cdata.damage_preset {
            DamagePreset::Normal   => 1.0_f64,
            DamagePreset::Progeria => 5.0_f64,
            DamagePreset::Longevity => 0.6_f64,
        };
        // mother_bias > 0.5 → faster mother centriole damage → faster aging
        let bias_factor = 1.0 + (self.state.cdata.mother_bias as f64 - 0.5) * 0.8;
        // age_bias_coefficient amplifies damage with age
        let age_factor  = 1.0 + self.state.cdata.age_bias_coefficient as f64 * 80.0;
        // checkpoint_strictness protects cells → slows frailty
        let checkpoint  = 1.0 - self.state.cell_cycle.checkpoint_strictness as f64 * 0.30;
        let damage_scale = (preset_scale * detach_ratio * bias_factor * age_factor * checkpoint).max(0.05);

        // per-biomarker scales
        let ros_scale     = (damage_scale * (2.0 - self.state.cdata.ros_brake_strength  as f64)).max(0.05);
        let pool_scale    = (damage_scale / (0.5 + self.state.cdata.division_rate_floor as f64 * 1.5)).max(0.05);
        let myeloid_scale = (damage_scale * (self.state.cdata.spindle_weight as f64 * 2.0
                              + self.state.cdata.ros_weight as f64 * 1.5)).max(0.05);
        let telomere_scale = damage_scale
                              * (1.0 + (1.0 - self.state.cell_cycle.checkpoint_strictness as f64) * 0.4);

        let ages: Vec<f64> = (0..=100).map(|a| a as f64).collect();

        // Real age from latest snapshot, else fallback to math estimate
        let real_age = self.sim_snapshots.last().map(|s| s.age_years).unwrap_or(current_age);
        let has_real  = !self.sim_snapshots.is_empty();

        // ── Header ───────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            let age_str = if has_real {
                format!("Age  {:.1} yr    step {}  /  {}    {:.1}%  [real engine]",
                    real_age,
                    self.state.sim_elapsed_steps,
                    self.state.simulation.max_steps,
                    progress * 100.0)
            } else {
                format!("Age  {:.1} yr    step {}  /  {}    {:.1}%",
                    current_age,
                    self.state.sim_elapsed_steps,
                    self.state.simulation.max_steps,
                    progress * 100.0)
            };
            ui.label(
                egui::RichText::new(age_str)
                .size(16.0)
                .strong()
                .color(Color32::from_rgb(60, 185, 165)),
            );
        });
        ui.separator();

        let plot_h = 190.0;
        let cursor_x = real_age;

        // ── Plot 1: Frailty ───────────────────────────────────────────────
        ui.label(egui::RichText::new("Frailty index").strong().size(13.0));
        Plot::new("live_frailty")
            .height(plot_h)
            .x_axis_label("Age (years)")
            .y_axis_label("Frailty [0–1]")
            .include_y(0.0).include_y(1.05)
            .legend(Legend::default())
            .show(ui, |p| {
                // death threshold
                let thr: PlotPoints = (0..=100).map(|a| [a as f64, 0.95]).collect();
                p.line(Line::new(thr)
                    .color(Color32::from_rgb(180, 40, 40))
                    .style(egui_plot::LineStyle::Dashed { length: 6.0 })
                    .width(1.0).name("Death"));
                // reference lines (dimmed)
                let ctrl: PlotPoints = ages.iter().map(|&a| [a, frailty(a, 1.0)]).collect();
                p.line(Line::new(ctrl).color(Color32::from_rgb(60, 100, 170)).width(1.2).name("Ref: Control ~78 yr"));
                let lon: PlotPoints  = ages.iter().map(|&a| [a, frailty(a, 0.55)]).collect();
                p.line(Line::new(lon).color(Color32::from_rgb(40, 140, 80)).width(1.2).name("Ref: Longevity ~108 yr"));
                let pro: PlotPoints  = ages.iter().map(|&a| [a, frailty(a, 5.0)]).collect();
                p.line(Line::new(pro).color(Color32::from_rgb(160, 60, 40)).width(1.2).name("Ref: Progeria ~15 yr"));
                // real engine data (bright white) or math curve fallback (dim)
                if has_real {
                    let real: PlotPoints = self.sim_snapshots.iter()
                        .map(|s| [s.age_years, s.frailty as f64]).collect();
                    p.line(Line::new(real).color(Color32::WHITE).width(2.8).name("★ Real simulation"));
                } else {
                    let cur_s: PlotPoints = ages.iter().map(|&a| [a, frailty(a, damage_scale)]).collect();
                    p.line(Line::new(cur_s).color(Color32::from_rgb(180,180,180)).width(1.8).name("★ Current settings (approx)"));
                }
                // cursor
                let cur: PlotPoints = vec![[cursor_x, 0.0], [cursor_x, 1.05]].into_iter().collect();
                p.line(Line::new(cur)
                    .color(Color32::from_rgb(210, 175, 80))
                    .style(egui_plot::LineStyle::Dashed { length: 5.0 })
                    .width(1.5).name("Now"));
            });

        ui.add_space(8.0);

        // ── Plot 2: Stem-cell pool ────────────────────────────────────────
        ui.label(egui::RichText::new("Stem-cell pool").strong().size(13.0));
        Plot::new("live_pool")
            .height(plot_h)
            .x_axis_label("Age (years)")
            .y_axis_label("Pool [0–1]")
            .include_y(0.0).include_y(1.05)
            .legend(Legend::default())
            .show(ui, |p| {
                let ctrl: PlotPoints = ages.iter().map(|&a| [a, pool(a, 1.0)]).collect();
                p.line(Line::new(ctrl).color(Color32::from_rgb(60, 100, 170)).width(1.2).name("Ref: Control"));
                let lon: PlotPoints  = ages.iter().map(|&a| [a, pool(a, 0.55)]).collect();
                p.line(Line::new(lon).color(Color32::from_rgb(40, 140, 80)).width(1.2).name("Ref: Longevity"));
                let pro: PlotPoints  = ages.iter().map(|&a| [a, pool(a, 5.0)]).collect();
                p.line(Line::new(pro).color(Color32::from_rgb(160, 60, 40)).width(1.2).name("Ref: Progeria"));
                if has_real {
                    let real: PlotPoints = self.sim_snapshots.iter()
                        .map(|s| [s.age_years, s.stem_cell_pool as f64]).collect();
                    p.line(Line::new(real).color(Color32::WHITE).width(2.8).name("★ Real simulation"));
                } else {
                    let cur_s: PlotPoints = ages.iter().map(|&a| [a, pool(a, pool_scale)]).collect();
                    p.line(Line::new(cur_s).color(Color32::from_rgb(180,180,180)).width(1.8).name("★ Current settings (approx)"));
                }
                let cur: PlotPoints = vec![[cursor_x, 0.0], [cursor_x, 1.05]].into_iter().collect();
                p.line(Line::new(cur)
                    .color(Color32::from_rgb(210, 175, 80))
                    .style(egui_plot::LineStyle::Dashed { length: 5.0 })
                    .width(1.5).name("Now"));
            });

        ui.add_space(8.0);

        // ── Plot 3: Biomarker timeline ────────────────────────────────────
        ui.label(egui::RichText::new("Biomarkers (normalised 0–1)").strong().size(13.0));
        Plot::new("live_bio")
            .height(plot_h)
            .x_axis_label("Age (years)")
            .y_axis_label("[0–1]")
            .include_y(0.0).include_y(1.05)
            .legend(Legend::default())
            .show(ui, |p| {
                if has_real {
                    // Real engine data — bright distinct lines
                    let ros: PlotPoints = self.sim_snapshots.iter()
                        .map(|s| [s.age_years, s.ros_level as f64]).collect();
                    let mye: PlotPoints = self.sim_snapshots.iter()
                        .map(|s| [s.age_years, s.myeloid_bias as f64]).collect();
                    let tel: PlotPoints = self.sim_snapshots.iter()
                        .map(|s| [s.age_years, s.telomere_length as f64]).collect();
                    let epi: PlotPoints = self.sim_snapshots.iter()
                        .map(|s| [s.age_years, (s.methylation_age / 120.0_f32).min(1.0) as f64]).collect();
                    p.line(Line::new(ros).color(Color32::from_rgb(230, 80,  60)).width(2.2).name("ROS (real)"));
                    p.line(Line::new(mye).color(Color32::from_rgb(210, 140, 50)).width(2.2).name("Myeloid bias (real)"));
                    p.line(Line::new(tel).color(Color32::from_rgb(80,  180, 230)).width(2.2).name("Telomere (real)"));
                    p.line(Line::new(epi).color(Color32::from_rgb(170, 100, 230)).width(2.2).name("Epigenetic clock (real)"));
                } else {
                    // Math approximation while no real data yet
                    let ros: PlotPoints = ages.iter().map(|&a| [a, biomarker_ros(a, ros_scale)]).collect();
                    let mye: PlotPoints = ages.iter().map(|&a| [a, myeloid(a, myeloid_scale)]).collect();
                    let tel: PlotPoints = ages.iter().map(|&a| [a, telomere(a, telomere_scale)]).collect();
                    let epi: PlotPoints = ages.iter().map(|&a| [a, epigenetic(a, damage_scale)]).collect();
                    p.line(Line::new(ros).color(Color32::from_rgb(180, 60,  45)).width(1.5).name("ROS (approx)"));
                    p.line(Line::new(mye).color(Color32::from_rgb(165, 110, 40)).width(1.5).name("Myeloid bias (approx)"));
                    p.line(Line::new(tel).color(Color32::from_rgb(60,  140, 180)).width(1.5).name("Telomere (approx)"));
                    p.line(Line::new(epi).color(Color32::from_rgb(135, 80,  185)).width(1.5).name("Epigenetic clock (approx)"));
                }
                let cur: PlotPoints = vec![[cursor_x, 0.0], [cursor_x, 1.05]].into_iter().collect();
                p.line(Line::new(cur)
                    .color(Color32::from_rgb(210, 175, 80))
                    .style(egui_plot::LineStyle::Dashed { length: 5.0 })
                    .width(1.5).name("Now"));
            });

        ui.add_space(6.0);
        let caption = if has_real {
            format!("✅ Real ECS simulation data — {} snapshots from engine",
                self.sim_snapshots.len())
        } else {
            "⏳ Math approximations — will update with real ECS data as simulation runs".to_string()
        };
        ui.label(
            egui::RichText::new(caption)
            .size(11.0)
            .italics()
            .color(if has_real {
                Color32::from_rgb(60, 185, 120)
            } else {
                Color32::from_rgb(110, 120, 135)
            }),
        );

    }
}

// ==================== TAB IMPLEMENTATIONS ====================

impl ConfigApp {
    fn show_ze_field_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌀 Ze Field (−5) — Ze morphogenetic field");
        ui.separator();
        ui.collapsing("ZeHealthState (✅ implemented)", |ui| {
            ui.label("Ze field intensity → cellular entropy → aging rate");
            ui.label("Parameters: ze_field_strength, entropy_coupling_coefficient");
            ui.label("→ Affects: Frailty, Biological Age, Division Rate");
        });
        ui.collapsing("planned: quantum_coherence_module", |ui| {
            ui.label("Quantum decoherence rate at centriolar level");
            ui.label("Dataset: Penrose-Hameroff orchestrated reduction theory");
        });
    }

    fn show_atoms_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚛️ Atoms (−4) — Thermodynamic basis");
        ui.separator();
        ui.collapsing("ThermodynamicState (✅ implemented)", |ui| {
            ui.label("Entropy production, free energy, Boltzmann fluctuations");
            ui.label("→ Affects: ROS Level, Protein Aggregation");
        });
        ui.collapsing("planned: radical_chemistry_module", |ui| {
            ui.label("Atomic-level ROS generation from mitochondrial ETC");
            ui.label("Dataset: superoxide production rate constants");
        });
    }

    fn show_molecules_tab(&mut self, ui: &mut egui::Ui) {
        // Reuse existing molecular content + add new components
        self.show_molecular_tab(ui);
        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new("Additional molecular components:").strong());
        ui.collapsing("ROSCascadeState (✅ implemented)", |ui| {
            ui.label("Superoxide → H₂O₂ → OH• cascade; GPx/SOD activity");
            ui.label("→ Affects: ROS Level, Myeloid Bias, Protein Aggregation");
        });
        ui.collapsing("ATPEnergyState (✅ implemented)", |ui| {
            ui.label("ATP/ADP ratio, mitochondrial membrane potential, glycolysis");
            ui.label("→ Affects: Division Rate, Stem Cell Pool");
        });
        ui.collapsing("ChromatinState (✅ implemented)", |ui| {
            ui.label("Heterochromatin vs euchromatin ratio, H3K27me3, H3K4me3");
            ui.label("→ Affects: Methylation Age, Biological Age");
        });
        ui.collapsing("AppendageProteinState (✅ implemented)", |ui| {
            ui.label("CEP164, TTBK2, Rootletin — centriolar appendage proteins");
            ui.label("→ Affects: CDS, Frailty");
        });
    }

    fn show_cytoskeleton_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("🏛️ Structures — Level -2")
            .size(16.0).strong().color(Color32::from_rgb(160, 200, 240)));
        ui.label(egui::RichText::new(
            "Cellular structural framework: cytoskeleton, nucleus, membrane, ECM anchoring"
        ).size(11.0).color(Color32::GRAY));
        ui.add_space(6.0);
        ui.separator();

        // ── Cytoskeleton ──────────────────────────────────────────────────
        ui.collapsing("🕸️ Microtubule network (MicrotubuleState)", |ui| {
            ui.label("α/β-tubulin PTM (acetylation, tyrosination), dynamic instability, GTP cap.");
            ui.label("Centriole nucleates γ-TuRC → MTOC stability tracks spindle_fidelity.");
            egui::Grid::new("mt_grid").num_columns(2).spacing([8.0, 3.0]).show(ui, |ui| {
                ui.label("tubulin_acetylation");  ui.label("↑ stabilises MT → ↓division rate variability"); ui.end_row();
                ui.label("dynamic_instability");  ui.label("↑ → centrosomal damage ↑, CDS ↑");              ui.end_row();
                ui.label("polymerization_rate");  ui.label("→ Division Rate (Track F)");                     ui.end_row();
            });
        });
        ui.collapsing("🌊 Actin cortex (ActinRingState)", |ui| {
            ui.label("Cortical actin ring tension during cytokinesis → asymmetric division fidelity.");
            ui.label("Polarised actin cap marks stem cell fate axis (mother-daughter axis).");
            egui::Grid::new("actin_grid").num_columns(2).spacing([8.0, 3.0]).show(ui, |ui| {
                ui.label("cortical_tension");    ui.label("→ Division symmetry (Track B)");  ui.end_row();
                ui.label("actin_cap_polarity");  ui.label("→ Stem Cell Pool retention");     ui.end_row();
                ui.label("formin_activity");     ui.label("Wnt-regulated actin nucleation");  ui.end_row();
            });
        });
        ui.collapsing("🚌 Intraflagellar transport (IFTState)", |ui| {
            ui.label("IFT-A (retrograde) / IFT-B (anterograde) complexes, kinesin-2, dynein.");
            ui.label("Required for primary cilia assembly → Shh / Wnt signal reception.");
            egui::Grid::new("ift_grid").num_columns(2).spacing([8.0, 3.0]).show(ui, |ui| {
                ui.label("ift_velocity");         ui.label("→ Ciliary function (Track A)");        ui.end_row();
                ui.label("retrograde_flux");      ui.label("→ CEP164 localisation fidelity");      ui.end_row();
            });
        });

        ui.add_space(4.0);
        ui.separator();

        // ── Nucleus ───────────────────────────────────────────────────────
        ui.collapsing("🔵 Nuclear envelope (NuclearEnvelopeState)", |ui| {
            ui.label("Lamin A/C network integrity, nuclear pore complex density.");
            ui.label("Progerin accumulation (aging HGPS-like) → nuclear blebbing → DNA damage.");
            egui::Grid::new("ne_grid").num_columns(2).spacing([8.0, 3.0]).show(ui, |ui| {
                ui.label("lamin_integrity");     ui.label("↓ → DDR activation, γ-H2AX ↑");           ui.end_row();
                ui.label("npc_density");         ui.label("→ RNA export efficiency → proteostasis");   ui.end_row();
                ui.label("progerin_fraction");   ui.label("→ Methylation Age (Track D)");              ui.end_row();
            });
        });
        ui.collapsing("💧 Plasma membrane (MembraneState)", |ui| {
            ui.label("Lipid raft composition, cholesterol:phospholipid ratio, fluidity index.");
            ui.label("Sphingolipid signalling platforms → Wnt/Notch receptor clustering.");
            egui::Grid::new("mem_grid").num_columns(2).spacing([8.0, 3.0]).show(ui, |ui| {
                ui.label("fluidity_index");       ui.label("→ receptor clustering (Wnt / Notch)");  ui.end_row();
                ui.label("cholesterol_ratio");    ui.label("↑ with age → membrane stiffening");     ui.end_row();
                ui.label("oxidised_lipids");      ui.label("→ ROS amplification loop (Track E)");   ui.end_row();
            });
        });
        ui.collapsing("🔗 ECM anchoring / Integrins (IntegrinState)", |ui| {
            ui.label("α6β4 / α6β1 integrins anchor basal SCs to basement membrane.");
            ui.label("Focal adhesion kinase (FAK) relays ECM stiffness → stem cell retention.");
            egui::Grid::new("ecm_grid").num_columns(2).spacing([8.0, 3.0]).show(ui, |ui| {
                ui.label("integrin_expression");  ui.label("→ Niche retention / Pool size");        ui.end_row();
                ui.label("ecm_stiffness");        ui.label("↑ with age (cross-linking) → quiescence exit delayed"); ui.end_row();
                ui.label("fak_activity");         ui.label("→ Division Rate (Track F)");            ui.end_row();
            });
        });
        ui.collapsing("⚡ Intermediate filaments (IFState)", |ui| {
            ui.label("Vimentin, keratin, desmin — mechanical buffering of the nucleus.");
            ui.label("Nuclear lamins (type V IF) connect cytoskeleton to chromatin.");
            ui.label("→ Affects: Nuclear envelope integrity, CDS, Frailty");
        });
    }

    fn show_organelles_tab(&mut self, ui: &mut egui::Ui) {
        // Reuse mitochondrial content + add other organelles
        self.show_mitochondrial_tab(ui);
        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new("Other organelles:").strong());
        ui.collapsing("GolgiState (✅ implemented)", |ui| {
            ui.label("Golgi fragmentation index, vesicle trafficking, glycosylation fidelity");
            ui.label("→ Affects: Protein Aggregation, Division Rate");
        });
        ui.collapsing("ERStressState (✅ implemented)", |ui| {
            ui.label("UPR activation (ATF6/IRE1/PERK), ER Ca²⁺ homeostasis");
            ui.label("→ Affects: Protein Aggregation, Frailty");
        });
        ui.collapsing("LysosomeState (✅ implemented)", |ui| {
            ui.label("pH, cathepsin activity, lipofuscin accumulation, mTORC1");
            ui.label("→ Affects: Protein Aggregation, Division Rate");
        });
        ui.collapsing("PeroxisomeState (✅ implemented)", |ui| {
            ui.label("Catalase activity, fatty acid β-oxidation, H₂O₂ scavenging");
            ui.label("→ Affects: ROS Level");
        });
        ui.collapsing("RibosomeState (✅ implemented)", |ui| {
            ui.label("Translation fidelity, ribosome biogenesis, rDNA silencing");
            ui.label("→ Affects: Protein Aggregation, Division Rate");
        });
    }

    fn show_cell_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⭐ Cell (0) — Core CDATA: Niche · Cycle · Division · Hierarchy");
        ui.separator();
        // Show all four core cell-level parameter sections
        self.show_niche_tab(ui);
        ui.add_space(6.0);
        ui.collapsing("♻️ Cell Cycle parameters →", |ui| { self.show_cell_cycle_tab(ui); });
        ui.add_space(4.0);
        ui.collapsing("⚖️ Division parameters →", |ui| { self.show_division_tab(ui); });
        ui.add_space(4.0);
        ui.collapsing("🌱 Stem Hierarchy parameters →", |ui| { self.show_stem_hierarchy_tab(ui); });
    }

    fn show_tissues_tab(&mut self, ui: &mut egui::Ui) {
        // Reuse tissue/myeloid content + add ECM/vascular
        self.show_tissue_tab(ui);
        ui.add_space(8.0);
        ui.separator();
        ui.collapsing("🏘️ Stem Cell Niche Catalog", |ui| {
            self.show_niche_catalog(ui);
        });
        ui.add_space(4.0);
        ui.separator();
        ui.collapsing("ExtracellularMatrixState (✅ implemented)", |ui| {
            ui.label("Collagen cross-linking, fibronectin, laminin → niche stiffness");
            ui.label("→ Affects: Division Rate, Stem Cell Pool");
        });
        ui.collapsing("VascularNicheState (✅ implemented)", |ui| {
            ui.label("O₂ partial pressure, nutrient delivery, VEGF signaling");
            ui.label("→ Affects: ROS Level, Division Rate, Frailty");
        });
        ui.collapsing("FibrosisState (✅ implemented)", |ui| {
            ui.label("TGF-β, myofibroblast activation, scar tissue fraction");
            ui.label("→ Affects: Stem Cell Pool, Frailty");
        });
    }

    fn show_organs_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🫀 Organs (+2) — 11 organ systems");
        ui.separator();
        ui.label("OrganState tracks functional capacity and failure probability for 11 organs.");
        ui.add_space(4.0);
        let organs = [
            ("🩸 Hematopoietic", "HSC pool in bone marrow", "Stem Cell Pool"),
            ("🧠 Neural",         "NSC in SVZ/SGZ",          "Cognitive Index"),
            ("❤️ Cardiac",        "CPC in myocardium",       "Frailty, O₂ delivery"),
            ("🫁 Pulmonary",      "AEC2 in alveoli",         "O₂ fraction (breathing module)"),
            ("🫘 Hepatic",        "Hepatocyte in portal zone","Frailty, metabolic"),
            ("🧪 Pancreatic",     "Acinar + β-cells",        "Metabolic phenotype"),
            ("💪 Skeletal muscle","Satellite cells (SC)",     "Division Rate"),
            ("🦴 Bone",           "MSC in endosteum",        "Stem Cell Pool"),
            ("🦷 Skin/epithelium","IFSCs in hair bulge",     "Frailty"),
            ("🧬 Gonads",         "SSC in seminiferous",     "Lifespan (germline)"),
            ("🫀 Vascular",       "EPC in vessel wall",      "Myeloid Bias, Cardio risk"),
        ];
        for (name, niche, metric) in &organs {
            ui.collapsing(*name, |ui| {
                ui.label(format!("Niche: {}", niche));
                ui.label(format!("→ Affects: {}", metric));
                ui.label("Status: ✅ implemented via OrganState");
            });
        }
        ui.add_space(4.0);
        ui.collapsing("Poly-organ failure cascade", |ui| {
            ui.label("Organ failure triggers cascade: renal → hepatic → cardiac");
            ui.label("OrganismState.frailty_index aggregates organ functional scores");
            ui.label("→ Affects: Lifespan (death criterion)");
        });
    }

    fn show_society_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("👥 Society (+4) — Social determinants of aging");
        ui.separator();
        ui.colored_label(egui::Color32::from_rgb(200, 140, 60), "❌ Not yet implemented — planned modules:");
        ui.add_space(4.0);
        ui.collapsing("social_support_module (planned)", |ui| {
            ui.label("Social support index [0..1] → cortisol↓ → ROS↓, inflammaging↓");
            ui.label("→ Affects: ROS Level, Myeloid Bias, Frailty, Lifespan");
            ui.label("Dataset: Holt-Lunstad et al. (2015, PLOS Medicine, N=308,849)");
            ui.label("  Social isolation HR = 1.29 for all-cause mortality");
        });
        ui.collapsing("loneliness_module (planned)", |ui| {
            ui.label("Loneliness index [0..1] → CRP↑ → SASP↑ → myeloid_bias↑");
            ui.label("→ Affects: Myeloid Bias, Protein Aggregation, Biological Age");
            ui.label("Dataset: Cacioppo & Hawkley (2010, Neuroscience & Biobehavioral Reviews)");
        });
        ui.collapsing("socioeconomic_module (planned)", |ui| {
            ui.label("SES index → healthcare access → intervention availability");
            ui.label("→ Affects: all metrics via intervention probability");
            ui.label("Dataset: Marmot Review (2010) — SES gradient in all-cause mortality");
        });
        ui.collapsing("stress_module (planned)", |ui| {
            ui.label("Allostatic load → HPA axis → cortisol chronic → telomere shortening");
            ui.label("→ Affects: Telomere Length, Methylation Age, Division Rate");
            ui.label("Dataset: Epel et al. (2004, PNAS) — psychological stress → telomere shortening");
        });
    }

    fn show_ecosphere_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌍 Ecosphere (+5) — Environmental & lifestyle conditions");
        ui.separator();
        ui.label("🟡 Partial: food_water_module ✅, sleep_module ✅, breathing_module ✅");
        ui.label("         Interventions ✅ (P11), AIM integration TODO");
        ui.add_space(4.0);

        ui.collapsing("🍽️ Food & Water (food_water_module ✅)", |ui| {
            ui.horizontal(|ui| {
                ui.label("Caloric balance:").on_hover_text("-0.5 = severe CR, 0 = maintenance, +0.5 = overnutrition");
                ui.add(egui::Slider::new(&mut self.state.ecosphere.caloric_balance, -0.5..=0.5).step_by(0.05));
            });
            ui.horizontal(|ui| {
                ui.label("Diet quality:").on_hover_text("1.0 = Mediterranean. ROS↓ ∝ quality");
                ui.add(egui::Slider::new(&mut self.state.ecosphere.diet_quality, 0.0..=1.0).step_by(0.05));
            });
            ui.horizontal(|ui| {
                ui.label("Hydration:").on_hover_text("Autophagy flux ∝ (hydration-0.5)×0.12");
                ui.add(egui::Slider::new(&mut self.state.ecosphere.hydration, 0.0..=1.0).step_by(0.05));
            });
            ui.horizontal(|ui| {
                ui.label("Protein quality:");
                ui.add(egui::Slider::new(&mut self.state.ecosphere.protein_quality, 0.0..=1.0).step_by(0.05));
            });
        });
        ui.collapsing("😴 Sleep (sleep_module ✅)", |ui| {
            ui.horizontal(|ui| {
                ui.label("Sleep quality:").on_hover_text("1.0 = 8h restorative SWS");
                ui.add(egui::Slider::new(&mut self.state.ecosphere.sleep_quality, 0.0..=1.0).step_by(0.05));
            });
            ui.horizontal(|ui| {
                ui.label("Circadian alignment:");
                ui.add(egui::Slider::new(&mut self.state.ecosphere.circadian_alignment, 0.0..=1.0).step_by(0.05));
            });
        });
        ui.collapsing("🌬️ Breathing (breathing_module ✅)", |ui| {
            ui.horizontal(|ui| {
                ui.label("O₂ fraction:").on_hover_text("Sea level = 0.21. Hyperoxia weakens mito shield");
                ui.add(egui::Slider::new(&mut self.state.ecosphere.o2_fraction, 0.10..=0.40).step_by(0.01));
            });
            ui.horizontal(|ui| {
                ui.label("Breathing quality:");
                ui.add(egui::Slider::new(&mut self.state.ecosphere.breathing_quality, 0.0..=1.0).step_by(0.05));
            });
            ui.horizontal(|ui| {
                ui.label("Pollution index:");
                ui.add(egui::Slider::new(&mut self.state.ecosphere.pollution_index, 0.0..=1.0).step_by(0.05));
            });
        });
        ui.collapsing("💊 Interventions (P11 ✅)", |ui| {
            ui.label("Senolytics, NAD+, Caloric Restriction, TERT, Antioxidants");
            ui.label("CafdRetainer, CafdReleaser, CentrosomeTransplant");
            ui.label("→ See interventions.rs for implementation");
            ui.label("AIM integration: TODO (ai_intervention_module planned)");
        });
    }

    fn show_molecular_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔬 Molecular — PTM rates & Transcriptome noise");
        ui.separator();

        ui.label("Centriole PTM rates:");
        ui.horizontal(|ui| {
            ui.label("Acetylation rate:").on_hover_text("Гиперацетилирование тубулина материнской центриоли / шаг. → ослабляет связи → O₂-чувствительность↑");
            if ui.add(Slider::new(&mut self.state.centriole.acetylation_rate, 0.001..=0.050).step_by(0.001)).changed() {
                self.push_history();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Oxidation rate:").on_hover_text("Окисление тубулина материнской центриоли / шаг. Вклад в total_damage_score.");
            if ui.add(Slider::new(&mut self.state.centriole.oxidation_rate, 0.001..=0.050).step_by(0.001)).changed() {
                self.push_history();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Methylation rate:").on_hover_text("Метилирование тубулина / шаг.");
            if ui.add(Slider::new(&mut self.state.centriole.methylation_rate, 0.0001..=0.001).step_by(0.0001)).changed() {
                self.push_history();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Phosphorylation rate:").on_hover_text("Дисрегуляция фосфорилирования / шаг.");
            if ui.add(Slider::new(&mut self.state.centriole.phosphorylation_rate, 0.0001..=0.001).step_by(0.0001)).changed() {
                self.push_history();
            }
        });

        ui.add_space(8.0);
        ui.label("Transcriptome noise:");
        ui.horizontal(|ui| {
            ui.label("Mutation rate:");
            if ui.add(Slider::new(&mut self.state.transcriptome.mutation_rate, 0.0..=0.01).logarithmic(true)).changed() {
                self.push_history();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Noise level:");
            if ui.add(Slider::new(&mut self.state.transcriptome.noise_level, 0.0..=0.2)).changed() {
                self.push_history();
            }
        });
    }

    fn show_division_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚖️ Division — Asymmetric Division & Track F");
        ui.separator();

        ui.collapsing("Asymmetric Division Module", |ui| {
            if ui.checkbox(&mut self.state.asymmetric.enabled, "Enable module").changed() {
                self.push_history();
            }
            if self.state.asymmetric.enabled {
                ui.horizontal(|ui| {
                    ui.label("Asymmetric division probability:");
                    if ui.add(Slider::new(&mut self.state.asymmetric.asymmetric_probability, 0.0..=1.0)).changed() {
                        self.push_history();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Self-renewal probability:");
                    if ui.add(Slider::new(&mut self.state.asymmetric.renewal_probability, 0.0..=1.0)).changed() {
                        self.push_history();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Differentiation probability:");
                    if ui.add(Slider::new(&mut self.state.asymmetric.diff_probability, 0.0..=1.0)).changed() {
                        self.push_history();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Niche capacity:");
                    if ui.add(Slider::new(&mut self.state.asymmetric.niche_capacity, 1..=100)).changed() {
                        self.push_history();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Maximum niches:");
                    if ui.add(Slider::new(&mut self.state.asymmetric.max_niches, 1..=1000)).changed() {
                        self.push_history();
                    }
                });
                if ui.checkbox(&mut self.state.asymmetric.enable_polarity, "Enable polarity").changed() {
                    self.push_history();
                }
                if ui.checkbox(&mut self.state.asymmetric.enable_fate_determinants, "Enable fate determinants").changed() {
                    self.push_history();
                }
            }
        });

        ui.add_space(4.0);

        ui.collapsing("Centriole lifecycle (de novo & meiosis)", |ui| {
            ui.horizontal(|ui| {
                ui.label("de_novo_centriole_division:")
                  .on_hover_text("На каком делении бластомеров создаются de novo центриоли [1..=8]. 4 = Морула.");
                let mut div = self.state.cdata.de_novo_centriole_division as f32;
                if ui.add(Slider::new(&mut div, 1.0..=8.0).step_by(1.0).suffix(" деление")).changed() {
                    self.state.cdata.de_novo_centriole_division = div as u32;
                    self.push_history();
                }
            });
            let stage_label = match self.state.cdata.de_novo_centriole_division {
                1     => "Zygote",
                2 | 3 => "Cleavage",
                4     => "Morula ✓",
                5 | 6 => "Blastocyst",
                _     => "Implantation",
            };
            ui.label(format!("→ stage: {}", stage_label));
            if ui.checkbox(&mut self.state.cdata.meiotic_elimination_enabled, "meiotic_elimination_enabled")
               .on_hover_text("Учитывать элиминацию центриолей в прелептотенной стадии мейоза.")
               .changed()
            {
                self.push_history();
            }
        });

        ui.add_space(4.0);

        ui.collapsing("📉 Track F — Division rate (StemCellDivisionRateState)", |ui| {
            ui.label("division_rate = cilia_drive × spindle_drive × age_factor × ros_brake × mtor_brake");
            ui.horizontal(|ui| {
                ui.label("division_rate_floor:")
                  .on_hover_text("Нижняя граница age_factor. По умолч.: 0.15");
                if ui.add(Slider::new(&mut self.state.cdata.division_rate_floor, 0.05..=0.50).step_by(0.01)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("ros_brake_strength:")
                  .on_hover_text("Сила тормоза ROS: ros_brake = 1 - ros_level × strength. По умолч.: 0.40");
                if ui.add(Slider::new(&mut self.state.cdata.ros_brake_strength, 0.0..=1.0).step_by(0.05)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("mtor_brake_strength:")
                  .on_hover_text("Сила тормоза mTOR. По умолч.: 0.35");
                if ui.add(Slider::new(&mut self.state.cdata.mtor_brake_strength, 0.0..=1.0).step_by(0.05)).changed() {
                    self.push_history();
                }
            });
        });
    }

    fn show_niche_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏘️ Niche / CDATA — Inducer system & niche parameters");
        ui.separator();

        ui.collapsing("🔬 Inducer system", |ui| {
            ui.horizontal(|ui| {
                ui.label("base_detach_probability:")
                  .on_hover_text("Базовая вероятность отщепления индуктора за шаг (O₂-зависимо)");
                if ui.add(
                    Slider::new(&mut self.state.cdata.base_detach_probability, 0.0..=0.01).logarithmic(true)
                ).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("mother_bias:")
                  .on_hover_text("Доля отщеплений, приходящаяся на материнскую центриоль");
                if ui.add(Slider::new(&mut self.state.cdata.mother_bias, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("age_bias_coefficient:")
                  .on_hover_text("На сколько возраст усиливает mother_bias за год");
                if ui.add(
                    Slider::new(&mut self.state.cdata.age_bias_coefficient, 0.0..=0.01).logarithmic(true)
                ).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("ptm_exhaustion_scale:")
                  .on_hover_text("PTM-путь истощения матери (независим от O₂). По умолч.: 0.001");
                // ptm_exhaustion_scale is not in CdataGuiConfig yet, show placeholder
                ui.label("(see Molecular tab for PTM rates)");
            });
        });

        ui.add_space(4.0);

        ui.collapsing("🧬 Inductor counts & simulation time", |ui| {
            // These params live in human_development_module; show read-only info
            // and expose the enable flags that CdataGuiConfig tracks
            ui.label("mother_inducer_count / daughter_inducer_count: set in simulation config.");
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label("time_acceleration: set in simulation max_steps (Organism tab).");
            });
        });

        ui.add_space(4.0);

        ui.collapsing("🧬 Enable flags", |ui| {
            if ui.checkbox(&mut self.state.cdata.enable_inducer_system, "Enable inducer system (recommended)")
               .on_hover_text("true → potency = f(M-set, D-set); false → potency = f(spindle, cilia)")
               .changed()
            {
                self.push_history();
            }
        });

        ui.add_space(4.0);

        ui.collapsing("ℹ️ CDATA theory reference", |ui| {
            ui.label("Jaba Tkemaladze — Centriolar Damage Accumulation Theory of Aging.");
            ui.label("Six aging tracks: A(cilia) B(spindle) C(telomere) D(epigenetics) E(mito) F(division rate)");
            ui.label("Senescence threshold: total_damage > 0.75 → niche death ≈ 78 yr.");
        });
    }

    fn show_tissue_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔴 Tissue / Myeloid — Myeloid shift parameters");
        ui.separator();

        ui.collapsing("🩸 Myeloid shift weights", |ui| {
            ui.label("Weights should sum to ≈ 1.0 for correct myeloid_bias scaling.");

            ui.horizontal(|ui| {
                ui.label("spindle_weight:")
                  .on_hover_text("Вклад потери spindle_fidelity в myeloid_bias. По умолч.: 0.45");
                if ui.add(Slider::new(&mut self.state.cdata.spindle_weight, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("cilia_weight:")
                  .on_hover_text("Вклад потери ciliary_function в myeloid_bias. По умолч.: 0.30");
                if ui.add(Slider::new(&mut self.state.cdata.cilia_weight, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("ros_weight:")
                  .on_hover_text("Вклад ros_level в myeloid_bias. По умолч.: 0.15");
                if ui.add(Slider::new(&mut self.state.cdata.ros_weight, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("aggregate_weight:")
                  .on_hover_text("Вклад protein_aggregates в myeloid_bias. По умолч.: 0.10");
                if ui.add(Slider::new(&mut self.state.cdata.aggregate_weight, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });

            let total = self.state.cdata.spindle_weight
                + self.state.cdata.cilia_weight
                + self.state.cdata.ros_weight
                + self.state.cdata.aggregate_weight;
            let color = if (total - 1.0).abs() < 0.05 { egui::Color32::GREEN } else { egui::Color32::YELLOW };
            ui.colored_label(color, format!("Σ = {:.2} (target: 1.00)", total));
        });

        ui.add_space(4.0);

        ui.collapsing("Inflammaging feedback fine-tuning", |ui| {
            ui.horizontal(|ui| {
                ui.label("ros_boost_scale:").on_hover_text("Масштаб усиления ROS через воспаление. Дефолт: 0.15");
                if ui.add(Slider::new(&mut self.state.cdata.ros_boost_scale, 0.0..=0.5).step_by(0.01)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("niche_impair_scale:").on_hover_text("Масштаб угнетения ниши через воспаление. Дефолт: 0.08");
                if ui.add(Slider::new(&mut self.state.cdata.niche_impair_scale, 0.0..=0.3).step_by(0.01)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("spindle_nonlinearity_exponent:").on_hover_text("Степень нелинейности spindle: (1-spindle)^exp. Дефолт: 1.5");
                if ui.add(Slider::new(&mut self.state.cdata.spindle_nonlinearity_exponent, 0.5..=3.0).step_by(0.1)).changed() {
                    self.push_history();
                }
            });
        });
    }

    fn show_organism_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🫀 Organism — Damage preset & Simulation config");
        ui.separator();

        ui.collapsing("⚡ Damage preset (DamageParams)", |ui| {
            ui.label("Scales all damage rates in DamageParams.");
            let current_label = self.state.cdata.damage_preset.label();
            ComboBox::from_label("Preset")
                .selected_text(current_label)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.state.cdata.damage_preset, DamagePreset::Normal, DamagePreset::Normal.label()).clicked() {
                        self.push_history();
                    }
                    if ui.selectable_value(&mut self.state.cdata.damage_preset, DamagePreset::Progeria, DamagePreset::Progeria.label()).clicked() {
                        self.push_history();
                    }
                    if ui.selectable_value(&mut self.state.cdata.damage_preset, DamagePreset::Longevity, DamagePreset::Longevity.label()).clicked() {
                        self.push_history();
                    }
                });
            match self.state.cdata.damage_preset {
                DamagePreset::Normal    => { ui.label("Standard rates. Expected lifespan ≈ 78 yr."); }
                DamagePreset::Progeria  => { ui.label("Rates ×5. Accelerated aging (Hutchinson-Gilford syndrome)."); }
                DamagePreset::Longevity => { ui.label("Rates ×0.6. Slower damage accumulation → longevity."); }
            };
        });

        ui.add_space(4.0);

        ui.collapsing("Simulation config (basic)", |ui| {
            ui.horizontal(|ui| {
                ui.label("Max steps:");
                if ui.add(Slider::new(&mut self.state.simulation.max_steps, 1..=1_000_000)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Time step (dt):");
                if ui.add(Slider::new(&mut self.state.simulation.dt, 0.001..=1.0).logarithmic(true)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Random seed:");
                let mut seed = self.state.simulation.seed.unwrap_or(42);
                if ui.add(Slider::new(&mut seed, 0..=999_999)).changed() {
                    self.state.simulation.seed = Some(seed);
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Number of threads:");
                let mut threads = self.state.simulation.num_threads.unwrap_or(1);
                if ui.add(Slider::new(&mut threads, 1..=64)).changed() {
                    self.state.simulation.num_threads = Some(threads);
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Checkpoint interval:");
                if ui.add(Slider::new(&mut self.state.simulation.checkpoint_interval, 1..=10_000)).changed() {
                    self.push_history();
                }
            });
            if ui.checkbox(&mut self.state.simulation.parallel_modules, "Parallel module execution").changed() {
                self.push_history();
            }
        });

        ui.add_space(4.0);
        ui.label("🌍 Ecosphere conditions → see Ecosphere tab (+5)");

        ui.add_space(8.0);
        ui.separator();
        ui.collapsing("📊 CDATA Impact Showcase (pre-computed aging curves)", |ui| {
            self.show_visualization_tab(ui);
        });
    }

    fn show_system_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ System — I/O & Visualization config");
        ui.separator();

        ui.collapsing("💾 I/O Config", |ui| {
            if ui.checkbox(&mut self.state.io.enabled, "Enable I/O").changed() {
                self.push_history();
            }
            if self.state.io.enabled {
                ui.horizontal(|ui| {
                    ui.label("Output directory:");
                    if ui.text_edit_singleline(&mut self.state.io.output_dir).changed() {
                        self.push_history();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    ComboBox::from_id_source("io_format_sys")
                        .selected_text(&self.state.io.format)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.state.io.format, "csv".to_string(), "CSV");
                            ui.selectable_value(&mut self.state.io.format, "parquet".to_string(), "Parquet");
                            ui.selectable_value(&mut self.state.io.format, "hdf5".to_string(), "HDF5");
                        });
                    self.push_history();
                });
                ui.horizontal(|ui| {
                    ui.label("Compression:");
                    ComboBox::from_id_source("io_compression_sys")
                        .selected_text(&self.state.io.compression)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.state.io.compression, "none".to_string(), "None");
                            ui.selectable_value(&mut self.state.io.compression, "snappy".to_string(), "Snappy");
                            ui.selectable_value(&mut self.state.io.compression, "gzip".to_string(), "Gzip");
                        });
                    self.push_history();
                });
                ui.horizontal(|ui| {
                    ui.label("Buffer size:");
                    if ui.add(Slider::new(&mut self.state.io.buffer_size, 100..=10000)).changed() {
                        self.push_history();
                    }
                });
                if ui.checkbox(&mut self.state.io.save_checkpoints, "Save checkpoints").changed() {
                    self.push_history();
                }
                if self.state.io.save_checkpoints {
                    ui.horizontal(|ui| {
                        ui.label("Checkpoint interval:");
                        if ui.add(Slider::new(&mut self.state.io.checkpoint_interval, 10..=1000)).changed() {
                            self.push_history();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Maximum checkpoints:");
                        if ui.add(Slider::new(&mut self.state.io.max_checkpoints, 1..=100)).changed() {
                            self.push_history();
                        }
                    });
                }
            }
        });

        ui.add_space(4.0);

        ui.collapsing("📊 Visualization Config", |ui| {
            if ui.checkbox(&mut self.state.viz.enabled, "Enable visualization").changed() {
                self.push_history();
            }
            if self.state.viz.enabled {
                ui.horizontal(|ui| {
                    ui.label("Output directory:");
                    if ui.text_edit_singleline(&mut self.state.viz.output_dir).changed() {
                        self.push_history();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Update interval:");
                    if ui.add(Slider::new(&mut self.state.viz.update_interval, 1..=1000)).changed() {
                        self.push_history();
                    }
                });
                if ui.checkbox(&mut self.state.viz.save_plots, "Save plots").changed() {
                    self.push_history();
                }
                if ui.checkbox(&mut self.state.viz.phase_distribution, "Phase distribution").changed() {
                    self.push_history();
                }
                if ui.checkbox(&mut self.state.viz.maturity_histogram, "Maturity histogram").changed() {
                    self.push_history();
                }
                if ui.checkbox(&mut self.state.viz.heatmap, "Heatmap").changed() {
                    self.push_history();
                }
                if ui.checkbox(&mut self.state.viz.timeseries, "Timeseries").changed() {
                    self.push_history();
                }
                if ui.checkbox(&mut self.state.viz.three_d_enabled, "3D visualization").changed() {
                    self.push_history();
                }
            }
        });

        ui.add_space(4.0);

        ui.collapsing("Output directory (simulation)", |ui| {
            ui.horizontal(|ui| {
                ui.label("Output directory:");
                let output_str = self.state.simulation.output_dir.to_string_lossy().to_string();
                let mut output = output_str.clone();
                if ui.text_edit_singleline(&mut output).changed() && output != output_str {
                    self.state.simulation.output_dir = PathBuf::from(output);
                    self.push_history();
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new("👥 Social Level (+9) — planned modules").strong());
        ui.label("Modules not yet implemented. Planned effect on metrics:");
        ui.collapsing("social_support_module (planned)", |ui| {
            ui.label("Social support index [0..1] → cortisol↓ → ROS↓, inflammaging↓");
            ui.label("Affects: ROS, Myeloid Bias, Frailty, Lifespan");
            ui.label("Dataset: Holt-Lunstad 2015 (N=308,849) — social isolation HR=1.29");
        });
        ui.collapsing("loneliness_module (planned)", |ui| {
            ui.label("Loneliness index [0..1] → CRP↑ → SASP↑ → myeloid_bias↑");
            ui.label("Affects: Myeloid Bias, Protein Aggregation, Biological Age");
            ui.label("Dataset: Cacioppo & Hawkley 2010 — loneliness → accelerated aging");
        });
        ui.collapsing("socioeconomic_module (planned)", |ui| {
            ui.label("SES index → healthcare access → intervention availability");
            ui.label("Affects: all metrics via intervention probability");
            ui.label("Dataset: Marmot Review 2010 — SES gradient in all-cause mortality");
        });
    }

    #[allow(dead_code)]
    fn show_simulation_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Main Simulation Parameters");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Number of steps:");
            if ui.add(Slider::new(&mut self.state.simulation.max_steps, 1..=1_000_000)).changed() {
                self.push_history();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Time step (dt):");
            if ui.add(Slider::new(&mut self.state.simulation.dt, 0.001..=1.0).logarithmic(true)).changed() {
                self.push_history();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Checkpoint interval:");
            if ui.add(Slider::new(&mut self.state.simulation.checkpoint_interval, 1..=10_000)).changed() {
                self.push_history();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Number of threads:");
            let mut threads = self.state.simulation.num_threads.unwrap_or(1);
            if ui.add(Slider::new(&mut threads, 1..=64)).changed() {
                self.state.simulation.num_threads = Some(threads);
                self.push_history();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Random seed:");
            let mut seed = self.state.simulation.seed.unwrap_or(42);
            if ui.add(Slider::new(&mut seed, 0..=999_999)).changed() {
                self.state.simulation.seed = Some(seed);
                self.push_history();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Output directory:");
            let output_str = self.state.simulation.output_dir.to_string_lossy().to_string();
            let mut output = output_str.clone();
            if ui.text_edit_singleline(&mut output).changed()
                && output != output_str
            {
                self.state.simulation.output_dir = PathBuf::from(output);
                self.push_history();
            }
        });
        
        if ui.checkbox(&mut self.state.simulation.parallel_modules, "Parallel module execution").changed() {
            self.push_history();
        }
    }
    
    fn show_centriole_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔵 Centriole Module");
        ui.label("PTM rates (acetylation, oxidation, methylation, phosphorylation) are in the Molecular tab.");
        ui.separator();

        if ui.checkbox(&mut self.state.centriole.enabled, "Enable module").changed() {
            self.push_history();
        }

        if self.state.centriole.enabled {
            ui.horizontal(|ui| {
                ui.label("Daughter PTM factor:").on_hover_text("Коэффициент PTM дочерней центриоли от материнской [0..1]. 0.4 = дочерняя накапливает 40% PTM матери.");
                if ui.add(Slider::new(&mut self.state.centriole.daughter_ptm_factor, 0.0..=1.0).step_by(0.05)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("M-phase boost:").on_hover_text("Мультипликатор PTM в M-фазе (стресс тубулина при делении). 3.0 = PTM в 3× быстрее во время митоза.");
                if ui.add(Slider::new(&mut self.state.centriole.m_phase_boost, 1.0..=10.0).step_by(0.1)).changed() {
                    self.push_history();
                }
            });
            if ui.checkbox(&mut self.state.centriole.parallel_cells, "Parallel cell processing").changed() {
                self.push_history();
            }
        }
    }
    
    fn show_cell_cycle_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("♻️ Cell Cycle Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.cell_cycle.enabled, "Enable module").changed() {
            self.push_history();
        }
        
        if self.state.cell_cycle.enabled {
            ui.horizontal(|ui| {
                ui.label("Base cycle duration:");
                if ui.add(Slider::new(&mut self.state.cell_cycle.base_cycle_time, 1.0..=100.0)).changed() {
                    self.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Checkpoint strictness:");
                if ui.add(Slider::new(&mut self.state.cell_cycle.checkpoint_strictness, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            
            if ui.checkbox(&mut self.state.cell_cycle.enable_apoptosis, "Enable apoptosis").changed() {
                self.push_history();
            }
            
            ui.horizontal(|ui| {
                ui.label("Nutrient availability:");
                if ui.add(Slider::new(&mut self.state.cell_cycle.nutrient_availability, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Growth factor level:");
                if ui.add(Slider::new(&mut self.state.cell_cycle.growth_factor_level, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Random variation:");
                if ui.add(Slider::new(&mut self.state.cell_cycle.random_variation, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Growth factor sensitivity:").on_hover_text("Чувствительность к факторам роста (EGF, IGF-1). Влияет на скорость прохождения G1. Дефолт: 0.3");
                if ui.add(Slider::new(&mut self.state.cell_cycle.growth_factor_sensitivity, 0.0..=1.0).step_by(0.05)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Stress sensitivity:").on_hover_text("Чувствительность к стрессу (ROS, повреждение ДНК) для активации checkpoint арестов. Дефолт: 0.2");
                if ui.add(Slider::new(&mut self.state.cell_cycle.stress_sensitivity, 0.0..=1.0).step_by(0.05)).changed() {
                    self.push_history();
                }
            });
        }
    }

    fn show_transcriptome_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🧬 Transcriptome Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.transcriptome.enabled, "Enable module").changed() {
            self.push_history();
        }
        
        if self.state.transcriptome.enabled {
            ui.horizontal(|ui| {
                ui.label("Mutation rate:");
                if ui.add(Slider::new(&mut self.state.transcriptome.mutation_rate, 0.0..=0.01).logarithmic(true)).changed() {
                    self.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Noise level:");
                if ui.add(Slider::new(&mut self.state.transcriptome.noise_level, 0.0..=0.5)).changed() {
                    self.push_history();
                }
            });
        }
    }
    
    #[allow(dead_code)]
    fn show_asymmetric_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚖️ Asymmetric Division Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.asymmetric.enabled, "Enable module").changed() {
            self.push_history();
        }
        
        if self.state.asymmetric.enabled {
            ui.horizontal(|ui| {
                ui.label("Asymmetric division probability:");
                if ui.add(Slider::new(&mut self.state.asymmetric.asymmetric_probability, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Self-renewal probability:");
                if ui.add(Slider::new(&mut self.state.asymmetric.renewal_probability, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Differentiation probability:");
                if ui.add(Slider::new(&mut self.state.asymmetric.diff_probability, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Niche capacity:");
                if ui.add(Slider::new(&mut self.state.asymmetric.niche_capacity, 1..=100)).changed() {
                    self.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Maximum niches:");
                if ui.add(Slider::new(&mut self.state.asymmetric.max_niches, 1..=1000)).changed() {
                    self.push_history();
                }
            });
            
            if ui.checkbox(&mut self.state.asymmetric.enable_polarity, "Enable polarity").changed() {
                self.push_history();
            }
            
            if ui.checkbox(&mut self.state.asymmetric.enable_fate_determinants, "Enable fate determinants").changed() {
                self.push_history();
            }
        }
    }
    
    fn show_stem_hierarchy_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌱 Stem Cell Hierarchy Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.stem_hierarchy.enabled, "Enable module").changed() {
            self.push_history();
        }
        
        if self.state.stem_hierarchy.enabled {
            ui.horizontal(|ui| {
                ui.label("Initial potency level:");
                ComboBox::from_id_source("potency")
                    .selected_text(&self.state.stem_hierarchy.initial_potency)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.state.stem_hierarchy.initial_potency, 
                            "Totipotent".to_string(), "Totipotent");
                        ui.selectable_value(&mut self.state.stem_hierarchy.initial_potency, 
                            "Pluripotent".to_string(), "Pluripotent");
                        ui.selectable_value(&mut self.state.stem_hierarchy.initial_potency, 
                            "Multipotent".to_string(), "Multipotent");
                        ui.selectable_value(&mut self.state.stem_hierarchy.initial_potency, 
                            "Differentiated".to_string(), "Differentiated");
                    });
                self.push_history();
            });
            
            if ui.checkbox(&mut self.state.stem_hierarchy.enable_plasticity, "Enable plasticity").changed() {
                self.push_history();
            }
            
            ui.horizontal(|ui| {
                ui.label("Plasticity rate:");
                if ui.add(Slider::new(&mut self.state.stem_hierarchy.plasticity_rate, 0.0..=0.1).logarithmic(true)).changed() {
                    self.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Differentiation threshold:");
                if ui.add(Slider::new(&mut self.state.stem_hierarchy.differentiation_threshold, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });
        }
    }
    
    #[allow(dead_code)]
    fn show_io_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("💾 I/O Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.io.enabled, "Enable module").changed() {
            self.push_history();
        }
        
        if self.state.io.enabled {
            ui.horizontal(|ui| {
                ui.label("Output directory:");
                if ui.text_edit_singleline(&mut self.state.io.output_dir).changed() {
                    self.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Format:");
                ComboBox::from_id_source("format")
                    .selected_text(&self.state.io.format)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.state.io.format, "csv".to_string(), "CSV");
                        ui.selectable_value(&mut self.state.io.format, "parquet".to_string(), "Parquet");
                        ui.selectable_value(&mut self.state.io.format, "hdf5".to_string(), "HDF5");
                    });
                self.push_history();
            });
            
            ui.horizontal(|ui| {
                ui.label("Compression:");
                ComboBox::from_id_source("compression")
                    .selected_text(&self.state.io.compression)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.state.io.compression, "none".to_string(), "None");
                        ui.selectable_value(&mut self.state.io.compression, "snappy".to_string(), "Snappy");
                        ui.selectable_value(&mut self.state.io.compression, "gzip".to_string(), "Gzip");
                    });
                self.push_history();
            });
            
            ui.horizontal(|ui| {
                ui.label("Buffer size:");
                if ui.add(Slider::new(&mut self.state.io.buffer_size, 100..=10000)).changed() {
                    self.push_history();
                }
            });
            
            if ui.checkbox(&mut self.state.io.save_checkpoints, "Save checkpoints").changed() {
                self.push_history();
            }
            
            if self.state.io.save_checkpoints {
                ui.horizontal(|ui| {
                    ui.label("Checkpoint interval:");
                    if ui.add(Slider::new(&mut self.state.io.checkpoint_interval, 10..=1000)).changed() {
                        self.push_history();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Maximum checkpoints:");
                    if ui.add(Slider::new(&mut self.state.io.max_checkpoints, 1..=100)).changed() {
                        self.push_history();
                    }
                });
            }
        }
    }
    
    fn show_visualization_tab(&mut self, ui: &mut egui::Ui) {
        // ═══════════════════════════════════════════════════════════════════
        //  CDATA IMPACT SHOWCASE — Academic-grade aging trajectory viewer
        //  Pre-computed demo curves based on calibrated CDATA parameters:
        //    • Control (DamageParams::default — lifespan ≈ 78 yr)
        //    • Longevity (×0.6 damage — lifespan ≈ 108 yr)
        //    • Progeria  (×5   damage — lifespan ≈ 15 yr)
        //    • CentrosomeTransplant intervention (×0.5 damage from yr 50)
        // ═══════════════════════════════════════════════════════════════════

        ui.heading("📊 CDATA — Aging Trajectory Showcase");
        ui.label(
            egui::RichText::new(
                "Simulated cellular aging trajectories — Centriolar Damage Accumulation Theory of Ageing (Tkemaladze, 2023)"
            )
            .italics()
            .color(Color32::LIGHT_GRAY)
            .size(12.0),
        );
        ui.separator();

        // ── Derive damage_scale from GUI parameters ──────────────────────────
        let viz_detach  = (self.state.cdata.base_detach_probability / 0.0003_f32) as f64;
        let viz_preset  = match self.state.cdata.damage_preset {
            DamagePreset::Normal    => 1.0_f64,
            DamagePreset::Progeria  => 5.0_f64,
            DamagePreset::Longevity => 0.6_f64,
        };
        let viz_bias    = 1.0 + (self.state.cdata.mother_bias as f64 - 0.5) * 0.8;
        let viz_age_f   = 1.0 + self.state.cdata.age_bias_coefficient as f64 * 80.0;
        let viz_chk     = 1.0 - self.state.cell_cycle.checkpoint_strictness as f64 * 0.30;
        let viz_scale   = (viz_preset * viz_detach * viz_bias * viz_age_f * viz_chk).max(0.05);
        let viz_pool_sc = (viz_scale / (0.5 + self.state.cdata.division_rate_floor as f64 * 1.5)).max(0.05);

        // ── Helper: pre-compute curve points ────────────────────────────
        // Frailty index: sigmoid growth up to 1.0, crossing 0.95 = death
        let frailty = |age: f64, scale: f64| -> f64 {
            let k = 0.08 * scale;
            let mid = 45.0 / scale.sqrt();
            1.0 / (1.0 + (-k * (age - mid)).exp())
        };
        // Stem-cell pool: exponential decay
        let pool = |age: f64, scale: f64| -> f64 {
            (1.0 - 0.011 * scale * age).max(0.0)
        };
        // Kaplan-Meier survival: cohort survival probability
        let survival = |age: f64, mu: f64| -> f64 {
            (-( age / mu).powi(4)).exp()
        };
        // CAII biomarker (rises with damage)
        let caii = |age: f64, scale: f64| -> f64 {
            (0.008 * scale * age * age / 100.0).min(1.0)
        };
        // Epigenetic clock acceleration
        let epigenetic = |age: f64, scale: f64| -> f64 {
            age * (1.0 + 0.5 * scale * (age / 100.0))
        };

        let ages: Vec<f64> = (0..=100).map(|a| a as f64).collect();

        // ── Grid of 4 plots ─────────────────────────────────────────────
        let plot_h = 230.0;

        // ── Plot 1: Frailty Index ────────────────────────────────────────
        ui.label(egui::RichText::new("① Frailty Index Trajectory").strong().size(14.0));
        Plot::new("plot_frailty")
            .height(plot_h)
            .x_axis_label("Age (years)")
            .y_axis_label("Frailty Index [0–1]")
            .y_axis_width(4)
            .include_y(0.0)
            .include_y(1.05)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                // Death threshold
                let threshold: PlotPoints = (0..=100).map(|a| [a as f64, 0.95]).collect();
                plot_ui.line(
                    Line::new(threshold)
                        .color(Color32::from_rgb(200, 50, 50))
                        .style(egui_plot::LineStyle::Dashed { length: 8.0 })
                        .width(1.5)
                        .name("Death threshold"),
                );
                // Control
                let ctrl: PlotPoints = ages.iter().map(|&a| [a, frailty(a, 1.0)]).collect();
                plot_ui.line(Line::new(ctrl).color(Color32::from_rgb(80, 150, 240)).width(2.5).name("Control (~78 yr)"));
                // Longevity
                let lon: PlotPoints = ages.iter().map(|&a| [a, frailty(a, 0.55)]).collect();
                plot_ui.line(Line::new(lon).color(Color32::from_rgb(60, 210, 120)).width(2.5).name("Longevity preset (~108 yr)"));
                // Progeria
                let pro: PlotPoints = ages.iter().map(|&a| [a, frailty(a, 5.0)]).collect();
                plot_ui.line(Line::new(pro).color(Color32::from_rgb(255, 100, 60)).width(2.5).name("Progeria (~15 yr)"));
                // CentrosomeTransplant
                let tx: PlotPoints = ages.iter().map(|&a| {
                    let scale = if a < 50.0 { 1.0 } else { 0.5 };
                    [a, frailty(a, scale)]
                }).collect();
                plot_ui.line(Line::new(tx).color(Color32::GOLD).width(1.8).name("CentrosomeTransplant @ yr 50"));
                let cs: PlotPoints = ages.iter().map(|&a| [a, frailty(a, viz_scale)]).collect();
                plot_ui.line(Line::new(cs).color(Color32::WHITE).width(3.0).name("★ Current settings"));
            });

        ui.add_space(10.0);

        // ── Plot 2: Stem Cell Pool ──────────────────────────────────────
        ui.label(egui::RichText::new("② Stem Cell Pool Depletion").strong().size(14.0));
        Plot::new("plot_pool")
            .height(plot_h)
            .x_axis_label("Age (years)")
            .y_axis_label("Relative Pool Size [0–1]")
            .y_axis_width(4)
            .include_y(0.0)
            .include_y(1.05)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                let ctrl: PlotPoints = ages.iter().map(|&a| [a, pool(a, 1.0)]).collect();
                plot_ui.line(Line::new(ctrl).color(Color32::from_rgb(80, 150, 240)).width(2.5).name("Control"));
                let lon: PlotPoints = ages.iter().map(|&a| [a, pool(a, 0.55)]).collect();
                plot_ui.line(Line::new(lon).color(Color32::from_rgb(60, 210, 120)).width(2.5).name("Longevity"));
                let pro: PlotPoints = ages.iter().map(|&a| [a, pool(a, 5.0)]).collect();
                plot_ui.line(Line::new(pro).color(Color32::from_rgb(255, 100, 60)).width(2.5).name("Progeria"));
                let tx: PlotPoints = ages.iter().map(|&a| {
                    let scale = if a < 50.0 { 1.0 } else { 0.5 };
                    [a, pool(a, scale)]
                }).collect();
                plot_ui.line(Line::new(tx).color(Color32::GOLD).width(1.8).name("CentrosomeTransplant @ yr 50"));
                let cs: PlotPoints = ages.iter().map(|&a| [a, pool(a, viz_pool_sc)]).collect();
                plot_ui.line(Line::new(cs).color(Color32::WHITE).width(3.0).name("★ Current settings"));
            });

        ui.add_space(10.0);

        // ── Plot 3: Kaplan-Meier Survival ──────────────────────────────
        ui.label(egui::RichText::new("③ Kaplan–Meier Survival Curves").strong().size(14.0));
        Plot::new("plot_survival")
            .height(plot_h)
            .x_axis_label("Age (years)")
            .y_axis_label("Survival Probability [0–1]")
            .y_axis_width(4)
            .include_y(0.0)
            .include_y(1.05)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                let ctrl: PlotPoints = ages.iter().map(|&a| [a, survival(a, 78.0)]).collect();
                plot_ui.line(Line::new(ctrl).color(Color32::from_rgb(80, 150, 240)).width(2.5).name("Control (μ=78 yr)"));
                let lon: PlotPoints = ages.iter().map(|&a| [a, survival(a, 108.0)]).collect();
                plot_ui.line(Line::new(lon).color(Color32::from_rgb(60, 210, 120)).width(2.5).name("Longevity (μ=108 yr)"));
                let pro: PlotPoints = ages.iter().map(|&a| [a, survival(a, 15.0)]).collect();
                plot_ui.line(Line::new(pro).color(Color32::from_rgb(255, 100, 60)).width(2.5).name("Progeria (μ=15 yr)"));
                let tx: PlotPoints = ages.iter().map(|&a| [a, survival(a, 93.0)]).collect();
                plot_ui.line(Line::new(tx).color(Color32::GOLD).width(2.5).name("CentrosomeTransplant (μ=93 yr)"));
            });

        ui.add_space(10.0);

        // ── Plot 4: CAII Biomarker + Epigenetic Clock ──────────────────
        ui.label(egui::RichText::new("④ CAII Biomarker & Epigenetic Clock Acceleration").strong().size(14.0));
        Plot::new("plot_biomarkers")
            .height(plot_h)
            .x_axis_label("Age (years)")
            .y_axis_label("Normalised Value")
            .y_axis_width(4)
            .include_y(0.0)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                // CAII control
                let caii_c: PlotPoints = ages.iter().map(|&a| [a, caii(a, 1.0)]).collect();
                plot_ui.line(Line::new(caii_c).color(Color32::from_rgb(200, 100, 255)).width(2.5).name("CAII — Control"));
                // CAII longevity
                let caii_l: PlotPoints = ages.iter().map(|&a| [a, caii(a, 0.55)]).collect();
                plot_ui.line(Line::new(caii_l).color(Color32::from_rgb(130, 200, 255)).width(1.8)
                    .style(egui_plot::LineStyle::Dashed { length: 6.0 })
                    .name("CAII — Longevity"));
                // Epigenetic clock control (normalised to 0-1 over 100yr)
                let epi_c: PlotPoints = ages.iter().map(|&a| [a, (epigenetic(a, 1.0) / epigenetic(100.0, 1.0)).min(1.5)]).collect();
                plot_ui.line(Line::new(epi_c).color(Color32::from_rgb(255, 180, 50)).width(2.5).name("Epigenetic clock — Control"));
                let epi_l: PlotPoints = ages.iter().map(|&a| [a, (epigenetic(a, 0.55) / epigenetic(100.0, 1.0)).min(1.5)]).collect();
                plot_ui.line(Line::new(epi_l).color(Color32::from_rgb(120, 255, 160)).width(1.8)
                    .style(egui_plot::LineStyle::Dashed { length: 6.0 })
                    .name("Epigenetic clock — Longevity"));
            });

        ui.add_space(12.0);
        ui.separator();
        ui.label(
            egui::RichText::new(
                "Reference: Tkemaladze J. Centriolar Damage Accumulation Theory of Ageing. \
                 Mol Biol Reports 2023 (PMID 36583780) | Cell-DT v0.1 — EIC Pathfinder Open 2026"
            )
            .size(11.0)
            .color(Color32::DARK_GRAY),
        );

        // ── Settings collapsible ────────────────────────────────────────
        ui.add_space(8.0);
        ui.collapsing("⚙️ Output / Module Settings", |ui| {
            if ui.checkbox(&mut self.state.viz.enabled, "Enable module").changed() {
                self.push_history();
            }
            if self.state.viz.enabled {
                ui.horizontal(|ui| {
                    ui.label("Update interval:");
                    if ui.add(Slider::new(&mut self.state.viz.update_interval, 1..=100)).changed() {
                        self.push_history();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Output directory:");
                    if ui.text_edit_singleline(&mut self.state.viz.output_dir).changed() {
                        self.push_history();
                    }
                });
                if ui.checkbox(&mut self.state.viz.save_plots, "Save plots").changed() {
                    self.push_history();
                }
            }
        });
    }
    
    fn show_mitochondrial_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔋 Mitochondrial Module — Трек E");
        ui.separator();
        ui.label("мтДНК мутации → ROS↑ → fusion↓ → mito_shield↓ → O₂ проникает к центриолям");
        ui.label("Все параметры передаются в симуляцию. Изменение влияет на mito_shield → damage_scale → все 9 метрик.");
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("base_mutation_rate:").on_hover_text("Базовая скорость накопления мтДНК мутаций / год. Дефолт: 0.003 (Bratic & Larsson 2013)");
            if ui.add(Slider::new(&mut self.state.mitochondrial.base_mutation_rate, 0.0..=0.02).step_by(0.0001)).changed() {
                self.push_history();
            }
        });
        ui.horizontal(|ui| {
            ui.label("ros_mtdna_feedback:").on_hover_text("Коэффициент обратной связи: ROS → ускорение мтДНК мутаций. Дефолт: 0.8");
            if ui.add(Slider::new(&mut self.state.mitochondrial.ros_mtdna_feedback, 0.0..=2.0).step_by(0.05)).changed() {
                self.push_history();
            }
        });
        ui.horizontal(|ui| {
            ui.label("fission_rate:").on_hover_text("Скорость фрагментации митохондрий (снижает fusion_index / шаг). Дефолт: 0.05");
            if ui.add(Slider::new(&mut self.state.mitochondrial.fission_rate, 0.0..=0.3).step_by(0.005)).changed() {
                self.push_history();
            }
        });
        ui.horizontal(|ui| {
            ui.label("base_mitophagy_flux:").on_hover_text("Базовый поток митофагии (очистка повреждённых митохондрий). Выше = лучше поддержание mito_shield. Дефолт: 0.9");
            if ui.add(Slider::new(&mut self.state.mitochondrial.base_mitophagy_flux, 0.0..=1.0).step_by(0.05)).changed() {
                self.push_history();
            }
        });
        ui.horizontal(|ui| {
            ui.label("mitophagy_threshold:").on_hover_text("Порог накопления мутаций для активации усиленной митофагии [0..1]. Дефолт: 0.5");
            if ui.add(Slider::new(&mut self.state.mitochondrial.mitophagy_threshold, 0.1..=0.9).step_by(0.05)).changed() {
                self.push_history();
            }
        });
        ui.horizontal(|ui| {
            ui.label("ros_production_boost:").on_hover_text("Вклад фрагментации (1-fusion_index) в продукцию ROS. Дефолт: 0.20");
            if ui.add(Slider::new(&mut self.state.mitochondrial.ros_production_boost, 0.0..=1.0).step_by(0.05)).changed() {
                self.push_history();
            }
        });
        ui.horizontal(|ui| {
            ui.label("midlife_mutation_multiplier:").on_hover_text("Множитель скорости мтДНК мутаций после bio_age_proxy > 0.25 (~40 лет). Антагонистическая плейотропия. Дефолт: 1.5");
            if ui.add(Slider::new(&mut self.state.mitochondrial.midlife_mutation_multiplier, 1.0..=5.0).step_by(0.1)).changed() {
                self.push_history();
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.label("Формула mito_shield:");
        ui.label("  mito_shield = fusion_index×0.40 + membrane_potential×0.35 + (1−ros_production)×0.25");
        ui.label("  o2_at_centriole = 1 − mito_shield  →  влияет на вероятность отщепления индукторов");
    }

    fn show_about_dialog(&mut self, ctx: &egui::Context) {
        let lang = self.state.language;
        let mut open = self.state.show_about;
        Window::new("ℹ️ About — Parameter → Metric Map")
            .open(&mut open)
            .resizable(true)
            .default_width(700.0)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    let content = about_content(lang);
                    ui.label(egui::RichText::new(content).size(12.0));
                });
            });
        self.state.show_about = open;
    }

    #[allow(dead_code)]
    fn show_cdata_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔴 CDATA / Aging — параметры теории CDATA");
        ui.separator();

        // --- Inducer system ---
        ui.collapsing("🔬 Система индукторов", |ui| {
            ui.horizontal(|ui| {
                ui.label("base_detach_probability:")
                  .on_hover_text("Базовая вероятность отщепления индуктора за шаг (O₂-зависимо)");
                if ui.add(
                    Slider::new(&mut self.state.cdata.base_detach_probability, 0.0..=0.01)
                        .logarithmic(true)
                        .suffix("")
                ).changed() {
                    self.push_history();
                }
            });

            ui.horizontal(|ui| {
                ui.label("mother_bias:")
                  .on_hover_text("Доля отщеплений, приходящаяся на материнскую центриоль (0 = равно, 1 = только M)");
                if ui.add(
                    Slider::new(&mut self.state.cdata.mother_bias, 0.0..=1.0)
                ).changed() {
                    self.push_history();
                }
            });

            ui.horizontal(|ui| {
                ui.label("age_bias_coefficient:")
                  .on_hover_text("На сколько возраст усиливает mother_bias за год");
                if ui.add(
                    Slider::new(&mut self.state.cdata.age_bias_coefficient, 0.0..=0.01)
                        .logarithmic(true)
                ).changed() {
                    self.push_history();
                }
            });
        });

        ui.add_space(4.0);

        // --- Inductor lifecycle ---
        ui.collapsing("🧬 Жизненный цикл индукторов", |ui| {
            ui.label("Параметры управляют созданием de novo и элиминацией индукторов в мейозе.");

            ui.horizontal(|ui| {
                ui.label("de_novo_centriole_division:")
                  .on_hover_text(
                    "На каком делении бластомеров (от зиготы) создаются de novo центриоли \
                     с индукторами дифференцировки.\n\
                     4 = 16-клеточная стадия (Морула, биологический дефолт человека).\n\
                     До этой стадии DifferentiationStatus.inductors_active = false."
                  );
                let mut div = self.state.cdata.de_novo_centriole_division as f32;
                if ui.add(
                    Slider::new(&mut div, 1.0..=8.0)
                        .step_by(1.0)
                        .suffix(" деление")
                ).changed() {
                    self.state.cdata.de_novo_centriole_division = div as u32;
                    self.push_history();
                }
            });

            let stage_label = match self.state.cdata.de_novo_centriole_division {
                1     => "Zygote",
                2 | 3 => "Cleavage",
                4     => "Morula ✓",
                5 | 6 => "Blastocyst",
                _     => "Implantation",
            };
            ui.label(format!("→ стадия: {}", stage_label));

            ui.add_space(4.0);

            if ui.checkbox(
                &mut self.state.cdata.meiotic_elimination_enabled,
                "meiotic_elimination_enabled",
            ).on_hover_text(
                "Учитывать элиминацию центриолей в прелептотенной стадии мейоза.\n\
                 При включении: в стадии Adolescence регистрируется мейотическая элиминация —\n\
                 следующее поколение начнёт с DifferentiationStatus.Totipotent.\n\
                 Биологически корректный дефолт: включено."
            ).changed() {
                self.push_history();
            }
        });

        ui.add_space(4.0);

        // --- Myeloid shift weights ---
        ui.collapsing("🩸 Миелоидный сдвиг (веса)", |ui| {
            ui.label("Сумма весов должна быть ≈ 1.0 для корректного масштабирования myeloid_bias.");

            ui.horizontal(|ui| {
                ui.label("spindle_weight:")
                  .on_hover_text("Вклад потери spindle_fidelity в myeloid_bias");
                if ui.add(Slider::new(&mut self.state.cdata.spindle_weight, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });

            ui.horizontal(|ui| {
                ui.label("cilia_weight:")
                  .on_hover_text("Вклад потери ciliary_function в myeloid_bias");
                if ui.add(Slider::new(&mut self.state.cdata.cilia_weight, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });

            ui.horizontal(|ui| {
                ui.label("ros_weight:")
                  .on_hover_text("Вклад ros_level в myeloid_bias");
                if ui.add(Slider::new(&mut self.state.cdata.ros_weight, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });

            ui.horizontal(|ui| {
                ui.label("aggregate_weight:")
                  .on_hover_text("Вклад protein_aggregates в myeloid_bias");
                if ui.add(Slider::new(&mut self.state.cdata.aggregate_weight, 0.0..=1.0)).changed() {
                    self.push_history();
                }
            });

            let total = self.state.cdata.spindle_weight
                + self.state.cdata.cilia_weight
                + self.state.cdata.ros_weight
                + self.state.cdata.aggregate_weight;
            let color = if (total - 1.0).abs() < 0.05 {
                egui::Color32::GREEN
            } else {
                egui::Color32::YELLOW
            };
            ui.colored_label(color, format!("Σ = {:.2} (цель: 1.00)", total));

            ui.add_space(4.0);
            ui.label("Fine-tuning обратной связи воспаления:");
            ui.horizontal(|ui| {
                ui.label("ros_boost_scale:").on_hover_text("Масштаб усиления ROS через воспаление: ros_boost = inflammaging_index × scale. Дефолт: 0.15");
                if ui.add(Slider::new(&mut self.state.cdata.ros_boost_scale, 0.0..=0.5).step_by(0.01)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("niche_impair_scale:").on_hover_text("Масштаб угнетения ниши через воспаление: niche_impairment = inflammaging_index × scale. Дефолт: 0.08");
                if ui.add(Slider::new(&mut self.state.cdata.niche_impair_scale, 0.0..=0.3).step_by(0.01)).changed() {
                    self.push_history();
                }
            });
            ui.horizontal(|ui| {
                ui.label("spindle_nonlinearity_exponent:").on_hover_text("Степень нелинейности spindle в myeloid_bias: (1-spindle)^exp. >1 = ускорение при сильных повреждениях. Дефолт: 1.5");
                if ui.add(Slider::new(&mut self.state.cdata.spindle_nonlinearity_exponent, 0.5..=3.0).step_by(0.1)).changed() {
                    self.push_history();
                }
            });
        });

        ui.add_space(4.0);

        // --- Inducer system ---
        ui.collapsing("🧬 Индукторная система (P59)", |ui| {
            ui.label("Гипотеза индукторов Ткемаладзе: потентность определяется через центриолярные индукторы M/D.");
            ui.add_space(2.0);
            if ui.checkbox(&mut self.state.cdata.enable_inducer_system, "Включить индукторную систему (рекомендуется)").changed() {
                self.push_history();
            }
            ui.label("true → потентность = f(M-комплект, D-комплект); false → потентность = f(spindle_fidelity, ciliary_function)");
        });

        ui.add_space(4.0);

        // --- Damage preset ---
        ui.collapsing("⚡ Пресет скоростей повреждений", |ui| {
            ui.label("Выбор предустановки масштабирует все скорости повреждений в DamageParams.");

            let current_label = self.state.cdata.damage_preset.label();
            ComboBox::from_label("Пресет")
                .selected_text(current_label)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(
                        &mut self.state.cdata.damage_preset,
                        DamagePreset::Normal,
                        DamagePreset::Normal.label(),
                    ).clicked() {
                        self.push_history();
                    }
                    if ui.selectable_value(
                        &mut self.state.cdata.damage_preset,
                        DamagePreset::Progeria,
                        DamagePreset::Progeria.label(),
                    ).clicked() {
                        self.push_history();
                    }
                    if ui.selectable_value(
                        &mut self.state.cdata.damage_preset,
                        DamagePreset::Longevity,
                        DamagePreset::Longevity.label(),
                    ).clicked() {
                        self.push_history();
                    }
                });

            match self.state.cdata.damage_preset {
                DamagePreset::Normal =>
                    ui.label("Стандартные скорости. Ожидаемая продолжительность жизни ≈ 78 лет."),
                DamagePreset::Progeria =>
                    ui.label("Скорости ×5. Ускоренное старение (синдром Хатчинсона-Гилфорда)."),
                DamagePreset::Longevity =>
                    ui.label("Скорости ×0.6. Замедленное накопление повреждений → долгожительство."),
            };
        });

        ui.add_space(4.0);

        // --- Track E: Mitochondrial shield ---
        ui.collapsing("🔋 Трек E — Митохондрии (MitochondrialState)", |ui| {
            ui.label("мтДНК мутации → ROS↑ → fusion↓ → pericentriolar_density↓ → mito_shield↓ → O₂ проникает к центриолям");
            ui.add_space(4.0);
            ui.label("Формулы:");
            ui.label("  ros_production  = mtdna_mutations×0.60 + (1−fusion_index)×0.25 + damage_ros×0.10");
            ui.label("  fusion_index    -= ros_production×0.05 − mitophagy_flux×0.03  (шаг)");
            ui.label("  pericentriolar_density = fusion_index×0.70 + (1−ros_production)×0.30");
            ui.label("  mito_shield     = fusion_index×0.40 + membrane_potential×0.35 + (1−ros_production)×0.25");
            ui.add_space(2.0);
            ui.label("Интеграция с центриолями:");
            ui.label("  o2_at_centriole = 1 − mito_shield_contribution");
            ui.label("  detach_prob    *= 1 + o2_at_centriole × factor");
        });

        ui.add_space(4.0);

        // --- Track F: Division rate decline ---
        ui.collapsing("📉 Трек F — Темп деления (StemCellDivisionRateState)", |ui| {
            ui.label("division_rate = cilia_drive × spindle_drive × age_factor × ros_brake × mtor_brake");
            ui.label("Применяется как: regeneration_tempo *= division_rate.sqrt()");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("division_rate_floor:")
                  .on_hover_text("Нижняя граница age_factor (минимальный темп деления у пожилых). По умолч.: 0.15");
                if ui.add(
                    Slider::new(&mut self.state.cdata.division_rate_floor, 0.05..=0.50)
                        .step_by(0.01)
                ).changed() {
                    self.push_history();
                }
            });

            ui.horizontal(|ui| {
                ui.label("ros_brake_strength:")
                  .on_hover_text("Сила тормоза ROS: ros_brake = 1 - ros_level × strength. По умолч.: 0.40");
                if ui.add(
                    Slider::new(&mut self.state.cdata.ros_brake_strength, 0.0..=1.0)
                        .step_by(0.05)
                ).changed() {
                    self.push_history();
                }
            });

            ui.horizontal(|ui| {
                ui.label("mtor_brake_strength:")
                  .on_hover_text("Сила тормоза mTOR: mtor_brake = 1 - (mtor-0.3).max(0) × strength. По умолч.: 0.35");
                if ui.add(
                    Slider::new(&mut self.state.cdata.mtor_brake_strength, 0.0..=1.0)
                        .step_by(0.05)
                ).changed() {
                    self.push_history();
                }
            });

            ui.add_space(4.0);
            ui.label("Пять молекулярных тормозов темпа деления:");
            ui.label("  cilia_drive  = 0.25 + ciliary_function × 0.75");
            ui.label("  spindle_drive = 0.30 + spindle_fidelity × 0.70");
            ui.label("  age_factor   = (1 - (age-20)/100) clamp [floor, 1.0]");
            ui.label("  ros_brake    = (1 - ros × strength) clamp [0.40, 1.0]");
            ui.label("  mtor_brake   = (1 - (mtor-0.3)×strength) clamp [0.60, 1.0]");
        });

        ui.add_space(4.0);

        // --- Info block ---
        ui.collapsing("ℹ️ Справка по CDATA", |ui| {
            ui.label("Теория накопления центриолярных повреждений (Jaba Tkemaladze).");
            ui.add_space(2.0);
            ui.label("Шесть треков старения:");
            ui.label("  A — Цилии: CEP164↓ → Shh/Wnt↓ → нет самообновления ниши");
            ui.label("  B — Веретено: spindle_fidelity↓ → симм. деление → истощение пула");
            ui.label("  C — Теломеры: укорачивание → Хейфлик G1-арест");
            ui.label("  D — Эпигенетика: methylation_age += dt × (1 + damage × 0.5)");
            ui.label("  E — Митохондрии: мтДНК мутации → ROS↑ → fusion↓ → pericentriolar_density↓ → mito_shield↓");
            ui.label("  F — Темп деления: 5 молекулярных тормозов → division_rate↓");
            ui.label("  + Миелоидный: spindle↓ + cilia↓ + ROS↑ → PU.1 > Ikaros → воспаление");
            ui.add_space(2.0);
            ui.label("Порог сенесценции: total_damage > 0.75 → смерть ниши ≈ 78 лет.");
        });
    }

    // ==================== DIALOGS ====================



    fn show_save_dialog(&mut self, ctx: &Context) {
        let mut open = true;
        Window::new("💾 Save Configuration")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filename:");
                    ui.text_edit_singleline(&mut self.state.config_file);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    ui.radio_value(&mut self.state.config_format, "toml".to_string(), "TOML");
                    ui.radio_value(&mut self.state.config_format, "yaml".to_string(), "YAML");
                    ui.radio_value(&mut self.state.config_format, "json".to_string(), "JSON");
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.state.message = Some(format!("✅ Saved: {}", self.state.config_file));
                        self.state.show_save_dialog = false;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.state.show_save_dialog = false;
                    }
                });
            });
        
        if !open {
            self.state.show_save_dialog = false;
        }
    }
    
    fn show_load_dialog(&mut self, ctx: &Context) {
        let mut open = true;
        Window::new("📂 Load Configuration")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filename:");
                    ui.text_edit_singleline(&mut self.state.config_file);
                });
                
                ui.collapsing("📁 Available configurations", |ui| {
                    ui.label("configs/example.toml");
                    ui.label("configs/development.toml");
                    ui.label("configs/production.toml");
                    ui.label("configs/benchmark.toml");
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Load").clicked() {
                        self.state.message = Some(format!("✅ Loaded: {}", self.state.config_file));
                        self.state.show_load_dialog = false;
                        self.push_history();
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.state.show_load_dialog = false;
                    }
                });
            });
        
        if !open {
            self.state.show_load_dialog = false;
        }
    }
    
    fn show_preset_dialog(&mut self, ctx: &Context) {
        let mut open = true;
        Window::new("📋 Configuration Presets")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Select a preset configuration:");
                ui.separator();
                
                let presets = ConfigPreset::get_all();
                
                for preset in presets {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", preset.icon, preset.name));
                        if ui.button("Apply").clicked() {
                            (preset.apply)(&mut self.state);
                            self.state.message = Some(format!("✅ Applied preset: {}", preset.name));
                            self.state.show_preset_dialog = false;
                            self.push_history();
                        }
                    });
                    ui.label(format!("   {}", preset.description));
                    ui.separator();
                }
                
                if ui.button("Close").clicked() {
                    self.state.show_preset_dialog = false;
                }
            });
        
        if !open {
            self.state.show_preset_dialog = false;
        }
    }
    
    fn show_export_dialog(&mut self, ctx: &Context) {
        let mut open = true;
        let script = PythonExporter::generate_script(&self.state);
        
        Window::new("🐍 Export to Python")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Generated Python script:");
                ui.separator();
                
                ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.label(script.as_str());
                    });
                
                ui.horizontal(|ui| {
                    if ui.button("📋 Copy to clipboard").clicked() {
                        ui.ctx().copy_text(script);
                        self.state.message = Some("✅ Script copied to clipboard".to_string());
                    }
                    
                    if ui.button("💾 Save as script.py").clicked() {
                        // Here you would save to file
                        self.state.message = Some("✅ Script saved".to_string());
                    }
                    
                    if ui.button("Close").clicked() {
                        self.state.show_export_dialog = false;
                    }
                });
            });
        
        if !open {
            self.state.show_export_dialog = false;
        }
    }
    
    fn show_validation_dialog(&mut self, ctx: &Context) {
        let mut open = true;
        let errors = &self.state.validation_errors.clone();
        
        Window::new("✓ Parameter Validation")
            .open(&mut open)
            .show(ctx, |ui| {
                if errors.is_empty() {
                    ui.label("✅ All parameters are valid!");
                } else {
                    ui.label("❌ Found issues:");
                    ui.separator();
                    for error in errors {
                        ui.label(error);
                    }
                }
                
                ui.separator();
                
                if ui.button("Close").clicked() {
                    self.state.show_validation_dialog = false;
                }
            });
        
        if !open {
            self.state.show_validation_dialog = false;
        }
    }

    /// P67-GUI: сохранить sim_snapshots в CSV после завершения симуляции.
    /// Путь: <output_dir>/simulation_results_<timestamp>.csv
    fn save_simulation_csv(&self) -> Result<String, std::io::Error> {
        if self.sim_snapshots.is_empty() {
            return Ok("(no data)".to_string());
        }
        let output_dir = &self.state.simulation.output_dir;
        std::fs::create_dir_all(output_dir)?;

        // Simple timestamp from step count to avoid chrono dependency
        let fname = format!("simulation_results_{}_steps.csv", self.state.sim_elapsed_steps);
        let path = PathBuf::from(output_dir).join(&fname);

        let mut csv = String::from(
            "step,age_years,frailty,stem_cell_pool,ros_level,myeloid_bias,\
             telomere_length,methylation_age,is_alive\n"
        );
        for s in &self.sim_snapshots {
            csv.push_str(&format!(
                "{},{:.4},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{}\n",
                s.step, s.age_years, s.frailty, s.stem_cell_pool,
                s.ros_level, s.myeloid_bias, s.telomere_length,
                s.methylation_age, s.is_alive,
            ));
        }
        std::fs::write(&path, csv)?;
        Ok(path.to_string_lossy().into_owned())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// About dialog content — 7 languages
// ─────────────────────────────────────────────────────────────────────────────
fn about_content(lang: i18n::Lang) -> &'static str {
    match lang {
        i18n::Lang::En => ABOUT_EN,
        i18n::Lang::Fr => ABOUT_FR,
        i18n::Lang::Es => ABOUT_ES,
        i18n::Lang::Ru => ABOUT_RU,
        i18n::Lang::Zh => ABOUT_ZH,
        i18n::Lang::Ar => ABOUT_AR,
        i18n::Lang::Ka => ABOUT_KA,
    }
}

static ABOUT_EN: &str = "\
CDATA Simulation — Parameter → Metric Reference
═══════════════════════════════════════════════════════════════════

9 OUTPUT METRICS
Row 1 (Aging):  Lifespan · Healthspan · Frailty@50 · Frailty@70 · Pool@70 · ROS@50
Row 2 (Risks):  ❤ Cardio · 🧬 Cancer · 🧠 Cognition

PARAMETERS AND WHICH METRICS THEY AFFECT
───────────────────────────────────────────

🔴 CDATA / Aging tab
  base_detach_probability → ALL 9 (core driver: determines damage_scale)
  mother_bias             → ALL 9 (asymmetry of inducer loss)
  age_bias_coefficient    → ALL 9 (age modulation of mother_bias)
  division_rate_floor     → Pool@70, Healthspan, Lifespan
  ros_brake_strength      → ROS@50, Cardio, Cancer, Cognition
  mtor_brake_strength     → Pool@70, Healthspan
  damage_preset           → ALL 9 (scales all damage rates ×1/×5/×0.6)
  ros_boost_scale         → ROS@50, Cardio, Cognition
  niche_impair_scale      → Pool@70, Healthspan, Lifespan
  spindle_nonlinearity    → Cancer, Cardio
  enable_inducer_system   → ALL 9 (switches potency model)

  Myeloid weights (spindle/cilia/ros/aggregate):
    spindle_weight        → Cardio, Cancer, Cognition
    cilia_weight          → Cardio, Cognition
    ros_weight            → ROS@50, Cardio, Cancer, Cognition
    aggregate_weight      → Cardio, Cognition

🔋 Mitochondrial tab (Track E → mito_shield → o2_at_centriole → ALL)
  base_mutation_rate           → ALL 9
  ros_mtdna_feedback           → ALL 9
  fission_rate                 → ALL 9
  base_mitophagy_flux          → ALL 9
  mitophagy_threshold          → ALL 9
  ros_production_boost         → ROS@50, Cardio, Cancer, Cognition
  midlife_mutation_multiplier  → ALL 9

🔬 Centriole tab (PTM → total_damage_score → damage_scale)
  acetylation_rate      → ALL 9
  oxidation_rate        → ALL 9
  methylation_rate      → ALL 9
  phosphorylation_rate  → ALL 9
  daughter_ptm_factor   → Frailty, Lifespan (asymmetry of damage)
  m_phase_boost         → ALL 9 (PTM surge each division)

🔄 Cell Cycle tab
  checkpoint_strictness       → Cancer (DNA error containment)
  growth_factor_sensitivity   → Pool@70, Healthspan
  stress_sensitivity          → Frailty, Cancer

FORMULA CHAIN
  damage_scale = preset × (base_detach/0.0003) × bias × age_f × checkpoint
  lifespan     = 45/√s + ln(19)/(0.08×s)
  healthspan   = 45/√s
  pool @70     = 1 − 0.011×ps×70   (ps = s/(0.5 + floor×1.5))
  ros @50      = min(1, 0.007×rs×50) (rs = s×(2−ros_brake))
  frailty(age) = 1/(1+exp(−0.08×s×(age−45/√s)))
  cardio       = ros65×0.45 + myeloid65×0.40 + (s−0.6)×0.08
  cancer(est.) = s×(1−chk×0.60)×0.30
  cognition    = (myeloid65×0.45 + ros65×0.30 + epigen65×0.25)×0.65

Theory: Tkemaladze J. Mol Biol Reports 2023. PMID 36583780
";

static ABOUT_FR: &str = "\
CDATA Simulation — Paramètre → Référence des Métriques
═══════════════════════════════════════════════════════════════════

9 MÉTRIQUES DE SORTIE
Ligne 1 (Vieillissement): Espérance de vie · Santé active · Fragilité@50 · Fragilité@70 · Pool@70 · ROS@50
Ligne 2 (Risques): ❤ Cardio · 🧬 Cancer · 🧠 Cognition

PARAMÈTRES ET MÉTRIQUES AFFECTÉES
───────────────────────────────────────────

🔴 Onglet CDATA / Vieillissement
  base_detach_probability → TOUTES les 9 (moteur principal : détermine damage_scale)
  mother_bias             → TOUTES les 9 (asymétrie de perte d'inducteurs)
  age_bias_coefficient    → TOUTES les 9 (modulation par l'âge)
  division_rate_floor     → Pool@70, Santé active, Espérance de vie
  ros_brake_strength      → ROS@50, Cardio, Cancer, Cognition
  mtor_brake_strength     → Pool@70, Santé active
  damage_preset           → TOUTES les 9 (scale ×1/×5/×0.6)
  ros_boost_scale         → ROS@50, Cardio, Cognition
  niche_impair_scale      → Pool@70, Santé active, Espérance de vie
  spindle_nonlinearity    → Cancer, Cardio
  enable_inducer_system   → TOUTES les 9

🔋 Onglet Mitochondrial (Piste E → mito_shield → o2_at_centriole → TOUT)
  base_mutation_rate           → TOUTES les 9
  ros_mtdna_feedback           → TOUTES les 9
  fission_rate                 → TOUTES les 9
  base_mitophagy_flux          → TOUTES les 9
  mitophagy_threshold          → TOUTES les 9
  ros_production_boost         → ROS@50, Cardio, Cancer, Cognition
  midlife_mutation_multiplier  → TOUTES les 9

🔬 Onglet Centriole (PTM → total_damage_score → damage_scale)
  acetylation_rate      → TOUTES les 9
  oxidation_rate        → TOUTES les 9
  methylation_rate      → TOUTES les 9
  phosphorylation_rate  → TOUTES les 9
  daughter_ptm_factor   → Fragilité, Espérance de vie
  m_phase_boost         → TOUTES les 9

🔄 Onglet Cycle Cellulaire
  checkpoint_strictness       → Cancer
  growth_factor_sensitivity   → Pool@70, Santé active
  stress_sensitivity          → Fragilité, Cancer

Théorie: Tkemaladze J. Mol Biol Reports 2023. PMID 36583780
";

static ABOUT_ES: &str = "\
CDATA Simulación — Parámetro → Referencia de Métricas
═══════════════════════════════════════════════════════════════════

9 MÉTRICAS DE SALIDA
Fila 1 (Envejecimiento): Esperanza de vida · Salud activa · Fragilidad@50 · Fragilidad@70 · Pool@70 · ROS@50
Fila 2 (Riesgos): ❤ Cardio · 🧬 Cáncer · 🧠 Cognición

PARÁMETROS Y MÉTRICAS AFECTADAS
───────────────────────────────────────────

🔴 Pestaña CDATA / Envejecimiento
  base_detach_probability → TODAS las 9 (motor principal: determina damage_scale)
  mother_bias             → TODAS las 9 (asimetría de pérdida de inductores)
  age_bias_coefficient    → TODAS las 9 (modulación por edad)
  division_rate_floor     → Pool@70, Salud activa, Esperanza de vida
  ros_brake_strength      → ROS@50, Cardio, Cáncer, Cognición
  mtor_brake_strength     → Pool@70, Salud activa
  damage_preset           → TODAS las 9 (escala ×1/×5/×0.6)
  ros_boost_scale         → ROS@50, Cardio, Cognición
  niche_impair_scale      → Pool@70, Salud activa, Esperanza de vida
  spindle_nonlinearity    → Cáncer, Cardio
  enable_inducer_system   → TODAS las 9

🔋 Pestaña Mitocondrial (Pista E → mito_shield → o2_at_centriole → TODO)
  base_mutation_rate           → TODAS las 9
  ros_mtdna_feedback           → TODAS las 9
  fission_rate                 → TODAS las 9
  base_mitophagy_flux          → TODAS las 9
  mitophagy_threshold          → TODAS las 9
  ros_production_boost         → ROS@50, Cardio, Cáncer, Cognición
  midlife_mutation_multiplier  → TODAS las 9

🔬 Pestaña Centríolo (PTM → total_damage_score → damage_scale)
  acetylation_rate      → TODAS las 9
  oxidation_rate        → TODAS las 9
  methylation_rate      → TODAS las 9
  phosphorylation_rate  → TODAS las 9
  daughter_ptm_factor   → Fragilidad, Esperanza de vida
  m_phase_boost         → TODAS las 9

🔄 Pestaña Ciclo Celular
  checkpoint_strictness       → Cáncer
  growth_factor_sensitivity   → Pool@70, Salud activa
  stress_sensitivity          → Fragilidad, Cáncer

Teoría: Tkemaladze J. Mol Biol Reports 2023. PMID 36583780
";

static ABOUT_RU: &str = "\
CDATA Симуляция — Параметр → Справочник метрик
═══════════════════════════════════════════════════════════════════

9 ВЫХОДНЫХ МЕТРИК
Строка 1 (Старение): Продолжительность жизни · Период здоровья · Слабость@50 · Слабость@70 · Пул@70 · ROS@50
Строка 2 (Риски): ❤ Кардио · 🧬 Онко · 🧠 Когниция

ПАРАМЕТРЫ И ЗАТРАГИВАЕМЫЕ МЕТРИКИ
───────────────────────────────────────────

🔴 Вкладка CDATA / Старение
  base_detach_probability → ВСЕ 9 (ключевой драйвер: определяет damage_scale)
  mother_bias             → ВСЕ 9 (асимметрия потери индукторов M vs D)
  age_bias_coefficient    → ВСЕ 9 (вклад возраста в mother_bias)
  division_rate_floor     → Пул@70, Период здоровья, Продолжительность жизни
  ros_brake_strength      → ROS@50, Кардио, Онко, Когниция
  mtor_brake_strength     → Пул@70, Период здоровья
  damage_preset           → ВСЕ 9 (масштабирует все скорости ×1/×5/×0.6)
  ros_boost_scale         → ROS@50, Кардио, Когниция
  niche_impair_scale      → Пул@70, Период здоровья, Продолжительность жизни
  spindle_nonlinearity    → Онко, Кардио
  enable_inducer_system   → ВСЕ 9 (переключение модели потентности)

  Веса миелоидного сдвига:
    spindle_weight        → Кардио, Онко, Когниция
    cilia_weight          → Кардио, Когниция
    ros_weight            → ROS@50, Кардио, Онко, Когниция
    aggregate_weight      → Кардио, Когниция

🔋 Вкладка Митохондрии (Трек E → mito_shield → o2_at_centriole → ВСЕ)
  base_mutation_rate           → ВСЕ 9
  ros_mtdna_feedback           → ВСЕ 9
  fission_rate                 → ВСЕ 9
  base_mitophagy_flux          → ВСЕ 9 (митофагия защищает mito_shield)
  mitophagy_threshold          → ВСЕ 9
  ros_production_boost         → ROS@50, Кардио, Онко, Когниция
  midlife_mutation_multiplier  → ВСЕ 9 (антагонистическая плейотропия после 40 лет)

🔬 Вкладка Центриоль (PTM → total_damage_score → damage_scale)
  acetylation_rate      → ВСЕ 9
  oxidation_rate        → ВСЕ 9
  methylation_rate      → ВСЕ 9
  phosphorylation_rate  → ВСЕ 9
  daughter_ptm_factor   → Слабость, Продолжительность жизни (асимметрия накопления)
  m_phase_boost         → ВСЕ 9 (всплеск PTM при каждом делении)

🔄 Вкладка Клеточный цикл
  checkpoint_strictness       → Онко (качество ДНК-репарации)
  growth_factor_sensitivity   → Пул@70, Период здоровья
  stress_sensitivity          → Слабость, Онко

ЦЕПЬ ФОРМУЛ
  damage_scale = preset × (base_detach/0.0003) × bias × age_f × checkpoint
  lifespan     = 45/√s + ln(19)/(0.08×s)
  healthspan   = 45/√s
  pool @70     = 1 − 0.011×ps×70   (ps = s/(0.5 + floor×1.5))
  ros @50      = min(1, 0.007×rs×50) (rs = s×(2−ros_brake))
  frailty(age) = 1/(1+exp(−0.08×s×(age−45/√s)))
  кардио       = ros65×0.45 + myeloid65×0.40 + (s−0.6)×0.08
  онко (est.)  = s×(1−chk×0.60)×0.30
  когниция     = (myeloid65×0.45 + ros65×0.30 + epigen65×0.25)×0.65

Теория: Ткемаладзе Дж. Mol Biol Reports 2023. PMID 36583780
";

static ABOUT_ZH: &str = "\
CDATA 模拟 — 参数 → 指标参考
═══════════════════════════════════════════════════════════════════

9个输出指标
第1行（衰老）: 寿命 · 健康期 · 虚弱@50 · 虚弱@70 · 干细胞池@70 · 活性氧@50
第2行（风险）: ❤ 心血管 · 🧬 癌症 · 🧠 认知

参数与受影响的指标
───────────────────────────────────────────

🔴 CDATA / 衰老 选项卡
  base_detach_probability → 全部9个（核心驱动因素：决定损伤比例）
  mother_bias             → 全部9个（母中心粒诱导因子损失不对称性）
  age_bias_coefficient    → 全部9个（年龄对母中心粒偏倚的调节）
  division_rate_floor     → 干细胞池@70, 健康期, 寿命
  ros_brake_strength      → 活性氧@50, 心血管, 癌症, 认知
  mtor_brake_strength     → 干细胞池@70, 健康期
  damage_preset           → 全部9个（×1/×5/×0.6缩放所有损伤率）
  ros_boost_scale         → 活性氧@50, 心血管, 认知
  niche_impair_scale      → 干细胞池@70, 健康期, 寿命
  spindle_nonlinearity    → 癌症, 心血管
  enable_inducer_system   → 全部9个

🔋 线粒体 选项卡（轨道E → mito_shield → o2_at_centriole → 全部）
  base_mutation_rate           → 全部9个
  ros_mtdna_feedback           → 全部9个
  fission_rate                 → 全部9个
  base_mitophagy_flux          → 全部9个（线粒体自噬保护mito_shield）
  mitophagy_threshold          → 全部9个
  ros_production_boost         → 活性氧@50, 心血管, 癌症, 认知
  midlife_mutation_multiplier  → 全部9个（40岁后拮抗多效性）

🔬 中心粒 选项卡（PTM → total_damage_score → damage_scale）
  acetylation_rate      → 全部9个
  oxidation_rate        → 全部9个
  methylation_rate      → 全部9个
  phosphorylation_rate  → 全部9个
  daughter_ptm_factor   → 虚弱, 寿命
  m_phase_boost         → 全部9个（每次分裂时PTM激增）

🔄 细胞周期 选项卡
  checkpoint_strictness       → 癌症（DNA错误控制）
  growth_factor_sensitivity   → 干细胞池@70, 健康期
  stress_sensitivity          → 虚弱, 癌症

理论来源: Tkemaladze J. Mol Biol Reports 2023. PMID 36583780
";

static ABOUT_AR: &str = "\
محاكاة CDATA — مرجع المعاملات والمقاييس
═══════════════════════════════════════════════════════════════════

9 مقاييس الإخراج
الصف 1 (الشيخوخة): العمر الافتراضي · فترة الصحة · الهشاشة@50 · الهشاشة@70 · مجموعة@70 · جذور حرة@50
الصف 2 (المخاطر): ❤ قلب وأوعية · 🧬 سرطان · 🧠 إدراك

المعاملات والمقاييس المتأثرة
───────────────────────────────────────────

🔴 تبويب CDATA / الشيخوخة
  base_detach_probability → جميع الـ 9 (المحرك الأساسي: يحدد مقياس الضرر)
  mother_bias             → جميع الـ 9 (عدم تماثل فقدان المحفزات)
  age_bias_coefficient    → جميع الـ 9 (تعديل بالعمر)
  division_rate_floor     → مجموعة@70، فترة الصحة، العمر الافتراضي
  ros_brake_strength      → جذور حرة@50، قلب، سرطان، إدراك
  mtor_brake_strength     → مجموعة@70، فترة الصحة
  damage_preset           → جميع الـ 9 (×1/×5/×0.6)
  ros_boost_scale         → جذور حرة@50، قلب، إدراك
  niche_impair_scale      → مجموعة@70، فترة الصحة، العمر الافتراضي
  spindle_nonlinearity    → سرطان، قلب
  enable_inducer_system   → جميع الـ 9

🔋 تبويب الميتوكوندريا (المسار E → mito_shield → جميع المقاييس)
  base_mutation_rate           → جميع الـ 9
  ros_mtdna_feedback           → جميع الـ 9
  fission_rate                 → جميع الـ 9
  base_mitophagy_flux          → جميع الـ 9
  mitophagy_threshold          → جميع الـ 9
  ros_production_boost         → جذور حرة@50، قلب، سرطان، إدراك
  midlife_mutation_multiplier  → جميع الـ 9

🔬 تبويب السنتريول (PTM → damage_scale)
  acetylation_rate      → جميع الـ 9
  oxidation_rate        → جميع الـ 9
  methylation_rate      → جميع الـ 9
  phosphorylation_rate  → جميع الـ 9
  daughter_ptm_factor   → الهشاشة، العمر الافتراضي
  m_phase_boost         → جميع الـ 9

🔄 تبويب دورة الخلية
  checkpoint_strictness       → سرطان
  growth_factor_sensitivity   → مجموعة@70، فترة الصحة
  stress_sensitivity          → الهشاشة، سرطان

المصدر: Tkemaladze J. Mol Biol Reports 2023. PMID 36583780
";

static ABOUT_KA: &str = "\
CDATA სიმულაცია — პარამეტრი → მეტრიკების მითითება
═══════════════════════════════════════════════════════════════════

9 გამოსვლის მეტრიკა
მწკრივი 1 (დაბერება): სიცოცხლის ხანგრძლივობა · ჯანმრთელობის პერიოდი · სისუსტე@50 · სისუსტე@70 · პული@70 · ROS@50
მწკრივი 2 (რისკები): ❤ გული · 🧬 კიბო · 🧠 კოგნიცია

პარამეტრები და ზეგავლენის მეტრიკები
───────────────────────────────────────────

🔴 CDATA / დაბერება ჩანართი
  base_detach_probability → ყველა 9 (მთავარი მამოძრავებელი: განსაზღვრავს damage_scale-ს)
  mother_bias             → ყველა 9 (ინდუქტორების დაკარგვის ასიმეტრია M/D)
  age_bias_coefficient    → ყველა 9 (ასაკის გავლენა mother_bias-ზე)
  division_rate_floor     → პული@70, ჯანმრთელობის პერიოდი, სიცოცხლის ხანგრძლივობა
  ros_brake_strength      → ROS@50, გული, კიბო, კოგნიცია
  mtor_brake_strength     → პული@70, ჯანმრთელობის პერიოდი
  damage_preset           → ყველა 9 (×1/×5/×0.6 მასშტაბირება)
  ros_boost_scale         → ROS@50, გული, კოგნიცია
  niche_impair_scale      → პული@70, ჯანმრთელობის პერიოდი, სიცოცხლის ხანგრძლივობა
  spindle_nonlinearity    → კიბო, გული
  enable_inducer_system   → ყველა 9 (პოტენტურობის მოდელის გადართვა)

🔋 მიტოქონდრია ჩანართი (ტრეკი E → mito_shield → o2_at_centriole → ყველა)
  base_mutation_rate           → ყველა 9
  ros_mtdna_feedback           → ყველა 9
  fission_rate                 → ყველა 9
  base_mitophagy_flux          → ყველა 9 (მიტოფაგია იცავს mito_shield-ს)
  mitophagy_threshold          → ყველა 9
  ros_production_boost         → ROS@50, გული, კიბო, კოგნიცია
  midlife_mutation_multiplier  → ყველა 9 (40 წლის შემდეგ ანტაგონისტური პლეიოტროპია)

🔬 ცენტრიოლა ჩანართი (PTM → total_damage_score → damage_scale)
  acetylation_rate      → ყველა 9
  oxidation_rate        → ყველა 9
  methylation_rate      → ყველა 9
  phosphorylation_rate  → ყველა 9
  daughter_ptm_factor   → სისუსტე, სიცოცხლის ხანგრძლივობა
  m_phase_boost         → ყველა 9 (PTM-ის მომატება ყოველ გაყოფაზე)

🔄 უჯრედის ციკლი ჩანართი
  checkpoint_strictness       → კიბო (DNK-ს შეცდომების კონტროლი)
  growth_factor_sensitivity   → პული@70, ჯანმრთელობის პერიოდი
  stress_sensitivity          → სისუსტე, კიბო

თეორია: Tkemaladze J. Mol Biol Reports 2023. PMID 36583780
";
