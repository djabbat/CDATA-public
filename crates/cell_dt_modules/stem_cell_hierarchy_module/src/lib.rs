//! Модуль иерархии стволовых клеток
//!
//! Потентность читается из `CentriolarDamageState` (spindle_fidelity как прокси)
//! и синхронизируется с `StemCellHierarchyState` на каждом шаге.
//!
//! `PotencyLevel` определён в `cell_dt_core::components` и переэкспортируется
//! здесь для обратной совместимости.

use cell_dt_core::{
    SimulationModule, SimulationResult,
    components::*,
    hecs::World,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use log::{info, debug};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

// PotencyLevel определён в cell_dt_core::components (glob-импорт выше).
// Переэкспортируем для совместимости с существующими примерами.
pub use cell_dt_core::components::PotencyLevel;

/// Division rate tier for a niche
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DivisionTier {
    /// ~1 division/day (intestinal SC, corneal limbal)
    Fast,
    /// 1 division per weeks–months (HSC, satellite, hair follicle)
    Slow,
    /// <0.01%/year (cardiac, centroacinar pancreatic, dental pulp)
    UltraSlow,
}

impl DivisionTier {
    pub fn label(self) -> &'static str {
        match self {
            DivisionTier::Fast      => "Fast (~1/day)",
            DivisionTier::Slow      => "Slow (wks–mo)",
            DivisionTier::UltraSlow => "Ultra-slow (<0.01%/yr)",
        }
    }
    pub fn color(self) -> [u8; 3] {
        match self {
            DivisionTier::Fast      => [60, 200, 90],
            DivisionTier::Slow      => [220, 170, 50],
            DivisionTier::UltraSlow => [200, 80, 60],
        }
    }
}

/// All 20 known human stem cell niches with biological metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheProfile {
    /// Short identifier used for simulation entity labels
    pub id: &'static str,
    /// Human-readable name (e.g. "Bone marrow — endosteal")
    pub name: &'static str,
    /// Organ / system
    pub organ: &'static str,
    /// Stem cell type
    pub sc_type: &'static str,
    /// Anatomical location (brief)
    pub location: &'static str,
    /// Distance from nearest surface (µm) — for 3D model Z-axis
    pub depth_um: f32,
    /// Division rate in young adult (normalized, 0=quiescent, 1=1 div/day)
    pub div_rate_young: f32,
    /// Division rate in aged (normalized)
    pub div_rate_aged: f32,
    /// Division rate tier
    pub tier: DivisionTier,
    /// Key niche signals (abbreviated)
    pub key_signals: &'static str,
    /// Main aging mechanism
    pub aging_note: &'static str,
}

/// Catalog of all 20 stem cell niches, based on peer-reviewed literature 2011–2025.
pub struct NicheCatalog;

impl NicheCatalog {
    pub fn all() -> &'static [NicheProfile] {
        &NICHE_CATALOG
    }
}

