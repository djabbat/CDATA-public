//! Модуль миелоидного сдвига (Myeloid Shift / Myeloid Bias)
//!
//! ## Биологический контекст
//!
//! С возрастом гематопоэтические и другие стволовые клетки постепенно смещают
//! дифференцировку от лимфоидного пути к миелоидному. В контексте CDATA этот
//! сдвиг **напрямую определяется накопленными повреждениями центриоли**:
//!
//! | Компонент CDATA         | Механизм                                            | Вклад |
//! |-------------------------|-----------------------------------------------------|-------|
//! | `spindle_fidelity ↓`    | Веретено не сегрегирует Numb/aPKC → оба потомка миелоидные | 45% |
//! | `ciliary_function ↓`    | Нет реснички → Wnt/Notch/Shh↓ → PU.1 побеждает    | 30%   |
//! | `ros_level ↑`           | ROS → NF-κB → IL-6/TNF-α → миелоидная среда        | 15%   |
//! | `protein_aggregates ↑`  | Агрегаты захватывают Ikaros (IKZF1) → миелоид     | 10%   |
//!
//! ## Обратные связи на CDATA
//!
//! Миелоидные клетки производят больше ROS и секретируют SASP-факторы,
//! которые повреждают нишу и ускоряют накопление повреждений центриоли.
//! Эта петля реализована через [`InflammagingState`]:
//!
//! ```text
//! myeloid_bias ↑ → inflammaging_index ↑ → InflammagingState { ros_boost, niche_impairment }
//!                                       → human_development_module ускоряет ROS-повреждения
//! ```
//!
//! ## Порядок модулей
//!
//! Регистрировать **после** `HumanDevelopmentModule`, так как модуль читает
//! `CentriolarDamageState` (синхронизируется human_development_module в step()).