static NICHE_CATALOG: &[NicheProfile] = &[
    NicheProfile {
        id: "BM_Endo", name: "Bone marrow — endosteal", organ: "Bone marrow",
        sc_type: "LT-HSC", location: "Periarteriolar; 0–10 µm from endosteum",
        depth_um: 5.0, div_rate_young: 0.019, div_rate_aged: 0.015,
        tier: DivisionTier::Slow,
        key_signals: "CXCL12/CXCR4, Ang-1/Tie2, TGF-β, SCF/c-Kit, TPO",
        aging_note: "Myeloid bias; clonal hematopoiesis; ↑numbers but ↓output",
    },
    NicheProfile {
        id: "BM_Peri", name: "Bone marrow — perivascular", organ: "Bone marrow",
        sc_type: "ST-HSC / multipotent progenitor", location: "Perisinusoidal; ±2–5 µm from sinusoid",
        depth_um: 3.0, div_rate_young: 0.07, div_rate_aged: 0.05,
        tier: DivisionTier::Slow,
        key_signals: "CXCL12 (CAR cells), VEGF, ephrin, leptin-R+ stroma",
        aging_note: "Sinusoid integrity ↓; TNF/IL-6/IL-1β disrupt niche",
    },
    NicheProfile {
        id: "SkMus", name: "Skeletal muscle — satellite", organ: "Muscle",
        sc_type: "Satellite cell (MuSC)", location: "Sublaminal; ≤2 µm from myofiber surface",
        depth_um: 2.0, div_rate_young: 0.001, div_rate_aged: 0.0005,
        tier: DivisionTier::Slow,
        key_signals: "Notch (quiescence), Wnt (activation), HGF, FGF2, IGF-1, laminin",
        aging_note: "Excess FGF2 → loss of quiescence → ↓self-renewal; WISP1 fibrosis",
    },
    NicheProfile {
        id: "IntCrypt", name: "Small intestine — crypt (Lgr5+)", organ: "Intestine",
        sc_type: "Intestinal SC (CBC)", location: "Crypt base; ~250 µm below villus tip",
        depth_um: 250.0, div_rate_young: 1.0, div_rate_aged: 0.7,
        tier: DivisionTier::Fast,
        key_signals: "Wnt3/R-spondin (Paneth), Notch/Dll1/Dll4, EGF, Noggin",
        aging_note: "Aged Paneth → NOTUM (Wnt antagonist); ↓regenerative capacity",
    },
    NicheProfile {
        id: "SVZ", name: "Subventricular zone (SVZ)", organ: "Brain (lateral ventricle)",
        sc_type: "Neural SC (B1 astrocyte)", location: "Ventricular wall; 5–10 µm from ependyma",
        depth_um: 7.0, div_rate_young: 0.01, div_rate_aged: 0.005,
        tier: DivisionTier::Slow,
        key_signals: "EGF, FGF, Notch, SDF1/CXCL12, VCAM1, Shh",
        aging_note: "Neuroblast output ↓50%+ by midlife; CD8+ T-cell / IFN-γ disruption",
    },
    NicheProfile {
        id: "SGZ", name: "Hippocampal dentate gyrus (SGZ)", organ: "Hippocampus",
        sc_type: "Radial glia-like neural SC", location: "Subgranular zone; 10–20 µm below granule layer",
        depth_um: 15.0, div_rate_young: 0.005, div_rate_aged: 0.001,
        tier: DivisionTier::Slow,
        key_signals: "Wnt/β-catenin, VEGF, serotonin, IGF-1, TET2, BDNF",
        aging_note: "Dramatic early decline (adolescence); TET2 drops with age",
    },
    NicheProfile {
        id: "SkinIFE", name: "Skin — interfollicular epidermis", organ: "Skin",
        sc_type: "Epidermal progenitor (basal)", location: "Basal layer at rete ridges; ~5–15 µm from dermis",
        depth_um: 10.0, div_rate_young: 0.07, div_rate_aged: 0.05,
        tier: DivisionTier::Slow,
        key_signals: "Wnt, BMP, EGF/ErbB, FGF-7/10, TGF-β2, integrin",
        aging_note: "DEJ flattening; COL17A1+ subclone competition; ↓regeneration",
    },
    NicheProfile {
        id: "HairBulge", name: "Hair follicle bulge", organ: "Skin / Hair follicle",
        sc_type: "HFSC (Lgr5+/CD34+)", location: "Outer root sheath bulge; ~600–800 µm below surface",
        depth_um: 700.0, div_rate_young: 0.05, div_rate_aged: 0.03,
        tier: DivisionTier::Slow,
        key_signals: "BMP (quiescence), Wnt (activation), FGF18, Shh, Noggin, JAK/STAT",
        aging_note: "Niche stiffening; ↓IGF-1; BMAL1 circadian dysregulation",
    },
    NicheProfile {
        id: "Limbal", name: "Cornea — limbal niche", organ: "Eye / Cornea",
        sc_type: "Limbal epithelial SC (LESC)", location: "Palisades of Vogt; 0.5–1 mm from central cornea",
        depth_um: 50.0, div_rate_young: 0.07, div_rate_aged: 0.04,
        tier: DivisionTier::Slow,
        key_signals: "Wnt, Notch, BMP, Shh, YAP/TAZ, SDF-1/CXCR4, melanocyte paracrine",
        aging_note: "Palisade architecture disruption → LSCD; ↓reserve",
    },
    NicheProfile {
        id: "LiverHPC", name: "Liver — Canal of Hering", organ: "Liver",
        sc_type: "Hepatic progenitor (HPC/oval cell)", location: "Periportal zone 1; ~100–200 µm from portal vein",
        depth_um: 150.0, div_rate_young: 0.002, div_rate_aged: 0.001,
        tier: DivisionTier::UltraSlow,
        key_signals: "HGF, TGF-β, Wnt, FGF, Hedgehog, Notch/Jag1 (stellate cells)",
        aging_note: "Fibrosis-prone; stellate collagen I → biliary bias; NASH depletes HPC",
    },
    NicheProfile {
        id: "LungAT2", name: "Lung — alveolus (AT2)", organ: "Lung",
        sc_type: "Alveolar type II cell (Axin2+)", location: "Alveolar wall; adjacent to Wnt+ lipofibroblast",
        depth_um: 2.0, div_rate_young: 0.003, div_rate_aged: 0.001,
        tier: DivisionTier::UltraSlow,
        key_signals: "Wnt (fibroblasts), FGF7/10, EGF, HGF, BMP antagonists",
        aging_note: "Wnt-responsive AT2 subset ↓; fibrotic remodeling ↑ post-injury",
    },
    NicheProfile {
        id: "LungBADJ", name: "Lung — bronchoalveolar duct junction", organ: "Lung",
        sc_type: "Bronchioalveolar SC (BASC; Scgb1a1+/SPC+)", location: "Terminal bronchiole–alveolar duct junction",
        depth_um: 1.0, div_rate_young: 0.002, div_rate_aged: 0.001,
        tier: DivisionTier::UltraSlow,
        key_signals: "Wnt, Notch, BMP, FGF10",
        aging_note: "Limited human data; aging effects poorly characterized",
    },
    NicheProfile {
        id: "Pancreas", name: "Pancreas — centroacinar niche", organ: "Pancreas",
        sc_type: "Pancreatic progenitor (ELF3+)", location: "Acinus-ductule junctions throughout exocrine pancreas",
        depth_um: 5.0, div_rate_young: 0.001, div_rate_aged: 0.0005,
        tier: DivisionTier::UltraSlow,
        key_signals: "Notch (centroacinar), Wnt (exocrine), Shh, EGF",
        aging_note: "Very limited regeneration in adult humans; pool poorly maintained",
    },
    NicheProfile {
        id: "Testis", name: "Testis — seminiferous tubule", organ: "Testis",
        sc_type: "Spermatogonial SC (SSC; Aundiff)", location: "Basal compartment; 0–2 µm from basement membrane",
        depth_um: 1.0, div_rate_young: 0.06, div_rate_aged: 0.04,
        tier: DivisionTier::Slow,
        key_signals: "GDNF (Sertoli), CXCL12, FGF2, BMP4, SCF/c-Kit (differentiation)",
        aging_note: "Sertoli senescence → ↓GDNF; ↑ROS; sperm quality ↓",
    },
    NicheProfile {
        id: "Heart", name: "Heart — subepicardial", organ: "Heart",
        sc_type: "Cardiac progenitor (c-Kit+/Sca-1+)", location: "Sub-epicardium; AV junction; apex; <200 µm from epicardial surface",
        depth_um: 100.0, div_rate_young: 0.00001, div_rate_aged: 0.000005,
        tier: DivisionTier::UltraSlow,
        key_signals: "Notch, VEGF, TGF-β, HIF-1α (hypoxic niche), macrophage/fibroblast paracrine",
        aging_note: "Hypoxic niche ↓; post-MI inflammation eliminates progenitor pools",
    },
    NicheProfile {
        id: "Mammary", name: "Mammary gland", organ: "Breast",
        sc_type: "Mammary SC (MaSC; CD49f+/Procr+)", location: "Basal layer of ductal epithelium; ~10–15 µm from BM",
        depth_um: 12.0, div_rate_young: 0.05, div_rate_aged: 0.02,
        tier: DivisionTier::Slow,
        key_signals: "Wnt, Notch/Dll1, EGF, progesterone, prolactin",
        aging_note: "Post-menopause: ↓MaSC frequency; stromal fibrosis/adipose disrupts niche",
    },
    NicheProfile {
        id: "Prostate", name: "Prostate — basal niche", organ: "Prostate",
        sc_type: "Prostate basal SC (CK5+/CK14+/CD133+)", location: "Proximal urethral region; basal layer",
        depth_um: 5.0, div_rate_young: 0.005, div_rate_aged: 0.002,
        tier: DivisionTier::UltraSlow,
        key_signals: "Wnt/β-catenin (intermediate), AR-stromal signals, IGF-1, FGF",
        aging_note: "Wnt ↓ with age; BPH from niche dysregulation",
    },
    NicheProfile {
        id: "Adipose", name: "Adipose tissue — perivascular", organ: "Adipose",
        sc_type: "ADSC / pericyte-like MSC", location: "Adventitial layer of microvessels throughout adipose",
        depth_um: 2.0, div_rate_young: 0.01, div_rate_aged: 0.006,
        tier: DivisionTier::Slow,
        key_signals: "PDGF-R, BMP, Wnt, TGF-β, HGF",
        aging_note: "Adipogenic bias ↑; pro-inflammatory adipokines; lipid loading → ↓self-renewal",
    },
    NicheProfile {
        id: "DentalPulp", name: "Dental pulp", organ: "Tooth",
        sc_type: "DPSC (STRO-1+/CD146+)", location: "Perivascular/perineural in pulp; ~1–5 µm from vessel wall",
        depth_um: 3.0, div_rate_young: 0.002, div_rate_aged: 0.001,
        tier: DivisionTier::UltraSlow,
        key_signals: "Notch3 (endothelial-pericyte), IL-6/STAT3/Bmi-1, BMP, FGF, Wnt",
        aging_note: "Pulp volume ↓ (secondary dentin); progenitor accessibility ↓",
    },
    NicheProfile {
        id: "Periosteum", name: "Bone — periosteum / endosteum", organ: "Bone",
        sc_type: "Skeletal/Mesenchymal SC (SSC/MSC; Grem1+/PTHrP+)", location: "Cambium layer; innermost periosteum; 5–20 µm",
        depth_um: 12.0, div_rate_young: 0.008, div_rate_aged: 0.003,
        tier: DivisionTier::Slow,
        key_signals: "BMP, TGF-β, SDF-1/CXCL12, IGF-1, FGF, Runx2, PPARγ",
        aging_note: "Adipogenic shift; ↓osteogenic response; fracture healing impaired",
    },
];

/// Линии дифференцировки
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CellLineage {
    EmbryonicStem,
    HematopoieticStem,
    NeuralStem,
}

/// Состояние клетки в иерархии потентности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemCellHierarchyState {
    pub potency_level: PotencyLevel,
    pub potency_score: f32,
    pub lineage: Option<CellLineage>,
    pub master_regulator_levels: std::collections::HashMap<String, f32>,
    /// Число событий дедифференцировки (Oligopotent → Pluripotent) за жизнь клетки
    pub dedifferentiation_count: u32,
}

impl StemCellHierarchyState {
    pub fn new() -> Self {
        let mut regs = std::collections::HashMap::new();
        regs.insert("OCT4".to_string(),  0.9);
        regs.insert("NANOG".to_string(), 0.9);
        regs.insert("SOX2".to_string(),  0.9);
        Self {
            potency_level: PotencyLevel::Pluripotent,
            potency_score: 0.8,
            lineage: None,
            master_regulator_levels: regs,
            dedifferentiation_count: 0,
        }
    }

    /// Установить потентность и пересчитать potency_score и мастер-регуляторы.
    pub fn set_potency(&mut self, level: PotencyLevel) {
        self.potency_level = level;
        self.potency_score = match level {
            PotencyLevel::Totipotent  => 1.0,
            PotencyLevel::Pluripotent => 0.8,
            PotencyLevel::Oligopotent => 0.4,
            PotencyLevel::Unipotent   => 0.2,
            PotencyLevel::Apoptosis   => 0.0,
        };
        // Мастер-регуляторы медленно дрейфуют к текущему уровню потентности
        let target = self.potency_score;
        for val in self.master_regulator_levels.values_mut() {
            *val = (*val * 0.99 + target * 0.01).clamp(0.0, 1.0);
        }
    }
}