use cell_dt_core::{
    SimulationModule, SimulationResult,
    hecs::World,
    components::{CentriolarDamageState, CellCycleStateExtended, InflammagingState, TrackABCrossState},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use log::{info, trace, warn};

// ---------------------------------------------------------------------------
// Публичные типы
// ---------------------------------------------------------------------------

/// Фенотип миелоидного сдвига — качественная оценка тяжести
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum MyeloidPhenotype {
    /// myeloid_bias < 0.30 — норма
    #[default]
    Healthy,
    /// 0.30..0.50 — субклинический сдвиг
    MildShift,
    /// 0.50..0.70 — клинически значимый (иммуносупрессия)
    ModerateShift,
    /// > 0.70 — тяжёлое иммуностарение
    SevereShift,
}

impl MyeloidPhenotype {
    pub fn from_bias(bias: f32) -> Self {
        match bias {
            b if b >= 0.70 => Self::SevereShift,
            b if b >= 0.50 => Self::ModerateShift,
            b if b >= 0.30 => Self::MildShift,
            _              => Self::Healthy,
        }
    }
}


/// ECS-компонент: состояние миелоидного сдвига ниши
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyeloidShiftComponent {
    /// Доля миелоидного выхода сверх исходного [0..1]
    pub myeloid_bias: f32,
    /// Дефицит лимфоидного выхода [0..1]
    pub lymphoid_deficit: f32,
    /// Воспалительный индекс (inflammaging) [0..1]
    pub inflammaging_index: f32,
    /// Иммунное старение (exhaustion) [0..1]
    pub immune_senescence: f32,
    /// Качественный фенотип
    pub phenotype: MyeloidPhenotype,
}

impl Default for MyeloidShiftComponent {
    fn default() -> Self {
        Self {
            myeloid_bias: 0.0,
            lymphoid_deficit: 0.0,
            inflammaging_index: 0.0,
            immune_senescence: 0.0,
            phenotype: MyeloidPhenotype::Healthy,
        }
    }
}

// ---------------------------------------------------------------------------
// Параметры модуля (панель управления)
// ---------------------------------------------------------------------------

/// Параметры модуля — веса и коэффициенты обратной связи
#[derive(Debug, Clone)]
pub struct MyeloidShiftParams {
    /// Вес spindle_fidelity в формуле myeloid_bias (default 0.45)
    pub spindle_weight: f32,
    /// Вес ciliary_function (default 0.30)
    pub cilia_weight: f32,
    /// Вес ros_level (default 0.15)
    pub ros_weight: f32,
    /// Вес protein_aggregates (default 0.10)
    pub aggregate_weight: f32,
    /// Масштабирование ros_boost → InflammagingState (default 0.15)
    pub ros_boost_scale: f32,
    /// Масштабирование niche_impairment → InflammagingState (default 0.08)
    pub niche_impair_scale: f32,
    /// Показатель степени нелинейности spindle_fidelity (P10).
    /// Default 1.5 воспроизводит исходную формулу `(1-sf)^1.5`.
    /// 1.0 = линейный, 2.0 = более резкий порог.
    pub spindle_nonlinearity_exponent: f32,
}

impl Default for MyeloidShiftParams {
    fn default() -> Self {
        Self {
            spindle_weight:    0.45,
            cilia_weight:      0.30,
            ros_weight:        0.15,
            aggregate_weight:  0.10,
            ros_boost_scale:   0.15,
            niche_impair_scale: 0.08,
            spindle_nonlinearity_exponent: 1.5,
        }
    }
}

// ---------------------------------------------------------------------------
// Модуль
// ---------------------------------------------------------------------------

pub struct MyeloidShiftModule {
    params: MyeloidShiftParams,
    step_count: u64,
}

impl MyeloidShiftModule {
    pub fn new() -> Self {
        Self { params: MyeloidShiftParams::default(), step_count: 0 }
    }

    pub fn with_params(params: MyeloidShiftParams) -> Self {
        Self { params, step_count: 0 }
    }

    /// Вычислить myeloid_bias из молекулярных повреждений центриоли.
    ///
    /// Формула CDATA-обоснована:
    /// * spindle_fidelity↓ → (1 − sf)^1.5 × w_spindle  (нелинейность важна: малые
    ///   повреждения не мешают, но после порога эффект резкий)
    /// * ciliary_function↓ → (1 − cf) × w_cilia
    /// * ros_level        → ros × w_ros
    /// * protein_aggregates → agg × w_agg
    ///
    /// P66: принимает эффективные значения spindle/cilia (с учётом TrackAB cross-penalties).
    fn compute_myeloid_bias(
        &self,
        damage: &CentriolarDamageState,
        effective_spindle: f32,
        effective_cilia: f32,
    ) -> f32 {
        let spindle_c  = (1.0 - effective_spindle)
            .powf(self.params.spindle_nonlinearity_exponent) * self.params.spindle_weight;
        let cilia_c    = (1.0 - effective_cilia)                    * self.params.cilia_weight;
        let ros_c      = damage.ros_level                           * self.params.ros_weight;
        let aggr_c     = damage.protein_aggregates                  * self.params.aggregate_weight;

        (spindle_c + cilia_c + ros_c + aggr_c).clamp(0.0, 1.0)
    }
}

impl Default for MyeloidShiftModule {
    fn default() -> Self { Self::new() }
}

impl SimulationModule for MyeloidShiftModule {
    fn name(&self) -> &str { "myeloid_shift_module" }

    fn step(&mut self, world: &mut World, _dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        trace!("MyeloidShift step {}", self.step_count);

        // Читаем CentriolarDamageState (standalone, синхронизирован human_development_module)
        // и пишем MyeloidShiftComponent + InflammagingState.
        // P66: также читаем TrackABCrossState для нелинейного cross-feedback.
        for (_, (damage, myeloid, inflammaging, track_ab_opt)) in world.query_mut::<(
            &CentriolarDamageState,
            &mut MyeloidShiftComponent,
            &mut InflammagingState,
            Option<&TrackABCrossState>,
        )>() {
            // P66: TrackAB cross-feedback — применить штрафы к эффективным значениям
            // effective_spindle/cilia снижены на cross-penalties (хуже, чем каждый в отдельности).
            let (effective_spindle, effective_cilia, cross_boost) = if let Some(cross) = track_ab_opt {
                let eff_sf = (damage.spindle_fidelity - cross.cilia_to_spindle_penalty).max(0.0);
                let eff_cf = (damage.ciliary_function  - cross.spindle_to_cilia_penalty).max(0.0);
                let boost = if cross.cross_talk_active {
                    cross.combined_dysfunction * 0.20 // до +20% дополнительного myeloid_bias
                } else {
                    0.0
                };
                (eff_sf, eff_cf, boost)
            } else {
                (damage.spindle_fidelity, damage.ciliary_function, 0.0)
            };

            let raw_bias = self.compute_myeloid_bias(damage, effective_spindle, effective_cilia);
            let myeloid_bias = (raw_bias + cross_boost).clamp(0.0, 1.0);

            // Лимфоидный дефицит — независимая метрика потери лимфоидного пути:
            //   cilia↓ → Wnt/Notch/Shh↓ → нет поддержки лимфоидных прогениторов (55%)
            //   aggregates↑ → Ikaros/IKZF1 захвачен агрегатами → лимфоидные TF потеряны (35%)
            //   hyperacetylation → хроматин лимфоидных генов закрыт (10%)
            let lymphoid_deficit = ((1.0 - damage.ciliary_function) * 0.55
                + damage.protein_aggregates * 0.35
                + damage.tubulin_hyperacetylation * 0.10).clamp(0.0, 1.0);

            // Воспалительный индекс: взвешенная сумма miyeloid bias и lymphoid deficit
            let inflammaging_index = (myeloid_bias * 0.60 + lymphoid_deficit * 0.40).clamp(0.0, 1.0);

            // Иммунное старение: вклад SASP и дефицита Т-/B-клеток
            let immune_senescence = (inflammaging_index * 0.70
                + (1.0 - damage.ciliary_function) * 0.30).clamp(0.0, 1.0);

            // Предупреждение: критический иммуностарение
            if myeloid_bias >= 0.95 {
                warn!("myeloid_bias={:.3} ≥ 0.95 — severe immunosenescence", myeloid_bias);
            }

            // Обновляем компонент
            myeloid.myeloid_bias      = myeloid_bias;
            myeloid.lymphoid_deficit  = lymphoid_deficit;
            myeloid.inflammaging_index = inflammaging_index;
            myeloid.immune_senescence  = immune_senescence;
            myeloid.phenotype          = MyeloidPhenotype::from_bias(myeloid_bias);

            // Обратная связь → human_development_module (применится на следующем шаге)
            inflammaging.ros_boost        = (inflammaging_index * self.params.ros_boost_scale)
                .clamp(0.0, 0.5);
            inflammaging.niche_impairment = (inflammaging_index * self.params.niche_impair_scale)
                .clamp(0.0, 0.5);
            inflammaging.sasp_intensity   = inflammaging_index;
        }

        Ok(())
    }

    fn get_params(&self) -> Value {
        json!({
            "spindle_weight":                self.params.spindle_weight,
            "cilia_weight":                  self.params.cilia_weight,
            "ros_weight":                    self.params.ros_weight,
            "aggregate_weight":              self.params.aggregate_weight,
            "ros_boost_scale":               self.params.ros_boost_scale,
            "niche_impair_scale":            self.params.niche_impair_scale,
            "spindle_nonlinearity_exponent": self.params.spindle_nonlinearity_exponent,
            "step_count":                    self.step_count,
        })
    }

    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        macro_rules! set_f32 {
            ($key:literal, $field:expr) => {
                if let Some(v) = params.get($key).and_then(|v| v.as_f64()) {
                    $field = v as f32;
                }
            };
        }
        set_f32!("spindle_weight",                self.params.spindle_weight);
        set_f32!("cilia_weight",                  self.params.cilia_weight);
        set_f32!("ros_weight",                    self.params.ros_weight);
        set_f32!("aggregate_weight",              self.params.aggregate_weight);
        set_f32!("ros_boost_scale",               self.params.ros_boost_scale);
        set_f32!("niche_impair_scale",            self.params.niche_impair_scale);
        set_f32!("spindle_nonlinearity_exponent", self.params.spindle_nonlinearity_exponent);
        Ok(())
    }

    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        info!("Initializing myeloid shift module");

        let entities: Vec<_> = world
            .query::<&CellCycleStateExtended>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        let count = entities.len();
        for &entity in &entities {
            if !world.contains(entity) { continue; }
            // MyeloidShiftComponent — основные метрики сдвига
            world.insert_one(entity, MyeloidShiftComponent::default())?;
            // InflammagingState уже добавлена human_development_module.initialize(),
            // но если этот модуль зарегистрирован без human_development_module —
            // добавляем сами (insert_one вернёт ошибку если компонент уже есть,
            // поэтому используем try + игнорируем ошибку дублирования).
            let _ = world.insert_one(entity, InflammagingState::default());
        }

        info!("MyeloidShift: initialized {} niches", count);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Тесты
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cell_dt_core::components::CentriolarDamageState;

    fn pristine() -> CentriolarDamageState { CentriolarDamageState::pristine() }

    fn max_damage() -> CentriolarDamageState {
        let mut d = CentriolarDamageState::pristine();
        d.spindle_fidelity = 0.0;
        d.ciliary_function = 0.0;
        d.ros_level        = 1.0;
        d.protein_aggregates = 1.0;
        d
    }

    fn module() -> MyeloidShiftModule { MyeloidShiftModule::new() }

    #[test]
    fn test_pristine_no_shift() {
        let d = pristine();
        let bias = module().compute_myeloid_bias(&d, d.spindle_fidelity, d.ciliary_function);
        // spindle=1, cilia=1, ros=0, agg=0 → bias = 0 + 0 + 0 + 0
        assert!(bias < 0.05, "pristine damage → myeloid_bias={} (expected ≈0)", bias);
    }

    #[test]
    fn test_max_damage_full_shift() {
        let d = max_damage();
        let bias = module().compute_myeloid_bias(&d, d.spindle_fidelity, d.ciliary_function);
        // (1)^1.5×0.45 + 1×0.30 + 1×0.15 + 1×0.10 = 1.0
        assert!((bias - 1.0).abs() < 0.01,
            "max damage → myeloid_bias={} (expected ≈1.0)", bias);
    }

    #[test]
    fn test_spindle_drives_shift() {
        let mut d = pristine();
        d.spindle_fidelity = 0.0;   // только сломано веретено
        let bias = module().compute_myeloid_bias(&d, d.spindle_fidelity, d.ciliary_function);
        // (1)^1.5×0.45 = 0.45
        assert!((bias - 0.45).abs() < 0.01,
            "spindle_fidelity=0 → bias={} (expected ≈0.45)", bias);
    }

    #[test]
    fn test_cilia_drives_shift() {
        let mut d = pristine();
        d.ciliary_function = 0.0;   // только нет реснички
        let bias = module().compute_myeloid_bias(&d, d.spindle_fidelity, d.ciliary_function);
        // 1×0.30 = 0.30
        assert!((bias - 0.30).abs() < 0.01,
            "ciliary_function=0 → bias={} (expected ≈0.30)", bias);
    }

    #[test]
    fn test_calibration_age70() {
        // При типичных повреждениях в 70 лет:
        // spindle≈0.40, cilia≈0.50, ros≈0.40, agg≈0.30
        let mut d = pristine();
        d.spindle_fidelity = 0.40;
        d.ciliary_function = 0.50;
        d.ros_level        = 0.40;
        d.protein_aggregates = 0.30;
        let bias = module().compute_myeloid_bias(&d, d.spindle_fidelity, d.ciliary_function);
        // ≈ 0.6^1.5×0.45 + 0.5×0.30 + 0.4×0.15 + 0.3×0.10
        // ≈ 0.209 + 0.15 + 0.06 + 0.03 = 0.449
        assert!(bias > 0.30 && bias < 0.65,
            "age-70 damage → bias={} (expected ModerateShift ≈0.45)", bias);
    }

    #[test]
    fn test_feedback_ros_boost() {
        let m = module();
        let inflammaging_index = 0.5_f32;
        let ros_boost = (inflammaging_index * m.params.ros_boost_scale).clamp(0.0, 0.5);
        assert!(ros_boost > 0.0, "inflammaging > 0 → ros_boost > 0");
        assert!(ros_boost <= 0.5, "ros_boost must not exceed 0.5");
    }

    #[test]
    fn test_phenotype_classification() {
        assert_eq!(MyeloidPhenotype::from_bias(0.10), MyeloidPhenotype::Healthy);
        assert_eq!(MyeloidPhenotype::from_bias(0.35), MyeloidPhenotype::MildShift);
        assert_eq!(MyeloidPhenotype::from_bias(0.60), MyeloidPhenotype::ModerateShift);
        assert_eq!(MyeloidPhenotype::from_bias(0.80), MyeloidPhenotype::SevereShift);
    }

    /// P10: показатель степени > 1.5 даёт более резкий порог (меньший bias при малых повреждениях)
    #[test]
    fn test_spindle_nonlinearity_exponent_effect() {
        let mut d = pristine();
        d.spindle_fidelity = 0.5; // умеренное повреждение

        let m_default = MyeloidShiftModule::new(); // exponent = 1.5
        let mut m_sharp = MyeloidShiftModule::new();
        m_sharp.params.spindle_nonlinearity_exponent = 2.5; // более резкий порог

        let bias_default = m_default.compute_myeloid_bias(&d, d.spindle_fidelity, d.ciliary_function);
        let bias_sharp   = m_sharp.compute_myeloid_bias(&d, d.spindle_fidelity, d.ciliary_function);

        // При sf=0.5: (0.5)^1.5 ≈ 0.354 vs (0.5)^2.5 ≈ 0.177
        // → sharp bias должен быть меньше при умеренных повреждениях
        assert!(bias_sharp < bias_default,
            "exponent=2.5 → меньший bias при sf=0.5: default={:.4}, sharp={:.4}",
            bias_default, bias_sharp);

        // При максимальных повреждениях (sf=0) оба дают одинаковый результат
        d.spindle_fidelity = 0.0;
        let bias_max_default = m_default.compute_myeloid_bias(&d, d.spindle_fidelity, d.ciliary_function);
        let bias_max_sharp   = m_sharp.compute_myeloid_bias(&d, d.spindle_fidelity, d.ciliary_function);
        assert!((bias_max_default - bias_max_sharp).abs() < 0.001,
            "при sf=0 оба варианта дают одинаковый bias: {:.4} vs {:.4}",
            bias_max_default, bias_max_sharp);
    }

    // ── P66 tests ────────────────────────────────────────────────────────────

    /// P66: combined_dysfunction нелинейна — одновременная потеря cilia И spindle
    /// даёт combined_dysfunction ВЫШЕ, чем при потере только одного трека.
    #[test]
    fn test_track_ab_cross_combined_dysfunction_nonlinear() {
        use cell_dt_core::components::{TrackABCrossState, TrackABCrossParams, update_track_ab_cross};

        let params = TrackABCrossParams::default(); // nonlinearity_exponent = 1.8

        // Только cilia нарушены (spindle здоров)
        let mut state_cilia_only = TrackABCrossState::default();
        update_track_ab_cross(&mut state_cilia_only, &params, 0.2, 1.0);

        // Только spindle нарушен (cilia здоровы)
        let mut state_spindle_only = TrackABCrossState::default();
        update_track_ab_cross(&mut state_spindle_only, &params, 1.0, 0.2);

        // Оба нарушены одновременно
        let mut state_both = TrackABCrossState::default();
        update_track_ab_cross(&mut state_both, &params, 0.2, 0.2);

        // combined_dysfunction при обоих дефектах должна быть выше, чем среднее двух одиночных
        let mean_single = (state_cilia_only.combined_dysfunction
            + state_spindle_only.combined_dysfunction) / 2.0;
        assert!(
            state_both.combined_dysfunction > mean_single,
            "P66: simultaneous cilia+spindle loss → combined_dysfunction={:.4} > mean_single={:.4}",
            state_both.combined_dysfunction, mean_single
        );
        // Убедимся что нелинейность реальная (не просто линейная сумма)
        let sum_linear = state_cilia_only.combined_dysfunction
            + state_spindle_only.combined_dysfunction;
        assert!(
            state_both.combined_dysfunction > sum_linear * 0.5,
            "combined_dysfunction должна отражать нелинейный эффект"
        );
    }

    /// P66: cross_talk_active → myeloid_bias превышает то, что даёт чистая линейная формула.
    #[test]
    fn test_track_ab_cross_boosts_myeloid_bias() {
        use cell_dt_core::components::{TrackABCrossState, TrackABCrossParams, update_track_ab_cross};

        let m = module();
        // Умеренное повреждение обоих треков — cross_talk_active = true (дефицит > 0.30)
        let mut d = pristine();
        d.spindle_fidelity = 0.30; // deficit = 0.70 > threshold 0.30
        d.ciliary_function = 0.30; // deficit = 0.70 > threshold 0.30
        d.ros_level = 0.2;

        // Без cross-feedback
        let bias_no_cross = m.compute_myeloid_bias(&d, d.spindle_fidelity, d.ciliary_function);

        // С cross-feedback
        let mut cross = TrackABCrossState::default();
        let params = TrackABCrossParams::default();
        update_track_ab_cross(&mut cross, &params, d.ciliary_function, d.spindle_fidelity);

        assert!(cross.cross_talk_active, "cross_talk должен быть активен при deficit=0.70");

        let eff_sf = (d.spindle_fidelity - cross.cilia_to_spindle_penalty).max(0.0);
        let eff_cf = (d.ciliary_function  - cross.spindle_to_cilia_penalty).max(0.0);
        let raw_bias = m.compute_myeloid_bias(&d, eff_sf, eff_cf);
        let cross_boost = cross.combined_dysfunction * 0.20;
        let bias_with_cross = (raw_bias + cross_boost).clamp(0.0, 1.0);

        assert!(
            bias_with_cross > bias_no_cross,
            "P66: cross_talk_active → bias_with_cross={:.4} > bias_no_cross={:.4}",
            bias_with_cross, bias_no_cross
        );
    }

    /// P66: cross_talk_active активируется только при дефиците > activation_threshold (0.30).
    #[test]
    fn test_track_ab_cross_talk_activates_at_threshold() {
        use cell_dt_core::components::{TrackABCrossState, TrackABCrossParams, update_track_ab_cross};

        let params = TrackABCrossParams::default(); // activation_threshold = 0.30

        // Ниже порога: cilia=0.75 (deficit=0.25 < 0.30), spindle=1.0 (deficit=0)
        let mut state_below = TrackABCrossState::default();
        update_track_ab_cross(&mut state_below, &params, 0.75, 1.0);
        assert!(
            !state_below.cross_talk_active,
            "P66: deficit=0.25 < threshold=0.30 → cross_talk НЕ активен"
        );

        // Выше порога: cilia=0.65 (deficit=0.35 > 0.30)
        let mut state_above = TrackABCrossState::default();
        update_track_ab_cross(&mut state_above, &params, 0.65, 1.0);
        assert!(
            state_above.cross_talk_active,
            "P66: deficit=0.35 > threshold=0.30 → cross_talk активен"
        );

        // На пороге: cilia=0.70 (deficit=0.30 — не превышает строго)
        let mut state_at = TrackABCrossState::default();
        update_track_ab_cross(&mut state_at, &params, 0.70, 1.0);
        // deficit=0.30 НЕ > 0.30, значит cross_talk НЕ активен
        assert!(
            !state_at.cross_talk_active,
            "P66: deficit=0.30 == threshold → cross_talk НЕ активен (строгое >)"
        );
    }
}