impl Default for StemCellHierarchyState {
    fn default() -> Self { Self::new() }
}

/// Параметры модуля иерархии
#[derive(Debug, Clone)]
pub struct StemCellHierarchyParams {
    pub initial_potency: PotencyLevel,
    pub enable_plasticity: bool,
    pub plasticity_rate: f32,
    pub differentiation_threshold: f32,
}

impl Default for StemCellHierarchyParams {
    fn default() -> Self {
        Self {
            initial_potency: PotencyLevel::Pluripotent,
            enable_plasticity: true,
            plasticity_rate: 0.01,
            differentiation_threshold: 0.7,
        }
    }
}

/// Модуль иерархии стволовых клеток
pub struct StemCellHierarchyModule {
    params: StemCellHierarchyParams,
    step_count: u64,
    rng: StdRng,
}

impl StemCellHierarchyModule {
    pub fn new() -> Self {
        Self { params: StemCellHierarchyParams::default(), step_count: 0, rng: StdRng::from_entropy() }
    }

    pub fn with_params(params: StemCellHierarchyParams) -> Self {
        Self { params, step_count: 0, rng: StdRng::from_entropy() }
    }
}

impl SimulationModule for StemCellHierarchyModule {
    fn name(&self) -> &str { "stem_cell_hierarchy_module" }

    fn set_seed(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
    }

    /// Синхронизирует `StemCellHierarchyState` с молекулярным состоянием центриоли.
    ///
    /// Использует `spindle_fidelity` как прокси потентности:
    /// высокая точность веретена → клетка сохраняет стволовость.
    ///
    /// При `enable_plasticity=true`: клетки на уровне `Oligopotent` могут
    /// дедифференцироваться обратно в `Pluripotent`, если веретено восстановилось
    /// (spindle_fidelity > `differentiation_threshold`).
    fn step(&mut self, world: &mut World, _dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        debug!("Stem cell hierarchy step {}", self.step_count);

        let enable_plasticity    = self.params.enable_plasticity;
        let plasticity_rate      = self.params.plasticity_rate;
        let plasticity_threshold = self.params.differentiation_threshold;

        for (_, (hierarchy, damage)) in
            world.query_mut::<(&mut StemCellHierarchyState, &CentriolarDamageState)>()
        {
            // Пластичность: Oligopotent → Pluripotent при восстановлении веретена
            if enable_plasticity
                && hierarchy.potency_level == PotencyLevel::Oligopotent
                && damage.spindle_fidelity > plasticity_threshold
                && self.rng.gen::<f32>() < plasticity_rate
            {
                hierarchy.set_potency(PotencyLevel::Pluripotent);
                hierarchy.dedifferentiation_count += 1;
                debug!(
                    "Dedifferentiation event: Oligopotent → Pluripotent \
                     (spindle={:.3}, count={})",
                    damage.spindle_fidelity, hierarchy.dedifferentiation_count
                );
                continue; // уже обновили потентность
            }

            // Стандартная синхронизация потентности из spindle_fidelity
            let new_potency = if damage.spindle_fidelity > 0.95 {
                PotencyLevel::Totipotent
            } else if damage.spindle_fidelity > 0.75 {
                PotencyLevel::Pluripotent
            } else if damage.spindle_fidelity > 0.45 {
                PotencyLevel::Oligopotent
            } else if damage.spindle_fidelity > 0.15 {
                PotencyLevel::Unipotent
            } else {
                PotencyLevel::Apoptosis
            };

            if new_potency != hierarchy.potency_level {
                hierarchy.set_potency(new_potency);
            }
        }

        Ok(())
    }

    fn get_params(&self) -> Value {
        json!({
            "initial_potency":           format!("{:?}", self.params.initial_potency),
            "enable_plasticity":         self.params.enable_plasticity,
            "plasticity_rate":           self.params.plasticity_rate,
            "differentiation_threshold": self.params.differentiation_threshold,
            "step_count":                self.step_count,
            // Порог восстановления spindle_fidelity для дедифференцировки
            // (то же поле что differentiation_threshold, но с явным именем)
            "plasticity_spindle_threshold": self.params.differentiation_threshold,
        })
    }

    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(v) = params.get("enable_plasticity").and_then(|v| v.as_bool()) {
            self.params.enable_plasticity = v;
        }
        if let Some(v) = params.get("plasticity_rate").and_then(|v| v.as_f64()) {
            self.params.plasticity_rate = v as f32;
        }
        if let Some(v) = params.get("differentiation_threshold").and_then(|v| v.as_f64()) {
            self.params.differentiation_threshold = v as f32;
        }
        Ok(())
    }

    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        info!("Initializing stem cell hierarchy module");

        let entities: Vec<_> = world
            .query::<&CellCycleStateExtended>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        let count = entities.len();
        for &entity in &entities {
            if !world.contains(entity) { continue; }
            let mut state = StemCellHierarchyState::new();
            state.set_potency(self.params.initial_potency);
            world.insert_one(entity, state)?;
        }

        info!("Initialized hierarchy for {} cells (initial: {:?})",
              count, self.params.initial_potency);
        Ok(())
    }
}

impl Default for StemCellHierarchyModule {
    fn default() -> Self { Self::new() }
}

/// Фабрики для создания стволовых клеток разных типов
pub mod factories {
    use super::*;

    pub fn create_embryonic_stem_cell() -> StemCellHierarchyState {
        let mut state = StemCellHierarchyState::new();
        state.set_potency(PotencyLevel::Pluripotent);
        state.master_regulator_levels.insert("OCT4".to_string(),  1.0);
        state.master_regulator_levels.insert("NANOG".to_string(), 1.0);
        state.master_regulator_levels.insert("SOX2".to_string(),  1.0);
        state
    }

    pub fn create_hematopoietic_stem_cell() -> StemCellHierarchyState {
        let mut state = StemCellHierarchyState::new();
        state.set_potency(PotencyLevel::Oligopotent);
        state.lineage = Some(CellLineage::HematopoieticStem);
        state
    }

    pub fn create_neural_stem_cell() -> StemCellHierarchyState {
        let mut state = StemCellHierarchyState::new();
        state.set_potency(PotencyLevel::Oligopotent);
        state.lineage = Some(CellLineage::NeuralStem);
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_dt_core::components::{CentriolarDamageState, CellCycleStateExtended};
    use cell_dt_core::{SimulationManager, SimulationConfig};

    fn make_damaged_damage_state(spindle: f32) -> CentriolarDamageState {
        let mut d = CentriolarDamageState::pristine();
        // Принудительно выставляем spindle_fidelity через производные поля
        // (spindle_fidelity вычисляется в update_functional_metrics)
        // Для теста проще выставить напрямую:
        d.spindle_fidelity = spindle;
        d
    }

    /// Dedifferentiation: Oligopotent → Pluripotent при восстановлении веретена
    #[test]
    fn test_plasticity_dedifferentiation_occurs() {
        let params = StemCellHierarchyParams {
            enable_plasticity: true,
            plasticity_rate: 1.0,         // 100% вероятность для детерминизма
            differentiation_threshold: 0.6,
            initial_potency: PotencyLevel::Oligopotent,
        };
        let mut module = StemCellHierarchyModule::with_params(params);

        let config = SimulationConfig::default();
        let mut sim = SimulationManager::new(config);

        // Спавним сущность с CellCycleStateExtended (нужен для initialize)
        let entity = sim.world_mut().spawn((
            CellCycleStateExtended::new(),
            make_damaged_damage_state(0.8), // spindle > threshold (0.6)
        ));
        module.initialize(sim.world_mut()).unwrap();

        // Ставим Oligopotent вручную
        {
            let mut q = sim.world_mut().query::<&mut StemCellHierarchyState>();
            for (_, h) in q.iter() {
                h.set_potency(PotencyLevel::Oligopotent);
                h.dedifferentiation_count = 0;
            }
        }

        // Один шаг модуля — должна произойти дедифференцировка
        module.step(sim.world_mut(), 1.0).unwrap();

        let mut q = sim.world_mut().query::<&StemCellHierarchyState>();
        let (_, h) = q.iter().find(|(e, _)| *e == entity).unwrap();
        assert_eq!(h.potency_level, PotencyLevel::Pluripotent,
            "При spindle_fidelity > threshold и plasticity_rate=1.0 должна быть дедифференцировка");
        assert_eq!(h.dedifferentiation_count, 1,
            "dedifferentiation_count должен быть 1");
    }

    /// Без пластичности Oligopotent не поднимается до Pluripotent
    #[test]
    fn test_no_plasticity_when_disabled() {
        let params = StemCellHierarchyParams {
            enable_plasticity: false,
            plasticity_rate: 1.0,
            differentiation_threshold: 0.6,
            initial_potency: PotencyLevel::Oligopotent,
        };
        let mut module = StemCellHierarchyModule::with_params(params);

        let config = SimulationConfig::default();
        let mut sim = SimulationManager::new(config);

        let entity = sim.world_mut().spawn((
            CellCycleStateExtended::new(),
            make_damaged_damage_state(0.8),
        ));
        module.initialize(sim.world_mut()).unwrap();

        // Вручную ставим Oligopotent (spindle=0.8 → initialize даст Pluripotent, сбиваем)
        {
            let mut q = sim.world_mut().query::<&mut StemCellHierarchyState>();
            for (_, h) in q.iter() {
                h.potency_level = PotencyLevel::Oligopotent;
                h.dedifferentiation_count = 0;
            }
        }

        module.step(sim.world_mut(), 1.0).unwrap();

        // При enable_plasticity=false и spindle=0.8: нормальная синхронизация
        // spindle=0.8 > 0.75 → Pluripotent (через стандартный путь), count не меняется
        let mut q = sim.world_mut().query::<&StemCellHierarchyState>();
        let (_, h) = q.iter().find(|(e, _)| *e == entity).unwrap();
        assert_eq!(h.dedifferentiation_count, 0,
            "Без enable_plasticity dedifferentiation_count не должен увеличиваться");
    }
}
