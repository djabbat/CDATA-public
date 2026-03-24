use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Phase {
    G1,
    S,
    G2,
    M,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeneExpression {
    pub profile: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellCycleState {
    pub phase: Phase,
    pub progress: f32,
}

impl Default for CellCycleState {
    fn default() -> Self {
        Self { phase: Phase::G1, progress: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PTMProfile {
    pub acetylation_level: f32,
    pub oxidation_level: f32,
    pub methylation_level: f32,
    pub phosphorylation_level: f32,
}

impl Default for PTMProfile {
    fn default() -> Self {
        Self {
            acetylation_level: 0.0,
            oxidation_level: 0.0,
            methylation_level: 0.0,
            phosphorylation_level: 0.0,
        }
    }
}

/// Centrosomal memory — high-dimensional history vector carried by each centriole.
///
/// Encodes the centriole's accumulated experience across its lifetime:
/// - `ptm_history[0..4]`: exponential moving average of PTM levels
///   (acetylation, oxidation, methylation, phosphorylation)
/// - `oxidative_stress_history`: cumulative oxidative stress experienced [0..1]
/// - `thermal_stress_history`: thermal/proteotoxic stress history [0..1]
/// - `metabolic_index`: ATP/ADP proxy — 1.0 = healthy metabolism, 0.0 = starved [0..1]
/// - `niche_orientation`: position relative to basement membrane [0..1, 1 = basal]
///
/// Updated each step in `CentrioleModule`. Inherited by daughter centriole at division
/// (epigenetic inheritance of stress context).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentrosomeMemo {
    pub ptm_history: [f32; 4],
    pub oxidative_stress_history: f32,
    pub thermal_stress_history: f32,
    pub metabolic_index: f32,
    pub niche_orientation: f32,
}

impl Default for CentrosomeMemo {
    fn default() -> Self {
        Self {
            ptm_history: [0.0; 4],
            oxidative_stress_history: 0.0,
            thermal_stress_history: 0.0,
            metabolic_index: 1.0,
            niche_orientation: 1.0,
        }
    }
}

impl CentrosomeMemo {
    /// Exponential moving average decay factor per step.
    /// α ≈ 0.01 means ~100-step memory horizon.
    const ALPHA: f32 = 0.01;

    /// Update PTM history from current PTM profile (EMA).
    pub fn update_ptm(&mut self, ptm: &PTMProfile) {
        let a = Self::ALPHA;
        self.ptm_history[0] = (1.0 - a) * self.ptm_history[0] + a * ptm.acetylation_level;
        self.ptm_history[1] = (1.0 - a) * self.ptm_history[1] + a * ptm.oxidation_level;
        self.ptm_history[2] = (1.0 - a) * self.ptm_history[2] + a * ptm.methylation_level;
        self.ptm_history[3] = (1.0 - a) * self.ptm_history[3] + a * ptm.phosphorylation_level;
    }

    /// Update oxidative stress history (EMA).
    pub fn update_oxidative(&mut self, ros_level: f32) {
        self.oxidative_stress_history =
            (1.0 - Self::ALPHA) * self.oxidative_stress_history + Self::ALPHA * ros_level.clamp(0.0, 1.0);
    }

    /// Aggregate damage score from memory (used by diagnosis_engine-style callers).
    pub fn memory_damage_score(&self) -> f32 {
        let ptm_mean = self.ptm_history.iter().sum::<f32>() / 4.0;
        0.5 * ptm_mean + 0.3 * self.oxidative_stress_history + 0.2 * (1.0 - self.metabolic_index)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CAFD {
    pub name: String,
    pub activity: f32,
    pub concentration: f32,
}

impl CAFD {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            activity: 0.0,
            concentration: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Centriole {
    pub maturity: f32,
    pub ptm_signature: PTMProfile,
    pub associated_cafds: Vec<CAFD>,
    /// Centrosomal memory — accumulated history vector (PTM, stress, metabolic context).
    pub centrosomal_memory: CentrosomeMemo,
}

impl Centriole {
    pub fn new(maturity: f32) -> Self {
        Self {
            maturity,
            ptm_signature: PTMProfile::default(),
            associated_cafds: Vec::new(),
            centrosomal_memory: CentrosomeMemo::default(),
        }
    }

    /// Create a daughter centriole that inherits the mother's centrosomal memory.
    /// This models epigenetic inheritance of stress context at division.
    pub fn new_daughter_from(mother: &Centriole) -> Self {
        Self {
            maturity: 0.0,
            ptm_signature: PTMProfile::default(),
            associated_cafds: Vec::new(),
            centrosomal_memory: mother.centrosomal_memory.clone(),
        }
    }

    pub fn new_daughter() -> Self {
        Self::new(0.0)
    }

    pub fn new_mature() -> Self {
        Self::new(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentriolePair {
    pub mother: Centriole,
    pub daughter: Centriole,
    pub cilium_present: bool,
    pub mtoc_activity: f32,
}

impl Default for CentriolePair {
    fn default() -> Self {
        Self {
            mother: Centriole::new_mature(),
            daughter: Centriole::new_daughter(),
            cilium_present: false,
            mtoc_activity: 0.5,
        }
    }
}

// Типы для расширенного клеточного цикла
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CyclinType {
    CyclinD,
    CyclinE,
    CyclinA,
    CyclinB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CdkType {
    Cdk4,
    Cdk6,
    Cdk2,
    Cdk1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Checkpoint {
    G1SRestriction,
    G2MCheckpoint,
    SpindleAssembly,
    DNARepair,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclinCdkComplex {
    pub cyclin_type: CyclinType,
    pub cdk_type: CdkType,
    pub activity: f32,
    pub concentration: f32,
    pub phosphorylation_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthFactors {
    pub growth_signal: f32,
    pub nutrient_level: f32,
    pub stress_level: f32,
    pub dna_damage: f32,
    pub oxidative_stress: f32,
}

impl Default for GrowthFactors {
    fn default() -> Self {
        Self {
            growth_signal: 0.8,
            nutrient_level: 1.0,
            stress_level: 0.0,
            dna_damage: 0.0,
            oxidative_stress: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointState {
    pub checkpoint: Checkpoint,
    pub satisfied: bool,
    pub time_in_checkpoint: f32,
    pub arrest_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellCycleStateExtended {
    pub phase: Phase,
    pub progress: f32,
    pub cyclin_cdk_complexes: Vec<CyclinCdkComplex>,
    pub checkpoints: Vec<CheckpointState>,
    pub current_checkpoint: Option<Checkpoint>,
    pub growth_factors: GrowthFactors,
    pub cycle_count: u32,
    pub time_in_current_phase: f32,
    pub total_time: f32,
    pub centriole_influence: f32,
}

impl CellCycleStateExtended {
    /// Создать клетку в фазе G1 (начало цикла).
    ///
    /// **Обязательный компонент при спавне сущностей.**
    /// Большинство модулей обнаруживают управляемые ими сущности именно по наличию
    /// `CellCycleStateExtended`. При спавне новой сущности всегда включайте этот компонент
    /// первым, затем остальные:
    ///
    /// ```rust,ignore
    /// world.spawn((
    ///     CellCycleStateExtended::new(),   // ← обязателен
    ///     CentriolarDamageState::pristine(),
    ///     // ... остальные компоненты
    /// ));
    /// ```
    pub fn new() -> Self {
        Self {
            phase: Phase::G1,
            progress: 0.0,
            cyclin_cdk_complexes: Vec::new(),
            checkpoints: Vec::new(),
            current_checkpoint: None,
            growth_factors: GrowthFactors::default(),
            cycle_count: 0,
            time_in_current_phase: 0.0,
            total_time: 0.0,
            centriole_influence: 0.0,
        }
    }
}

impl Default for CellCycleStateExtended {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// CDATA — Centriolar Damage Accumulation Theory of Aging
// Компоненты для моделирования полного жизненного цикла
// ============================================================

/// Стадии развития организма (от зиготы до смерти)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DevelopmentalStage {
    /// Оплодотворённая яйцеклетка — нет центриолей, тотипотентность
    Zygote,
    /// 2–16 клеток, де-ново формирование центриолей
    Cleavage,
    /// Бластоциста — ВКМ (плюрипотентные) vs трофобласт
    Blastocyst,
    /// Гаструляция — три зародышевых листка
    Gastrulation,
    /// Нейруляция и органогенез
    Organogenesis,
    /// Плодный период
    Fetal,
    /// Постнатальный рост и развитие (0–18 лет)
    Postnatal,
    /// Взрослый организм (18–40 лет) — гомеостаз тканей
    Adult,
    /// Средний возраст (40–65 лет) — начало накопления повреждений
    MiddleAge,
    /// Пожилой (65+ лет) — выраженное старение
    Senescent,
    /// Смерть организма
    Death,
}

impl DevelopmentalStage {
    /// Возраст (в годах) начала стадии
    pub fn age_start_years(&self) -> f64 {
        match self {
            DevelopmentalStage::Zygote        => 0.0,
            DevelopmentalStage::Cleavage      => 0.0,
            DevelopmentalStage::Blastocyst    => 0.0,
            DevelopmentalStage::Gastrulation  => 0.0,
            DevelopmentalStage::Organogenesis => 0.0,
            DevelopmentalStage::Fetal         => 0.0,
            DevelopmentalStage::Postnatal     => 0.0,
            DevelopmentalStage::Adult         => 18.0,
            DevelopmentalStage::MiddleAge     => 40.0,
            DevelopmentalStage::Senescent     => 65.0,
            DevelopmentalStage::Death         => 80.0,
        }
    }

    /// Следующая стадия развития
    pub fn next(&self) -> Option<DevelopmentalStage> {
        match self {
            DevelopmentalStage::Zygote        => Some(DevelopmentalStage::Cleavage),
            DevelopmentalStage::Cleavage      => Some(DevelopmentalStage::Blastocyst),
            DevelopmentalStage::Blastocyst    => Some(DevelopmentalStage::Gastrulation),
            DevelopmentalStage::Gastrulation  => Some(DevelopmentalStage::Organogenesis),
            DevelopmentalStage::Organogenesis => Some(DevelopmentalStage::Fetal),
            DevelopmentalStage::Fetal         => Some(DevelopmentalStage::Postnatal),
            DevelopmentalStage::Postnatal     => Some(DevelopmentalStage::Adult),
            DevelopmentalStage::Adult         => Some(DevelopmentalStage::MiddleAge),
            DevelopmentalStage::MiddleAge     => Some(DevelopmentalStage::Senescent),
            DevelopmentalStage::Senescent     => Some(DevelopmentalStage::Death),
            DevelopmentalStage::Death         => None,
        }
    }
}

/// Комплект индукторов дифференцировки на одной центриоли (CDATA)
///
/// Материнская и дочерняя центриоли имеют **разные** комплекты (M и D).
/// Индукторы отщепляются необратимо при проникновении O₂ к центриолям.
/// Новая центриоль синтезируется с числом индукторов, равным ТЕКУЩЕМУ
/// остатку родительской (не исходному максимуму зиготы).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentrioleInducerSet {
    /// Текущий остаток индукторов
    pub remaining: u32,
    /// Количество, унаследованное при синтезе (не абсолютный максимум зиготы)
    pub inherited_count: u32,
}

impl CentrioleInducerSet {
    pub fn full(count: u32) -> Self {
        Self { remaining: count, inherited_count: count }
    }

    pub fn empty() -> Self {
        Self { remaining: 0, inherited_count: 0 }
    }

    /// Дочерний комплект: наследует ТЕКУЩИЙ остаток, а не inherited_count.
    pub fn inherit_from(&self) -> Self {
        Self { remaining: self.remaining, inherited_count: self.remaining }
    }

    /// Полный ли комплект относительно наследованного количества?
    pub fn is_full(&self) -> bool {
        self.inherited_count > 0 && self.remaining == self.inherited_count
    }

    pub fn has_any(&self) -> bool { self.remaining > 0 }

    /// Необратимо отщепить один индуктор. Возвращает true если был доступен.
    pub fn detach_one(&mut self) -> bool {
        if self.remaining > 0 { self.remaining -= 1; true } else { false }
    }
}

/// Уровень потентности — определяется по состоянию обоих индукторных комплектов.
///
/// Переход происходит через отщепление индукторов при O₂-воздействии на центриоли.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PotencyLevel {
    /// M=полный И D=полный — оба комплекта нетронуты
    Totipotent,
    /// M≥1 И D≥1 — оба непусты, но не оба полные
    Pluripotent,
    /// Одна центриоль пуста, другая содержит ≥2 индуктора
    Oligopotent,
    /// Ровно 1 индуктор на одной центриоли, другая пуста
    Unipotent,
    /// M=0 И D=0 — запущен путь запрограммированного апоптоза
    Apoptosis,
}

/// Параметры отщепления индукторов при O₂-воздействии (для панели управления)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InducerDetachmentParams {
    /// Базовая вероятность отщепления на шаг при oxygen_level=1.0
    pub base_detach_probability: f32,
    /// Доля вероятности, приходящаяся на материнскую центриоль [0..1]
    /// 0.5 = равновероятно (по умолчанию); >0.5 = материнская теряет чаще
    pub mother_bias: f32,
    /// Коэффициент влияния возраста (лет) на mother_bias.
    /// По умолчанию 0.0 — возраст не является причиной потери индукторов.
    pub age_bias_coefficient: f32,
    /// Масштаб PTM-опосредованного истощения материнского комплекта.
    ///
    /// Второй, независимый от O₂ путь: структурные ПТМ матери ослабляют
    /// связи индукторов. Вероятность = ptm_asymmetry × ptm_exhaustion_scale.
    /// 0.0 → механизм выключен.
    pub ptm_exhaustion_scale: f32,
}

impl Default for InducerDetachmentParams {
    fn default() -> Self {
        Self {
            base_detach_probability: 0.0003,
            mother_bias: 0.5,           // одинаковая вероятность для M и D
            age_bias_coefficient: 0.0,  // возраст не влияет по умолчанию
            ptm_exhaustion_scale: 0.001, // PTM-асимметрия → истощение матери
        }
    }
}

impl InducerDetachmentParams {
    pub fn effective_mother_bias(&self, age_years: f32) -> f32 {
        (self.mother_bias + age_years * self.age_bias_coefficient).min(0.95)
    }
    pub fn mother_prob(&self, oxygen_level: f32, age_years: f32) -> f32 {
        oxygen_level * self.base_detach_probability * self.effective_mother_bias(age_years)
    }
    pub fn daughter_prob(&self, oxygen_level: f32, age_years: f32) -> f32 {
        oxygen_level * self.base_detach_probability * (1.0 - self.effective_mother_bias(age_years))
    }
}

/// Пара индукторных комплектов (материнская + дочерняя центриоль).
///
/// Заменяет устаревший `CentriolarInducers`. Асимметрия дифференцировки
/// возникает из разного остатка комплектов M и D при O₂-отщеплении:
/// материнская центриоль накапливает больше ПТМ → теряет индукторы чаще.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentriolarInducerPair {
    /// Комплект M: индукторы на материнской центриоли
    pub mother_set: CentrioleInducerSet,
    /// Комплект D: индукторы на дочерней центриоли (другой тип молекул)
    pub daughter_set: CentrioleInducerSet,
    /// Суммарное число делений данной клеточной линии
    pub division_count: u32,
    /// Параметры отщепления (настраиваемые через панель управления)
    pub detachment_params: InducerDetachmentParams,
}

impl CentriolarInducerPair {
    /// Зигота: полные комплекты на обеих центриолях.
    pub fn zygote(mother_max: u32, daughter_max: u32) -> Self {
        Self {
            mother_set: CentrioleInducerSet::full(mother_max),
            daughter_set: CentrioleInducerSet::full(daughter_max),
            division_count: 0,
            detachment_params: InducerDetachmentParams::default(),
        }
    }

    /// Определить уровень потентности по текущему состоянию обоих комплектов.
    pub fn potency_level(&self) -> PotencyLevel {
        let m = self.mother_set.remaining;
        let d = self.daughter_set.remaining;
        match (m, d) {
            (0, 0) => PotencyLevel::Apoptosis,
            (1, 0) | (0, 1) => PotencyLevel::Unipotent,
            (_, 0) | (0, _) => PotencyLevel::Oligopotent,
            _ if self.mother_set.is_full() && self.daughter_set.is_full() => PotencyLevel::Totipotent,
            _ => PotencyLevel::Pluripotent,
        }
    }

    pub fn is_apoptotic(&self) -> bool {
        self.potency_level() == PotencyLevel::Apoptosis
    }

    /// Создать пары для двух дочерних клеток при делении.
    /// Новая дочерняя центриоль синтезируется с ТЕКУЩИМ остатком родительской.
    pub fn divide(&mut self) -> (CentriolarInducerPair, CentriolarInducerPair) {
        self.division_count += 1;
        let cell_a = CentriolarInducerPair {
            mother_set:  self.mother_set.clone(),
            daughter_set: self.mother_set.inherit_from(),
            division_count: 0,
            detachment_params: self.detachment_params,
        };
        let cell_b = CentriolarInducerPair {
            mother_set:  self.daughter_set.clone(),
            daughter_set: self.daughter_set.inherit_from(),
            division_count: 0,
            detachment_params: self.detachment_params,
        };
        (cell_a, cell_b)
    }
}

impl Default for CentriolarInducerPair {
    fn default() -> Self { Self::zygote(10, 8) }
}

/// Состояние повреждений центриоли (CDATA)
///
/// Повреждения накапливаются необратимо в материнской центриоли
/// стволовых клеток на протяжении всей жизни организма.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentriolarDamageState {
    // --- Молекулярные повреждения ---
    /// Уровень карбонилирования белков (SAS-6, CEP135) — окислительный стресс
    pub protein_carbonylation: f32,
    /// Гиперацетилирование альфа-тубулина (снижение HDAC6/SIRT2) [0..1]
    pub tubulin_hyperacetylation: f32,
    /// Агрегаты белков (CPAP, CEP290) — блокируют аппарат дупликации [0..1]
    pub protein_aggregates: f32,
    /// Нарушение фосфорилирования (PLK4, NEK2, PP1) [0..1]
    pub phosphorylation_dysregulation: f32,

    // --- Потеря дистальных придатков ---
    /// Целостность CEP164 (главный: инициация ресничек) [0..1]
    pub cep164_integrity: f32,
    /// Целостность CEP89 [0..1]
    pub cep89_integrity: f32,
    /// Целостность Ninein (субдистальные придатки, якорение) [0..1]
    pub ninein_integrity: f32,
    /// Целостность CEP170 [0..1]
    pub cep170_integrity: f32,

    // --- Производные функциональные метрики ---
    /// Функциональность первичной реснички [0..1] — зависит от придатков
    pub ciliary_function: f32,
    /// Точность ориентации веретена деления [0..1] — определяет АКД
    pub spindle_fidelity: f32,
    /// Кумулятивный уровень ROS в нише (петля обратной связи)
    pub ros_level: f32,

    /// Общее число делений клетки (счётчик Хейфлика)
    pub total_divisions: u32,
    /// Клетка вошла в сенесценцию?
    pub is_senescent: bool,
    /// Порог total_damage_score для входа в сенесценцию.
    /// Синхронизируется из `DamageParams::senescence_threshold` через `accumulate_damage()`.
    /// По умолчанию: 0.75 (соответствует ~78 годам при нормальном старении).
    pub senescence_threshold: f32,
}

impl CentriolarDamageState {
    /// Новорождённая центриоль (де-ново или в зиготе) — без повреждений
    pub fn pristine() -> Self {
        Self {
            protein_carbonylation: 0.0,
            tubulin_hyperacetylation: 0.0,
            protein_aggregates: 0.0,
            phosphorylation_dysregulation: 0.0,
            cep164_integrity: 1.0,
            cep89_integrity: 1.0,
            ninein_integrity: 1.0,
            cep170_integrity: 1.0,
            ciliary_function: 1.0,
            spindle_fidelity: 1.0,
            ros_level: 0.05,
            total_divisions: 0,
            is_senescent: false,
            senescence_threshold: 0.75,
        }
    }

    /// Обновить производные метрики из молекулярных повреждений
    pub fn update_functional_metrics(&mut self) {
        // Функция реснички — зависит от целостности дистальных придатков
        let appendage_mean = (self.cep164_integrity
            + self.cep89_integrity
            + self.ninein_integrity
            + self.cep170_integrity) / 4.0;
        self.ciliary_function = appendage_mean * (1.0 - self.protein_aggregates * 0.5);

        // Точность веретена — деградирует от карбонилирования и агрегатов
        let structural_damage = (self.protein_carbonylation + self.protein_aggregates) / 2.0;
        self.spindle_fidelity = (1.0 - structural_damage).max(0.0)
            * (1.0 - self.phosphorylation_dysregulation * 0.3);

        // Сенесценция — когда суммарный ущерб превышает настраиваемый порог.
        // Порог синхронизируется из DamageParams::senescence_threshold через accumulate_damage().
        let total_damage = self.total_damage_score();
        if total_damage > self.senescence_threshold {
            self.is_senescent = true;
        }
    }

    /// Суммарный балл повреждений [0..1]
    pub fn total_damage_score(&self) -> f32 {
        let mol_damage = (self.protein_carbonylation
            + self.tubulin_hyperacetylation
            + self.protein_aggregates
            + self.phosphorylation_dysregulation) / 4.0;
        let appendage_loss = 1.0 - (self.cep164_integrity
            + self.cep89_integrity
            + self.ninein_integrity
            + self.cep170_integrity) / 4.0;
        (mol_damage + appendage_loss) / 2.0
    }

    /// Вероятность симметричного деления (оба потомка дифференцируются
    /// ИЛИ оба самообновляются) — растёт по мере снижения spindle_fidelity
    pub fn symmetric_division_probability(&self) -> f32 {
        (1.0 - self.spindle_fidelity).powf(1.5)
    }

    /// Вероятность истощения пула (оба потомка дифференцируются)
    pub fn pool_exhaustion_probability(&self) -> f32 {
        self.symmetric_division_probability() * 0.6
    }

    /// Hill-функция активации GLI через первичную ресничку (Трек A, нелинейный).
    ///
    /// Моделирует реснично-зависимую обработку GLI-транскрипционного фактора
    /// в позвоночных: SMO → кончик реснички → GLI-активатор.
    ///
    /// Формула: `GLI = cilia^n / (K^n + cilia^n)`
    /// - K = 0.5 (EC50, нормализованный): при ciliary_function=0.5 ответ = 50%
    /// - n = 2.0 (коэффициент Хилла): кооперативность → сигмоидный порог
    ///
    /// Биологическое обоснование: переход от линейного к нелинейному (~возраст 50 лет)
    /// соответствует клиническим данным о резком снижении Hedgehog-зависимого
    /// самообновления HSC и нейральных прогениторов (Rohatgi et al., 2007).
    pub fn gli_activation(&self) -> f32 {
        const K: f32 = 0.5; // EC50 нормализованный
        const N: f32 = 2.0; // коэффициент Хилла (кооперативность SMO-GLI)
        let c = self.ciliary_function;
        c.powf(N) / (K.powf(N) + c.powf(N))
    }
}

impl Default for CentriolarDamageState {
    fn default() -> Self {
        Self::pristine()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AppendageProteinState — независимый ECS-компонент для белков дистальных
// придатков материнской центриоли.
//
// Отделён от CentriolarDamageState чтобы:
//   1. Моделировать независимую кинетику каждого белка (разная чувствительность
//      к OH·, разные скорости репарации).
//   2. Вычислять CAII (Centriolar Appendage Integrity Index) — первичный
//      биомаркер EIC WP1 (U-ExM, n=288 участников).
//   3. Разделить функциональные роли: IFT/Shh, MT-якорение, удлинение MT.
//
// При наличии этого компонента HumanDevelopmentModule обновляет его
// независимо, а потом синхронизирует cep164_integrity ... cep170_integrity
// обратно в CentriolarDamageState.
// ─────────────────────────────────────────────────────────────────────────────

/// Независимое состояние белков дистальных придатков материнской центриоли.
///
/// Каждый белок имеет свою чувствительность к OH· (гидроксил-радикал через
/// реакцию Фентона: Fe²⁺ + H₂O₂ → OH· + Fe³⁺ + OH⁻) и отдельную роль
/// в функции первичной реснички и MTOC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendageProteinState {
    // ── Дистальные придатки (контакт с плазматической мембраной) ─────────────

    /// Целостность CEP164 [0..1].
    ///
    /// Наиболее чувствителен к OH·: содержит коiled-coil-домены с Met/Cys,
    /// которые преимущественно окисляются при Fenton-реакции.
    /// Функция: рекрутинг IFT-A/B комплекса → начало цилиогенеза → Shh/Wnt↑.
    /// При CEP164 < 0.3: первичная ресничка не образуется (полная потеря Shh).
    /// Источник: Graser et al. 2007; Lo et al. 2012.
    pub cep164: f32,

    /// Целостность CEP89 [0..1].
    ///
    /// Умеренная чувствительность к OH·.
    /// Функция: структурная стабилизация дистального аппендажа →
    ///   закрепление переходной зоны реснички → барьер диффузии.
    /// Источник: Sillibourne et al. 2013.
    pub cep89: f32,

    // ── Субдистальные придатки (якорение микротрубочек) ──────────────────────

    /// Целостность Ninein [0..1].
    ///
    /// Низкая чувствительность к OH· (защищена субдистальным расположением).
    /// Функция: якорение минус-концов микротрубочек к материнской центриоли
    ///   → MTOC-активность → правильная ориентация веретена.
    /// Источник: Mogensen et al. 2000; Dammermann & Merdes 2002.
    pub ninein: f32,

    /// Целостность CEP170 [0..1].
    ///
    /// Наименее чувствителен к OH·.
    /// Функция: регуляция динамики MT в интерфазе (скорость полимеризации).
    /// Дефицит CEP170: нарушение цитокинеза → анеуплоидия.
    /// Источник: Guarguaglini et al. 2005.
    pub cep170: f32,

    // ── Производные метрики ───────────────────────────────────────────────────

    /// Centriolar Appendage Integrity Index (CAII) [0..1].
    ///
    /// Взвешенное геометрическое среднее четырёх белков:
    ///   CAII = cep164^0.40 × cep89^0.25 × ninein^0.20 × cep170^0.15
    ///
    /// Веса отражают относительный вклад в функцию реснички:
    ///   CEP164 (0.40) > CEP89 (0.25) > Ninein (0.20) > CEP170 (0.15)
    ///
    /// Первичный биомаркер EIC WP1 — измеряется через U-ExM
    /// (Ultrastructure Expansion Microscopy, ~20nm разрешение).
    /// n=288 доноров (240 evaluable), возраст 20–80 лет.
    pub caii: f32,
}

impl AppendageProteinState {
    /// Интактное состояние всех придатков (новорождённая материнская центриоль).
    pub fn pristine() -> Self {
        Self {
            cep164: 1.0,
            cep89:  1.0,
            ninein: 1.0,
            cep170: 1.0,
            caii:   1.0,
        }
    }

    /// Пересчитать CAII из текущих значений белков.
    ///
    /// Вызывать после каждого обновления белков (потеря / репарация).
    pub fn update_caii(&mut self) {
        self.caii = self.cep164.powf(0.40)
            * self.cep89.powf(0.25)
            * self.ninein.powf(0.20)
            * self.cep170.powf(0.15);
    }

    /// Инициация цилиогенеза (IFT/Shh-путь) — совместная функция CEP164 + CEP89.
    ///
    /// Геометрическое среднее: оба белка необходимы для начала цилиогенеза.
    pub fn ciliary_initiation(&self) -> f32 {
        (self.cep164 * self.cep89).sqrt()
    }

    /// Якорение микротрубочек (MTOC) — определяется Ninein.
    pub fn mt_anchoring(&self) -> f32 {
        self.ninein
    }

    /// Суммарная функция реснички с учётом белков-агрегатов.
    ///
    /// `protein_aggregates` — из CentriolarDamageState (блокируют IFT-трафик).
    pub fn ciliary_function(&self, protein_aggregates: f32) -> f32 {
        self.caii * (1.0 - protein_aggregates * 0.5)
    }
}

impl Default for AppendageProteinState {
    fn default() -> Self {
        Self::pristine()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ThermodynamicState — ECS-компонент (Уровень -4: атомный)
//
// Вводит температурозависимость повреждений через уравнение Аррениуса:
//   k(T) = A × exp(-Eₐ / R / T)
//   mult(T) = k(T) / k(T_ref) = exp(Eₐ/R × (1/T_ref - 1/T))
//
// Связь с Ze Vector Theory:
//   PTM-накопление = необратимый перевод биологического времени в пространство.
//   d(entropy)/dt > 0 всегда (ΔG < 0 для карбонилирования/агрегации).
//   ze_velocity_analog → v* = 0.456 у молодого здорового организма;
//   при старении/воспалении ze_velocity > v* (ускоренное «расходование» времени).
//
// Связь с InflammagingState:
//   sasp_intensity → local_temp_celsius (цитокины → локальное нагревание ниши)
//   → damage_rate_multiplier → умножает effective_dt в accumulate_damage()
// ─────────────────────────────────────────────────────────────────────────────

/// Термодинамическое состояние стволовой ниши (уровень -4: атомный).
///
/// Моделирует температурозависимое ускорение молекулярных повреждений
/// через уравнение Аррениуса и отслеживает энтропийное производство
/// как меру необратимости накопленных PTM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermodynamicState {
    /// Локальная температура в нише [°C].
    ///
    /// Базовая линия: 36.6°C (костномозговая ниша HSC — прохладнее ядра тела 37°C).
    /// Вклад воспаления: sasp_intensity × 2.4°C (предел ≈ 39°C при SASP=1.0).
    /// Источник: Hasday & Singh 2000 — TNF/IL-6 повышают локальную T на 1–3°C.
    pub local_temp_celsius: f32,

    /// Множитель скорости повреждения из уравнения Аррениуса [≥ 1.0].
    ///
    /// mult = exp(Eₐ_mean / R × (1/T_ref − 1/T_local))
    /// T_ref = 310.15 K (37°C). При T=T_ref: mult = 1.0.
    /// При T=39°C (+2°C): mult ≈ 1.14–1.22 в зависимости от трека повреждения.
    /// При T=38°C (+1°C): mult ≈ 1.07–1.10.
    ///
    /// Применяется как дополнительный множитель к effective_dt в accumulate_damage().
    pub damage_rate_multiplier: f32,

    /// Кумулятивное производство энтропии от необратимых PTM [нормированное, ≥ 0].
    ///
    /// Интеграл dS/dt = Σᵢ (damage_rate_i × entropy_weight_i) × dt.
    /// Entropy_weight для каждого трека:
    ///   карбонилирование: 1.2 (C=O — конечный продукт, ΔG << 0)
    ///   агрегация:        2.0 (конформационная → огромное ΔS)
    ///   ацетилирование:   0.8 (модифицирует заряд, умеренный ΔS)
    ///   фосфо-дисрег:     0.5 (обратимо при наличии фосфатаз)
    pub entropy_production: f32,

    /// Ze velocity analog — нормированная скорость энтропийного производства [0..1].
    ///
    /// ze_velocity = entropy_production / (entropy_production + K_ze)
    /// K_ze = 2.0 нормализован так, что ze_velocity ≈ v* = 0.456 в возрасте ~20 лет.
    ///
    /// Ze Theory (Tkemaladze): v* = 0.456 — критическая точка баланса T/S квантов.
    /// Биологическое соответствие:
    ///   v < v*  : гипометаболизм (гибернация, крионика — теоретически)
    ///   v ≈ v*  : молодой здоровый организм (~18–25 лет)
    ///   v > v*  : ускоренное старение (inflammaging, прогерия)
    ///   v → 1.0 : сенесценция / смерть клетки
    pub ze_velocity_analog: f32,
}

impl ThermodynamicState {
    /// Нормальное состояние молодой ниши (37°C, нет воспаления).
    pub fn pristine() -> Self {
        Self {
            local_temp_celsius:     36.6,
            damage_rate_multiplier: 1.0,
            entropy_production:     0.0,
            ze_velocity_analog:     0.0,
        }
    }

    /// T_ref = 310.15 K (37°C) — эталонная температура Аррениуса.
    pub const T_REF_K: f32 = 310.15;

    /// Ze K_ze константа нормировки: entropy/(entropy+K_ze) = v* = 0.456 при ~20 лет.
    pub const ZE_K: f32 = 2.0;
    pub const ZE_OPTIMAL: f32 = 0.456;
}

impl Default for ThermodynamicState {
    fn default() -> Self {
        Self::pristine()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ROSCascadeState — ECS-компонент (Уровень -3: молекулярный, ROS-каскад)
//
// Детализирует единый скаляр `ros_level` в 4-переменный каскад:
//
//   Митохондрии → O₂⁻ (супероксид)
//     ↓ SOD1/SOD2 (супероксид-дисмутаза)
//   H₂O₂ (пероксид водорода)
//     ↓ каталаза / GPx → H₂O + O₂  (детоксикация)
//     ↓ Fe²⁺ (Фентон: Fe²⁺ + H₂O₂ → OH· + Fe³⁺ + OH⁻)
//   OH· (гидроксил-радикал) — наиболее разрушительный, t½ < 1 нс
//     → окисляет CEP164 > CEP89 > Ninein > CEP170 (по чувствительности)
//     → карбонилирует SAS-6/CEP135
//   labile_iron: Fe²⁺/Fe³⁺ пул — катализирует Фентон, пополняется ферритином
//
// Связи:
//   MitochondrialState.ros_production → superoxide (источник)
//   superoxide → hydrogen_peroxide (SOD)
//   hydrogen_peroxide + labile_iron → hydroxyl_radical (Фентон)
//   hydroxyl_radical → AppendageProteinState (OH·-чувствительность)
//   hydroxyl_radical → CentriolarDamageState.protein_carbonylation
//   ros_level (обратная совместимость) = hydrogen_peroxide (основной измеримый ROS)
// ─────────────────────────────────────────────────────────────────────────────

/// ROS-каскад стволовой ниши — 4-переменная молекулярная модель (Уровень -3).
///
/// Заменяет скалярный `ros_level` в `CentriolarDamageState` детализированным
/// каскадом: O₂⁻ → H₂O₂ → OH· с железо-Фентоновским ветвлением.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROSCascadeState {
    /// Супероксид-анион O₂⁻ [нормированный, 0..1].
    ///
    /// Источник: «утечка электронов» комплекса I/III митохондрий (~2% от O₂-потока).
    /// Детоксикация: SOD1 (цитозоль), SOD2 (матрикс) → H₂O₂.
    /// Физиологический уровень: 0.02–0.05. При дисфункции митохондрий: > 0.15.
    /// Источник: Chance et al. 1979; Murphy 2009 Biochem J 417:1-13.
    pub superoxide: f32,

    /// Пероксид водорода H₂O₂ [нормированный, 0..1].
    ///
    /// Образуется из O₂⁻ через SOD (скорость пропорциональна superoxide).
    /// Детоксикация: каталаза (пероксисомы), GPx (цитозоль/митохондрии).
    /// Диффундирует через мембраны → системный сигнал (акваглицерин AQP8).
    /// Соответствует `CentriolarDamageState.ros_level` для обратной совместимости.
    /// Физиологический уровень: 0.03–0.08.
    pub hydrogen_peroxide: f32,

    /// Гидроксил-радикал OH· [нормированный, 0..1].
    ///
    /// Образуется через Фентон: Fe²⁺ + H₂O₂ → OH· + Fe³⁺ + OH⁻.
    /// t½ < 1 нс — реагирует немедленно на месте образования.
    /// Наиболее разрушительный ROS: окисляет боковые цепи Met/Cys/His/Arg.
    /// Прямо управляет AppendageProteinState через oh_sensitivity.
    /// Физиологический уровень: крайне мал (< 0.01); при патологии → 0.05–0.15.
    pub hydroxyl_radical: f32,

    /// Лабильный пул железа Fe²⁺/Fe³⁺ [нормированный, 0..1].
    ///
    /// Катализатор Фентона: высокий labile_iron → больше OH· при том же H₂O₂.
    /// Пополняется: ферритин-деградация (аутофагия ферритина = феррофагия).
    /// Снижается: ферропортин-экспорт, хелатирование (дефероксамин).
    /// Связь с возрастом: накопление негемового железа в нейронах/HSC (Bartzokis 2004).
    /// Физиологический уровень: 0.05–0.15.
    pub labile_iron: f32,
}

impl ROSCascadeState {
    /// Физиологическое начальное состояние (молодая ниша, нет стресса).
    pub fn physiological() -> Self {
        Self {
            superoxide:       0.03,
            hydrogen_peroxide: 0.05,
            hydroxyl_radical: 0.005,
            labile_iron:      0.10,
        }
    }

    /// Вычислить эффективный уровень OH· для AppendageProteinState.
    ///
    /// OH· = hydroxyl_radical × (1 + labile_iron × fenton_amplification).
    /// Чем больше лабильного железа, тем эффективнее Фентон-реакция.
    /// `fenton_amplification` — усиление от Fe²⁺/Fe³⁺ (дефолт: 1.5).
    pub fn effective_oh(&self, fenton_amplification: f32) -> f32 {
        (self.hydroxyl_radical * (1.0 + self.labile_iron * fenton_amplification)).min(1.0)
    }

    /// Ros_level — совместимость с CentriolarDamageState (= H₂O₂).
    ///
    /// Используется для синхронизации `ros_level` при наличии ROSCascadeState.
    pub fn ros_level_compat(&self) -> f32 {
        self.hydrogen_peroxide
    }

    /// Суммарный оксидативный стресс [0..1].
    ///
    /// Взвешенная сумма: OH· вдвое опаснее H₂O₂, O₂⁻ — умеренно.
    pub fn total_oxidative_stress(&self) -> f32 {
        (self.superoxide * 0.3
            + self.hydrogen_peroxide * 0.5
            + self.hydroxyl_radical  * 2.0
            + self.labile_iron       * 0.2)
            .min(1.0) / (0.3 + 0.5 + 2.0 + 0.2)  // нормировка к [0..1]
    }
}

impl Default for ROSCascadeState {
    fn default() -> Self {
        Self::physiological()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GeneticProfile — ECS-компонент (Уровень 0: клетка)
//
// SNP-профиль стволовой ниши: мультипликативные модификаторы DamageParams.
//
// Каждая ниша может иметь индивидуальный генетический фон, задаваемый при
// инициализации. Во время симуляции GeneticProfile не изменяется — читается
// как постоянный контекст при каждом шаге.
//
// # Генетические ассоциации (источники)
//   APOE4:      carbonylation↑, ros_feedback↑, aggregation↑ (Liu 2013)
//   APOE2:      longevity↑ (Lill 2012)
//   LRRK2-G2019S: phospho↑↑, aggregation↑↑ — паркинсонизм (Paisan-Ruiz 2004)
//   FOXO3a:     longevity_factor↓ — голубая зона (Willcox 2008)
//   SOD2-Ala16Val: ros_feedback↑ — неполный импорт в митохондрии (Rosenblum 1996)
//   GPx1-Pro198Leu: ros_feedback↑ (Rahmanto 2012)
// ─────────────────────────────────────────────────────────────────────────────

/// Класс генетического варианта для логирования и SA-анализа.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeneticVariant {
    /// Среднепопуляционный (все множители = 1.0).
    Average,
    /// APOE ε4/ε4 — повышенный риск нейродегенерации, воспаления.
    Apoe4,
    /// APOE ε2/ε2 — протективный вариант, снижение воспаления.
    Apoe2,
    /// LRRK2 G2019S — болезнь Паркинсона, фосфо-дисрегуляция↑↑.
    Lrrk2G2019s,
    /// FOXO3a rs2802292 — долгожительство (голубые зоны).
    FoxO3aLongevity,
    /// SOD2 Ala16Val + GPx1 Pro198Leu — повышенный ROS-петлевой коэффициент.
    Sod2Ala16Val,
    /// Пользовательский (все поля задаются вручную).
    Custom,
}

/// Генетический профиль стволовой ниши (Уровень 0: клетка).
///
/// Мультипликативные модификаторы DamageParams — применяются каждый шаг
/// поверх интервенционных эффектов. Позволяет моделировать SNP-зависимую
/// гетерогенность пула стволовых клеток.
///
/// Значения > 1.0 = повышенный риск. Значения < 1.0 = протективный вариант.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticProfile {
    /// Риск карбонилирования (SAS-6/CEP135 через ROS) [0.5..3.0].
    /// APOE4: 1.30; APOE2: 0.90; среднее: 1.0.
    pub carbonylation_risk: f32,

    /// Риск гиперацетилирования тубулина (HDAC6/SIRT2 варианты) [0.5..2.0].
    pub acetylation_risk: f32,

    /// Риск агрегации (CPAP/CEP290; LRRK2/PINK1/SNCA) [0.5..3.0].
    /// LRRK2-G2019S: 1.60; APOE4: 1.15.
    pub aggregation_risk: f32,

    /// Риск фосфо-дисрегуляции (PLK4/NEK2 полиморфизмы) [0.5..3.0].
    /// LRRK2-G2019S: 1.40 (избыточная LRRK2-киназная активность).
    pub phospho_risk: f32,

    /// Риск потери придатков (CEP164/CEP89 полиморфизмы) [0.5..2.0].
    /// Применяется к дистальным (полностью) и субдистальным (×0.5) придаткам.
    pub appendage_risk: f32,

    /// Риск петли обратной связи ROS (SOD2/GPx1 варианты) [0.5..2.5].
    /// SOD2-Ala16Val: 1.25 (неполный митохондриальный импорт → ROS↑).
    pub ros_feedback_risk: f32,

    /// Фактор долгожительства (FOXO3a/CETP/APOE2) [0.5..1.2].
    /// FOXO3a-долгожитель: 0.80. Среднее: 1.0. Применяется ко всем rate-полям.
    pub longevity_factor: f32,

    /// Тип генетического варианта (для логирования и фильтрации).
    pub variant: GeneticVariant,
}

impl GeneticProfile {
    /// Среднепопуляционный профиль (нейтральный).
    pub fn average() -> Self {
        Self {
            carbonylation_risk: 1.0,
            acetylation_risk:   1.0,
            aggregation_risk:   1.0,
            phospho_risk:       1.0,
            appendage_risk:     1.0,
            ros_feedback_risk:  1.0,
            longevity_factor:   1.0,
            variant:            GeneticVariant::Average,
        }
    }

    /// APOE ε4/ε4 — повышенный нейродегенеративный риск.
    ///
    /// Источники: Liu et al. 2013 Nat Rev Neurosci; Huang & Mahley 2014 Neurobiol Dis.
    pub fn apoe4() -> Self {
        Self {
            carbonylation_risk: 1.30,
            acetylation_risk:   1.05,
            aggregation_risk:   1.15,
            phospho_risk:       1.05,
            appendage_risk:     1.10,
            ros_feedback_risk:  1.20,
            longevity_factor:   1.0,
            variant:            GeneticVariant::Apoe4,
        }
    }

    /// APOE ε2/ε2 — протективный вариант.
    pub fn apoe2() -> Self {
        Self {
            carbonylation_risk: 0.90,
            acetylation_risk:   1.0,
            aggregation_risk:   0.90,
            phospho_risk:       1.0,
            appendage_risk:     0.92,
            ros_feedback_risk:  0.90,
            longevity_factor:   1.0,
            variant:            GeneticVariant::Apoe2,
        }
    }

    /// LRRK2 G2019S — доминантная мутация болезни Паркинсона.
    ///
    /// Gain-of-function киназной активности → PLK4-дисрегуляция, синуклеин-агрегация.
    /// Источник: Paisan-Ruiz et al. 2004 Neuron.
    pub fn lrrk2_g2019s() -> Self {
        Self {
            carbonylation_risk: 1.10,
            acetylation_risk:   1.10,
            aggregation_risk:   1.60,
            phospho_risk:       1.40,
            appendage_risk:     1.20,
            ros_feedback_risk:  1.15,
            longevity_factor:   1.0,
            variant:            GeneticVariant::Lrrk2G2019s,
        }
    }

    /// FOXO3a rs2802292 — долгожительство (Окинава, Сардиния).
    ///
    /// Источник: Willcox et al. 2008 PNAS.
    pub fn foxo3a_longevity() -> Self {
        Self {
            carbonylation_risk: 0.90,
            acetylation_risk:   0.95,
            aggregation_risk:   0.90,
            phospho_risk:       0.95,
            appendage_risk:     0.90,
            ros_feedback_risk:  0.90,
            longevity_factor:   0.80,
            variant:            GeneticVariant::FoxO3aLongevity,
        }
    }

    /// SOD2 Ala16Val — сниженный митохондриальный импорт СОД.
    ///
    /// Источник: Rosenblum et al. 1996; Mehdikhani 2011.
    pub fn sod2_ala16val() -> Self {
        Self {
            carbonylation_risk: 1.15,
            acetylation_risk:   1.05,
            aggregation_risk:   1.10,
            phospho_risk:       1.05,
            appendage_risk:     1.05,
            ros_feedback_risk:  1.25,
            longevity_factor:   1.0,
            variant:            GeneticVariant::Sod2Ala16Val,
        }
    }
}

impl Default for GeneticProfile {
    fn default() -> Self {
        Self::average()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ZeHealthState — ECS-компонент (Уровень -5: Ze Vector Theory)
//
// Интерпретационный слой: переводит молекулярный биомаркер CAII
// в пространство Ze Vector Theory (Tkemaladze).
//
// Ze Theory: каждая биологическая система движется в пространстве-времени
// с нормированной скоростью v ∈ [0,1].
//   v < v* : гипо-метаболизм (гибернация — теоретически)
//   v = v* : оптимальное здоровье (молодой взрослый ~18–25 лет)
//   v > v* : ускоренное старение (inflammaging, прогерия)
//   v → 1  : коллапс (смерть клетки/организма)
//
// Формула: v = v* + (1 − v*) × (1 − CAII)
//   При CAII=1.0: v = v* = 0.456 (интактные придатки = оптимальная скорость)
//   При CAII=0.0: v = 1.0 (полная потеря придатков = коллапс)
// ─────────────────────────────────────────────────────────────────────────────

/// Ze-состояние здоровья стволовой ниши (Уровень -5: Ze Vector Theory).
///
/// Вычисляется из CAII (Centriolar Appendage Integrity Index) через
/// Ze-преобразование. Опционально дополняется энтропийной оценкой
/// из ThermodynamicState.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeHealthState {
    /// Ze-скорость v [0..1].
    ///
    /// Вычисляется из CAII:
    ///   v = v* + (1 − v*) × (1 − CAII) = 0.456 + 0.544 × (1 − CAII)
    ///
    /// Биологический смысл: скорость «потребления» биологического времени.
    /// Молодая клетка (CAII≈1): v≈v*=0.456. Старая (CAII→0): v→1.0.
    pub v: f32,

    /// Отклонение от оптимальной Ze-скорости |v − v*| [0..0.544].
    ///
    /// deviation = 0 → идеальное здоровье.
    /// deviation = 0.544 → полная потеря придатков (CAII=0).
    pub deviation_from_optimal: f32,

    /// Ze-индекс здоровья [0..1].
    ///
    /// ze_health = 1 − deviation / (1 − v*) = CAII.
    /// Эквивалентен CAII — удобный нормированный клинический биомаркер.
    /// При v=v*: ze_health=1.0. При v=1.0: ze_health=0.0.
    pub ze_health_index: f32,

    /// Ze-скорость из энтропийной оценки (ThermodynamicState) [0..1].
    ///
    /// v_entropy = entropy_production / (entropy_production + K_ze)
    /// Независимый термодинамический биомаркер; обновляется если
    /// ThermodynamicState присутствует у сущности.
    pub v_entropy: f32,

    /// Консенсусная Ze-скорость — среднее v и v_entropy [0..1].
    ///
    /// При отсутствии ThermodynamicState: = v.
    pub v_consensus: f32,
}

impl ZeHealthState {
    /// Оптимальная Ze-скорость (критическая точка T/S квантов).
    pub const V_OPTIMAL: f32 = 0.456;
    /// Максимальное отклонение (при CAII=0): 1.0 − v* = 0.544.
    pub const MAX_DEVIATION: f32 = 1.0 - Self::V_OPTIMAL;

    /// Вычислить ZeHealthState из CAII (без энтропийной оценки).
    pub fn from_caii(caii: f32) -> Self {
        let v = Self::V_OPTIMAL + Self::MAX_DEVIATION * (1.0 - caii);
        let deviation = (v - Self::V_OPTIMAL).abs();
        Self {
            v,
            deviation_from_optimal: deviation,
            ze_health_index: caii,
            v_entropy:   0.0,
            v_consensus: v,
        }
    }

    /// Обновить из CAII и опционально из энтропийной оценки.
    pub fn update(&mut self, caii: f32, entropy_v: Option<f32>) {
        self.v = Self::V_OPTIMAL + Self::MAX_DEVIATION * (1.0 - caii);
        self.deviation_from_optimal = (self.v - Self::V_OPTIMAL).abs();
        self.ze_health_index = caii;
        if let Some(ev) = entropy_v {
            self.v_entropy   = ev;
            self.v_consensus = (self.v + ev) / 2.0;
        } else {
            self.v_consensus = self.v;
        }
    }

    /// Биологическая интерпретация текущего состояния.
    pub fn interpretation(&self) -> &'static str {
        match self.deviation_from_optimal {
            d if d < 0.05 => "optimal",
            d if d < 0.15 => "mild_aging",
            d if d < 0.30 => "moderate_aging",
            d if d < 0.45 => "severe_aging",
            _             => "near_collapse",
        }
    }
}

impl Default for ZeHealthState {
    fn default() -> Self {
        Self::from_caii(1.0) // pristine = v*
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MicrotubuleState — ECS-компонент (Уровень -2: органеллы/цитоскелет)
//
// Детализирует скалярный spindle_fidelity через динамику микротрубочек.
//
// Динамическая нестабильность MT (Mitchison & Kirschner 1984):
//   MT переключаются между ростом (polymerization) и распадом (catastrophe).
//   Баланс определяет стабильность MTOC и веретена деления.
//
// Связи:
//   tubulin_hyperacetylation → polymerization_rate↓
//     (гиперацетилированный тубулин: стабильный, но GTPase-неактивный)
//   phosphorylation_dysregulation → catastrophe_rate↑
//     (PLK4/NEK2 дисбаланс → Aurora B нарушен → MT не стабилизируются)
//   ninein_integrity → spindle_fidelity_derived
//     (Ninein якорит минус-концы MT к центриоли → стабилизирует веретено)
//   dynamic_instability_index → spindle_fidelity_derived
// ─────────────────────────────────────────────────────────────────────────────

/// Динамика микротрубочек стволовой ниши (Уровень -2: цитоскелет).
///
/// Заменяет скалярный `spindle_fidelity` в `CentriolarDamageState` моделью,
/// учитывающей полимеризацию и катастрофу MT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrotubuleState {
    /// Нормированная скорость полимеризации MT [0..1].
    ///
    /// Снижается при: tubulin_hyperacetylation (GTPase-инактивация).
    /// Физиологически: ≈ 0.85–1.0. При патологии: < 0.5.
    /// Источник: Bhattacharya et al. 2008 — HDAC6/tubulin deacetylase.
    pub polymerization_rate: f32,

    /// Нормированная частота катастроф MT [0..1].
    ///
    /// Повышается при: phosphorylation_dysregulation (PLK4/NEK2/Aurora B).
    /// Физиологически: ≈ 0.10–0.20. При патологии: > 0.50.
    /// Источник: Гарнер et al. 2004 — PLK1 регулирует катастрофу.
    pub catastrophe_rate: f32,

    /// Индекс динамической нестабильности (DII) [0..1].
    ///
    /// DII = catastrophe_rate / (polymerization_rate + catastrophe_rate)
    /// DII→0: MT стабильны (оба конца растут). DII→1: хаотичный распад.
    /// Источник: Mitchison & Kirschner 1984 Nature 312:237-242.
    pub dynamic_instability_index: f32,

    /// Точность веретена деления из MT-динамики [0..1].
    ///
    /// spindle_fidelity = (1 − DII) × ninein_factor
    /// ninein_factor: целостность Ninein (якорение минус-концов MT).
    /// При наличии этого компонента переопределяет
    /// `CentriolarDamageState.spindle_fidelity`.
    pub spindle_fidelity_derived: f32,
}

impl MicrotubuleState {
    /// Нормальное молодое состояние (активная полимеризация, редкие катастрофы).
    pub fn pristine() -> Self {
        let poly = 0.90_f32;
        let cat  = 0.10_f32;
        let dii  = cat / (poly + cat);
        Self {
            polymerization_rate:        poly,
            catastrophe_rate:           cat,
            dynamic_instability_index:  dii,
            spindle_fidelity_derived:   1.0 - dii, // ninein=1.0
        }
    }

    /// Обновить DII и spindle_fidelity из текущих poly/catastrophe + ninein.
    pub fn update_derived(&mut self, ninein_integrity: f32) {
        let total = self.polymerization_rate + self.catastrophe_rate;
        self.dynamic_instability_index = if total > 0.0 {
            self.catastrophe_rate / total
        } else {
            1.0
        };
        // Ninein — якорение минус-концов: низкая целостность → ненадёжное веретено
        self.spindle_fidelity_derived =
            (1.0 - self.dynamic_instability_index) * ninein_integrity;
    }
}

impl Default for MicrotubuleState {
    fn default() -> Self {
        Self::pristine()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GolgiState — ECS-компонент (Уровень -1: органоиды)
//
// Моделирует состояние аппарата Гольджи стволовой ниши.
//
// Биологические связи:
//   ROS / SASP → фрагментация Гольджи (cisternae разрушаются)
//     → снижение гликозилирующей ёмкости
//     → неполное гликозилирование CEP164
//     → ускоренная убиквитин-протеасомная деградация CEP164
//     → потеря придатков (AppendageProteinState.cep164↓)
//
//   Везикулярный трафик (COPI/COPII) → антероградный транспорт к цилиям
//     → снижение traffic_rate при фрагментации → ciliary_function↓
//
// Ключевые источники:
//   - Sundaramoorthy et al. 2016 (CEP164 гликозилирование и Golgi)
//   - Colanzi & Corda 2007 (Golgi fragmentation и клеточный цикл)
//   - Gonatas et al. 2006 (Golgi fragmentation при нейродегенерации)
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние аппарата Гольджи стволовой ниши (Уровень -1: органоиды).
///
/// Связывает органелльный уровень с молекулярным:
/// фрагментация Гольджи → нарушение гликозилирования CEP164
/// → ускоренная деградация придатков (AppendageProteinState.cep164↓).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolgiState {
    /// Индекс фрагментации Гольджи [0..1].
    ///
    /// 0.0 = интактный аппарат (молодая клетка).
    /// 1.0 = полная фрагментация (тяжёлый стресс/старость).
    /// Растёт при: ROS, SASP-цитокины (TNF/IL-6), ER-стресс.
    /// Снижается при: аутофагия, UPR-адаптация.
    pub fragmentation_index: f32,

    /// Нормированная гликозилирующая ёмкость [0..1].
    ///
    /// Производная от fragmentation_index:
    ///   glycosylation_capacity = (1 − fragmentation × 0.85).clamp(0.1, 1.0)
    /// При полной фрагментации: минимум 0.1 (остаточная активность ER-гликозилирования).
    pub glycosylation_capacity: f32,

    /// Гликозилирование CEP164 [0..1].
    ///
    /// CEP164 требует N-гликозилирования для стабильного фолдинга и рекрутирования
    /// к дистальным придаткам (Sundaramoorthy et al. 2016).
    /// При низком значении: ускоренная убиквитин-зависимая деградация.
    /// cep164_glycosylation = glycosylation_capacity × 0.95 (предпочтительный субстрат).
    pub cep164_glycosylation: f32,

    /// Скорость везикулярного трафика (COPI/COPII) к цилиям [0..1].
    ///
    /// Антероградный транспорт карго (IFT-компоненты, CEP164 precursor,
    /// INPP5E → фосфоинозитидный сигналинг реснички).
    /// traffic_rate = glycosylation_capacity × 0.90.
    pub vesicle_trafficking_rate: f32,
}

impl GolgiState {
    /// Молодое физиологическое состояние Гольджи.
    pub fn pristine() -> Self {
        Self {
            fragmentation_index:    0.05, // небольшая фоновая фрагментация
            glycosylation_capacity: 0.96, // (1 - 0.05×0.85) = 0.957 ≈ 0.96
            cep164_glycosylation:   0.91, // 0.96 × 0.95
            vesicle_trafficking_rate: 0.86, // 0.96 × 0.90
        }
    }

    /// Пересчитать производные метрики из fragmentation_index.
    pub fn update_derived(&mut self) {
        self.glycosylation_capacity =
            (1.0 - self.fragmentation_index * 0.85).clamp(0.10, 1.0);
        self.cep164_glycosylation =
            (self.glycosylation_capacity * 0.95).clamp(0.0, 1.0);
        self.vesicle_trafficking_rate =
            (self.glycosylation_capacity * 0.90).clamp(0.0, 1.0);
    }

    /// Дополнительная скорость потери CEP164 из-за гипогликозилирования [/год].
    ///
    /// Нормальный CEP164 (glycosylation=1.0): extra_loss=0.
    /// При glycosylation=0.5: extra_loss = 0.5 × sensitivity.
    pub fn cep164_extra_loss_rate(&self, sensitivity: f32) -> f32 {
        (1.0 - self.cep164_glycosylation) * sensitivity
    }
}

impl Default for GolgiState {
    fn default() -> Self {
        Self::pristine()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ATPEnergyState — ECS-компонент (Уровень -3: молекулы — энергетика)
//
// Моделирует энергетический статус клетки через ATP/ADP-баланс.
// Протеасома — АТФ-зависимый комплекс: при снижении energy_charge
// её активность падает → убиквитинированные белки накапливаются.
//
// Связи:
//   MitochondrialState.membrane_potential → atp_production_rate
//   energy_charge < 0.7 → proteasome_activity_modifier < 1.0
//   proteasome_activity_modifier → масштабирует ProteostasisState.proteasome_activity
// ─────────────────────────────────────────────────────────────────────────────

/// Энергетическое состояние клетки (Уровень -3: молекулы).
///
/// ATP/ADP-баланс определяет эффективность протеасомной деградации
/// и синтез новых белков-придатков (CEP164, HSP70).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATPEnergyState {
    /// Отношение ATP/ADP [0..1].
    ///
    /// Молодая клетка: ~0.90 (высокий АТФ).
    /// Старая/повреждённая: ~0.50–0.60 (митохондриальная недостаточность).
    pub atp_adp_ratio: f32,

    /// Энергетический заряд клетки: (ATP + 0.5×ADP) / (ATP+ADP+AMP) [0..1].
    ///
    /// Нормальное значение: 0.80–0.95 (Atkinson 1968).
    /// При < 0.5: нарушение активного транспорта, синтеза белков, митоза.
    pub energy_charge: f32,

    /// Производная: модификатор активности протеасомы [0..1].
    ///
    /// = min(1.0, energy_charge / 0.70)
    /// При energy_charge ≥ 0.70: proteasome работает нормально (=1.0).
    /// При energy_charge = 0.35: proteasome работает вполовину силы (=0.50).
    pub proteasome_activity_modifier: f32,
}

impl ATPEnergyState {
    /// Молодая клетка с высоким АТФ.
    pub fn pristine() -> Self {
        Self {
            atp_adp_ratio:               0.90,
            energy_charge:               0.90,
            proteasome_activity_modifier: 1.0,
        }
    }

    /// Пересчитать производную метрику.
    pub fn update_derived(&mut self) {
        self.proteasome_activity_modifier =
            (self.energy_charge / 0.70).min(1.0).max(0.0);
    }
}

impl Default for ATPEnergyState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// ChromatinState — ECS-компонент (Уровень -3: молекулы — 3D-геном)
//
// Моделирует целостность TAD-структуры (топологически ассоциированных доменов)
// и состояние хроматина. Нарушение TAD → DDR-гены недоступны → медленная репарация.
//
// Связи:
//   EpigeneticClockState.methylation_age → tad_integrity (деметилирование нарушает TAD)
//   heterochromatin_fraction↓ → SAHF разрушение → SASP↑
//   dna_accessibility↑ → DDRState.gamma_h2ax реагирует сильнее
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние хроматина стволовой ниши (Уровень -3: молекулы).
///
/// Отражает организацию 3D-генома: TAD-целостность и
/// фракционирование хроматина (еухроматин vs гетерохроматин).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromatinState {
    /// Целостность TAD-структуры [0..1].
    ///
    /// TAD = топологически ассоциированные домены — единицы 3D-организации генома.
    /// Нарушение при: деметилировании CTCF-сайтов, коге́зиновой дисфункции.
    /// Снижается с methylation_age: tad_integrity ≈ 1 − methylation_age × 0.6.
    pub tad_integrity: f32,

    /// Фракция конститутивного гетерохроматина [0..1].
    ///
    /// Молодая клетка: ~0.30 (≈30% генома — гетерохроматин).
    /// При старении: SAHF (Senescence-Associated Heterochromatin Foci)
    /// разрушаются → гетерохроматин теряет компактность → frac↓.
    pub heterochromatin_fraction: f32,

    /// Доступность ДНК для DDR-машинерии [0..1].
    ///
    /// При потере гетерохроматина: дезорганизованный хроматин открывает
    /// больше двунитевых разрывов → DDRState.gamma_h2ax растёт быстрее.
    /// dna_accessibility = 1 − heterochromatin_fraction × 0.7 + tad_breakdown × 0.3
    pub dna_accessibility: f32,
}

impl ChromatinState {
    /// Молодая клетка с интактным 3D-геномом.
    pub fn pristine() -> Self {
        let het = 0.30_f32;
        let tad = 1.0_f32;
        Self {
            tad_integrity:            tad,
            heterochromatin_fraction: het,
            dna_accessibility:        Self::calc_accessibility(tad, het),
        }
    }

    fn calc_accessibility(tad: f32, het: f32) -> f32 {
        (1.0 - het * 0.7 + (1.0 - tad) * 0.3).clamp(0.0, 1.0)
    }

    /// Пересчитать dna_accessibility из текущих полей.
    pub fn update_derived(&mut self) {
        self.dna_accessibility =
            Self::calc_accessibility(self.tad_integrity, self.heterochromatin_fraction);
    }
}

impl Default for ChromatinState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// IFTState — ECS-компонент (Уровень -2: цитоскелет — внутрижгутиковый транспорт)
//
// Моделирует IFT (Intraflagellar Transport) — молекулярный транспорт
// внутри первичной реснички.
//
// Связи:
//   AppendageProteinState.cep164 → anterograde_velocity (IFT docking)
//   GolgiState.vesicle_trafficking_rate → cargo_delivery (IFT грузы из Гольджи)
//   cargo_delivery → ciliary_function (SMO, GLI транспортируются в реснички)
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние внутрижгутикового транспорта (Уровень -2: цитоскелет).
///
/// IFT — молекулярная магистраль первичной реснички.
/// Без IFT Shh-рецепторы (SMO, PTCH) не могут достичь кончика реснички.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IFTState {
    /// Антероградная скорость IFT-B (основание → кончик реснички) [0..1].
    ///
    /// Зависит от CEP164 (IFT docking в переходной зоне) и
    /// целостности субдистальных придатков (Ninein — якорение MT).
    pub anterograde_velocity: f32,

    /// Ретроградная скорость IFT-A (кончик → основание) [0..1].
    ///
    /// Более устойчива: нарушается позже при старении.
    /// При ретроградном дефиците: аккумуляция грузов в кончике → bulge.
    pub retrograde_velocity: f32,

    /// Эффективность доставки сигнальных грузов (SMO, GLI) [0..1].
    ///
    /// = min(anterograde, retrograde) × vesicle_availability
    /// Определяет реальный Shh-ответ (выше, чем только CEP164-присутствие).
    pub cargo_delivery: f32,
}

impl IFTState {
    /// Молодая клетка с активным IFT.
    pub fn pristine() -> Self {
        Self {
            anterograde_velocity: 1.0,
            retrograde_velocity:  1.0,
            cargo_delivery:       1.0,
        }
    }

    /// Обновить cargo_delivery из скоростей и доступности везикул Гольджи.
    pub fn update_derived(&mut self, vesicle_availability: f32) {
        self.cargo_delivery =
            (self.anterograde_velocity.min(self.retrograde_velocity) * vesicle_availability)
                .clamp(0.0, 1.0);
    }
}

impl Default for IFTState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// ActinRingState — ECS-компонент (Уровень -2: цитоскелет — цитокинез)
//
// Моделирует актиновое кольцо цитокинеза.
// ROS окисляют актин → деполимеризация → неполный цитокинез
// → бинуклеарность/анеуплоидия → усиление геномной нестабильности.
//
// Связи:
//   ROSCascadeState.hydroxyl_radical → снижает actin_polymerization_rate
//   contractile_ring_integrity < 0.5 → вероятность неполного цитокинеза↑
//   неполный цитокинез → CentriolarDamageState.phosphorylation_dysregulation↑
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние актинового кольца цитокинеза (Уровень -2: цитоскелет).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActinRingState {
    /// Целостность сократительного актинового кольца [0..1].
    ///
    /// 1.0 = нормальный цитокинез. < 0.5 = риск неполного разделения.
    /// Снижается при: ROS-окислении актина, Cofilin-дисрегуляции.
    pub contractile_ring_integrity: f32,

    /// Скорость полимеризации актина [0..1].
    ///
    /// Снижается при: ROS (окисление Cys374 β-актина → ингибирование).
    /// Нормальная: 1.0. При OH·=0.5: actin_poly ≈ 0.70.
    pub actin_polymerization_rate: f32,

    /// Вероятность неполного цитокинеза за шаг [0..1].
    ///
    /// = max(0, 1 − contractile_ring_integrity) × 0.3
    /// Производная метрика для расчёта PLK4-дисрегуляции.
    pub incomplete_cytokinesis_prob: f32,
}

impl ActinRingState {
    pub fn pristine() -> Self {
        Self {
            contractile_ring_integrity:  1.0,
            actin_polymerization_rate:   1.0,
            incomplete_cytokinesis_prob: 0.0,
        }
    }

    pub fn update_derived(&mut self) {
        self.incomplete_cytokinesis_prob =
            ((1.0 - self.contractile_ring_integrity) * 0.3).clamp(0.0, 1.0);
    }
}

impl Default for ActinRingState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// ERStressState — ECS-компонент (Уровень -1: органоиды — ЭПС)
//
// Эндоплазматический ретикулум (ЭПС) — главный хаперон-содержащий органоид.
// При перегрузке неправильно свёрнутыми белками → UPR (Unfolded Protein Response).
// Хронический UPR → апоптоз через CHOP (DDIT3).
//
// Связи:
//   ProteostasisState.proteasome_activity↓ → unfolded_protein_response↑
//   GolgiState.fragmentation_index → chaperone_saturation (Гольджи-ЭПС связаны)
//   unfolded_protein_response > 0.7 → AgingPhenotype::ERStress
//   ca2_buffer_capacity↓ → MitochondrialState: Ca²⁺-перегрузка → mPTP открытие
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние ЭПС и UPR-ответа (Уровень -1: органоиды).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERStressState {
    /// Активация ответа на неправильно свёрнутые белки (UPR) [0..1].
    ///
    /// 0.0 = нет стресса (GRP78/BiP связывают все клиенты).
    /// > 0.5 = умеренный стресс (IRE1/ATF6/PERK ветви активны).
    /// > 0.8 = хронический стресс → CHOP → апоптоз.
    pub unfolded_protein_response: f32,

    /// Буферная ёмкость Ca²⁺ в ЭПС [0..1].
    ///
    /// ЭПС хранит ~70% внутриклеточного Ca²⁺ (Orrenius et al. 2003).
    /// При UPR: Ca²⁺ выходит в цитозоль → активация кальпаинов, mPTP.
    /// 1.0 = полная ёмкость. При upr > 0.5: capacity ≈ 1 − upr × 0.6.
    pub ca2_buffer_capacity: f32,

    /// Степень насыщения шаперонов GRP78/BiP [0..1].
    ///
    /// GRP78 — главный маркер UPR. При насыщении > 0.8: шаперонов не хватает
    /// → новые белки деградируют по ERAD или агрегируют.
    pub chaperone_saturation: f32,
}

impl ERStressState {
    pub fn pristine() -> Self {
        Self {
            unfolded_protein_response: 0.0,
            ca2_buffer_capacity:       1.0,
            chaperone_saturation:      0.0,
        }
    }

    pub fn update_derived(&mut self) {
        self.ca2_buffer_capacity =
            (1.0 - self.unfolded_protein_response * 0.6).clamp(0.1, 1.0);
    }
}

impl Default for ERStressState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// LysosomeState — ECS-компонент (Уровень -1: органоиды — лизосомы)
//
// Лизосомы = ацидные органоиды деградации (pH 4.5–5.0).
// При возрастном защелачивании → снижение гидролазной активности → аутофагия↓.
//
// Связи:
//   AutophagyState.autophagy_flux ← hydrolase_activity (лизосомы финализируют аутофагию)
//   ROSCascadeState.hydroxyl_radical → membrane_permeability↑ (LAMP2-повреждение)
//   ph_level > 5.5 → cathepsin_leakage → апоптоз-сигналинг
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние лизосом стволовой ниши (Уровень -1: органоиды).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LysosomeState {
    /// Внутрилизосомальный pH [4.5..7.0].
    ///
    /// Нормальный: 5.0. При старении: защелачивается до 5.5–6.5
    /// (v-ATPase дефицит: Settembre et al. 2013).
    /// При pH > 5.5: гидролазная активность резко падает.
    pub ph_level: f32,

    /// Активность кислых гидролаз (cathepsins B/D/L, β-hex) [0..1].
    ///
    /// = max(0, 1 − (ph_level − 5.0) × 0.40)
    /// При pH=5.0: activity=1.0. При pH=6.5: activity≈0.40.
    pub hydrolase_activity: f32,

    /// Проницаемость лизосомной мембраны [0..1].
    ///
    /// 0.0 = интактная. Увеличивается при OH·-повреждении LAMP1/LAMP2.
    /// При > 0.5: катепсины утекают в цитозоль → воспалительный каспаз-сигнал.
    pub membrane_permeability: f32,
}

impl LysosomeState {
    pub fn pristine() -> Self {
        Self {
            ph_level:             5.0,
            hydrolase_activity:   1.0,
            membrane_permeability: 0.0,
        }
    }

    pub fn update_derived(&mut self) {
        self.hydrolase_activity =
            (1.0 - (self.ph_level - 5.0) * 0.40).clamp(0.0, 1.0);
    }
}

impl Default for LysosomeState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// PeroxisomeState — ECS-компонент (Уровень -1: органоиды — пероксисомы)
//
// Пероксисомы — главный орган детоксикации H₂O₂ (каталаза).
// Каталаза снижается с возрастом → больше H₂O₂ → Fenton-реакция усиливается.
//
// Связи:
//   h2o2_clearance_rate → ROSCascadeState: снижает накопление hydrogen_peroxide
//   fatty_acid_oxidation↓ → метаболический фенотип (липидный стресс)
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние пероксисом стволовой ниши (Уровень -1: органоиды).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeroxisomeState {
    /// Активность каталазы [0..1].
    ///
    /// Снижается с возрастом: −0.4%/год после 40 лет (Tian et al. 1998).
    /// Молодая клетка: 1.0. В 80 лет: ~0.60.
    pub catalase_activity: f32,

    /// Скорость удаления H₂O₂ [0..1].
    ///
    /// = catalase_activity × 0.80 + gpx_activity × 0.20
    /// (каталаза доминирует, GPx — вспомогательная система).
    pub h2o2_clearance_rate: f32,

    /// β-окисление жирных кислот [0..1].
    ///
    /// Снижается при пероксисомной дисфункции → липидный стресс,
    /// накопление VLCFA (X-linked adrenoleukodystrophy модель).
    pub fatty_acid_oxidation: f32,
}

impl PeroxisomeState {
    pub fn pristine() -> Self {
        Self {
            catalase_activity:    1.0,
            h2o2_clearance_rate:  0.80,
            fatty_acid_oxidation: 1.0,
        }
    }

    pub fn update_derived(&mut self) {
        // Упрощённо: GPx-вклад = 0.20 × catalase (коррелированы)
        self.h2o2_clearance_rate = (self.catalase_activity * 0.80 + self.catalase_activity * 0.20)
            .clamp(0.0, 1.0);
    }
}

impl Default for PeroxisomeState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// RibosomeState — ECS-компонент (Уровень -1: органоиды — рибосомы)
//
// Рибосомы — синтетический аппарат клетки.
// Скорость трансляции определяет, как быстро восстанавливаются CEP164, HSP70.
// RQC (Ribosome Quality Control) — удаляет незавершённые полипептиды.
//
// Связи:
//   ATPEnergyState.energy_charge → translation_rate (ГТФ нужен для элонгации)
//   translation_rate → repair_rate придатков (CEP164 синтезируется de novo)
//   ribosome_quality↓ → больше аберрантных белков → ProteostasisState нагрузка↑
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние рибосомального аппарата (Уровень -1: органоиды).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RibosomeState {
    /// Скорость трансляции (нормированная) [0..1].
    ///
    /// Снижается при: энергетическом дефиците (АТФ/ГТФ↓),
    /// окислении рибосомальных белков, rRNA-повреждениях.
    pub translation_rate: f32,

    /// Качество рибосомального контроля (RQC) [0..1].
    ///
    /// RQC = Ltn1/NEMF/VCP комплекс: спасает застрявшие рибосомы.
    /// При RQC < 0.5: незавершённые полипептиды → агрегаты ↑.
    pub ribosome_quality: f32,

    /// Доступность аминоацил-тРНК [0..1].
    ///
    /// Снижается при: дефиците аминокислот, mTOR-ингибировании.
    /// CR (caloric restriction): aminoacyl_availability ≈ 0.75 → mTOR↓ →
    /// замедление трансляции → меньше аберрантных белков.
    pub aminoacyl_availability: f32,
}

impl RibosomeState {
    pub fn pristine() -> Self {
        Self {
            translation_rate:        1.0,
            ribosome_quality:        1.0,
            aminoacyl_availability:  1.0,
        }
    }
}

impl Default for RibosomeState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// ExtracellularMatrixState — ECS-компонент (Уровень +1: ткань — матрикс)
//
// Внеклеточный матрикс (ECM) — физический контекст стволовой ниши.
// Жёсткий матрикс → YAP/TAZ-сигналинг → симметричные деления.
//
// Связи:
//   ROS → collagen_crosslinking (LOX-фермент окисляется и инактивируется,
//          а AGE накапливаются → перекрёстные связи)
//   stiffness → integrin_signaling → влияет на тип деления
//   fibrosis: collagen_deposition → functional_capacity ниши↓
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние внеклеточного матрикса (Уровень +1: ткань).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtracellularMatrixState {
    /// Степень перекрёстного сшивания коллагена [0..1].
    ///
    /// 0.0 = молодой эластичный матрикс.
    /// Растёт из-за: AGE (Advanced Glycation End-products) + LOX/LOXL.
    pub collagen_crosslinking: f32,

    /// Механическая жёсткость матрикса [0..1].
    ///
    /// = collagen_crosslinking × 0.70 + fibrosis_contribution × 0.30
    /// Жёсткий матрикс (>0.6): YAP/TAZ ядерный → симметричные деления.
    pub stiffness: f32,

    /// Сила механосигналинга через интегрины [0..1].
    ///
    /// = stiffness × integrin_expression (интегрины αV/β3, α5/β1).
    /// Снижается при потере интегрин-ECM взаимодействий в старой нише.
    pub integrin_signaling: f32,
}

impl ExtracellularMatrixState {
    pub fn pristine() -> Self {
        Self {
            collagen_crosslinking: 0.0,
            stiffness:             0.0,
            integrin_signaling:    0.5,  // базовый механосигналинг
        }
    }

    pub fn update_derived(&mut self) {
        self.stiffness = (self.collagen_crosslinking * 0.70).clamp(0.0, 1.0);
        self.integrin_signaling = (self.stiffness * 0.60 + 0.2).clamp(0.0, 1.0);
    }
}

impl Default for ExtracellularMatrixState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// VascularNicheState — ECS-компонент (Уровень +1: ткань — сосудистая ниша)
//
// Сосудистая ниша определяет O₂-доступность и концентрацию факторов роста.
// При старении: ангиогенез снижается → ниша гипоксическая → митохондрии
// усиливают гликолиз → ROS↑ → CDATA-ускорение.
//
// Связи:
//   oxygen_supply → o2_at_centriole (прямая замена или вклад в mito_shield)
//   growth_factor_gradient → regeneration_tempo ниши
//   angiogenesis_index: снижается с возрастом → oxygen_supply↓
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние сосудистой ниши (Уровень +1: ткань).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VascularNicheState {
    /// Доступность O₂ из капилляров [0..1].
    ///
    /// Молодая ниша: 1.0 (нормоксия ~21% O₂).
    /// Старая/патологическая: 0.3–0.5 (относительная гипоксия).
    pub oxygen_supply: f32,

    /// Концентрация нишевых факторов роста [0..1].
    ///
    /// HSC-ниша: SCF, THPO, CXCL12, Ang-1.
    /// NSC-ниша: FGF-2, EGF, VEGF.
    /// Снижается при редукции капиллярной сети.
    pub growth_factor_gradient: f32,

    /// Плотность капиллярной сети (ангиогенез) [0..1].
    ///
    /// Снижается с возрастом: VEGF↓, HIF-1α дисрегуляция.
    /// Определяет oxygen_supply и growth_factor_gradient.
    pub angiogenesis_index: f32,
}

impl VascularNicheState {
    pub fn pristine() -> Self {
        Self {
            oxygen_supply:          1.0,
            growth_factor_gradient: 1.0,
            angiogenesis_index:     1.0,
        }
    }

    pub fn update_derived(&mut self) {
        self.oxygen_supply          = self.angiogenesis_index.clamp(0.0, 1.0);
        self.growth_factor_gradient = self.angiogenesis_index.clamp(0.0, 1.0);
    }
}

impl Default for VascularNicheState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// FibrosisState — ECS-компонент (Уровень +1: ткань — фиброз)
//
// Фиброз = замещение паренхимы фиброзной тканью.
// SASP → TGF-β → активация миофибробластов → коллагеновые рубцы.
// Замещённая ткань не может выполнять регенеративную функцию.
//
// Связи:
//   InflammagingState.sasp_intensity → fibroblast_activation
//   functional_replacement → functional_capacity ниши↓ (прямой эффект)
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние фиброза ниши (Уровень +1: ткань).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FibrosisState {
    /// Активность миофибробластов [0..1].
    ///
    /// Активируются TGF-β1/3 из SASP.
    /// Производят коллаген I/III, фибронектин.
    pub fibroblast_activation: f32,

    /// Скорость отложения коллагена [0..1].
    ///
    /// = fibroblast_activation × 0.8
    pub collagen_deposition_rate: f32,

    /// Доля паренхимы, замещённой фиброзной тканью [0..1].
    ///
    /// Накапливается необратимо: integral of collagen_deposition_rate × dt.
    /// Прямо снижает functional_capacity: fc × (1 − functional_replacement × 0.8).
    pub functional_replacement: f32,
}

impl FibrosisState {
    pub fn pristine() -> Self {
        Self {
            fibroblast_activation:    0.0,
            collagen_deposition_rate: 0.0,
            functional_replacement:   0.0,
        }
    }

    pub fn update_derived(&mut self) {
        self.collagen_deposition_rate = (self.fibroblast_activation * 0.8).clamp(0.0, 1.0);
    }

    /// Вклад фиброза в снижение functional_capacity.
    pub fn fc_penalty(&self) -> f32 {
        (self.functional_replacement * 0.8).clamp(0.0, 1.0)
    }
}

impl Default for FibrosisState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// HPAAxisState — ECS-компонент (Уровень +3: организм — нейроэндокринная ось)
//
// Гипоталамо-гипофизарно-надпочечниковая ось (HPA).
// Хронический стресс → кортизол↑ → иммуносупрессия + инсулинорезистентность.
//
// Связи:
//   cortisol_level → InflammagingState: иммуносупрессия → нарушение SASP-клиренса
//   hpa_reactivity × chronic_stress → ускоренное старение HSC (Flach et al. 2014)
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние HPA-оси (Уровень +3: организм).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HPAAxisState {
    /// Базальный уровень кортизола [0..1].
    ///
    /// 0.3 = норма (молодой взрослый).
    /// > 0.6 = хроническая гиперкортизолемия → иммуносупрессия.
    pub cortisol_level: f32,

    /// Реактивность HPA-оси (CRH/ACTH чувствительность) [0..1].
    ///
    /// Снижается при хронической активации (истощение оси).
    /// Слишком высокая → паника/тревога. Слишком низкая → адреналиновая недостаточность.
    pub hpa_reactivity: f32,

    /// Индекс хронического стресса [0..1].
    ///
    /// Интеграл кортизоль-нагрузки за время. Необратим как эпигенетический след.
    pub chronic_stress_index: f32,
}

impl HPAAxisState {
    pub fn pristine() -> Self {
        Self {
            cortisol_level:      0.30,
            hpa_reactivity:      0.70,
            chronic_stress_index: 0.0,
        }
    }
}

impl Default for HPAAxisState {
    fn default() -> Self { Self::pristine() }
}

// ─────────────────────────────────────────────────────────────────────────────
// MetabolicPhenotypeState — ECS-компонент (Уровень +3: организм — метаболизм)
//
// Метаболический фенотип: BMI/ожирение → адипокины → воспаление → CDATA.
// Лептин/адипонектин дисбаланс → хроническая ИМТ-зависимая inflammaging.
//
// Связи:
//   adipokine_level > 0.6 → InflammagingState: ros_boost↑, sasp↑
//   insulin_sensitivity↓ → энергетический голод клетки → ATPEnergyState.energy_charge↓
// ─────────────────────────────────────────────────────────────────────────────

/// Метаболический фенотип организма (Уровень +3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicPhenotypeState {
    /// Нормированный ИМТ-индекс [0..1].
    ///
    /// 0.0 = норма (ИМТ 22). 1.0 = тяжёлое ожирение (ИМТ ≥ 40).
    pub bmi_index: f32,

    /// Уровень провоспалительных адипокинов (лептин, резистин) [0..1].
    ///
    /// = bmi_index × 0.70 + chronic_stress × 0.30
    /// При > 0.5: inflammaging↑, SASP усиливается.
    pub adipokine_level: f32,

    /// Чувствительность к инсулину [0..1].
    ///
    /// 1.0 = норма. Снижается при ожирении (ИМТ > 0.4 → ≈0.50).
    /// Инсулинорезистентность → клетки голодают → ATPEnergyState.energy_charge↓.
    pub insulin_sensitivity: f32,
}

impl MetabolicPhenotypeState {
    pub fn pristine() -> Self {
        Self {
            bmi_index:            0.0,
            adipokine_level:      0.0,
            insulin_sensitivity:  1.0,
        }
    }

    pub fn update_derived(&mut self) {
        self.adipokine_level = (self.bmi_index * 0.70).clamp(0.0, 1.0);
        self.insulin_sensitivity = (1.0 - self.bmi_index * 0.60).clamp(0.1, 1.0);
    }
}

impl Default for MetabolicPhenotypeState {
    fn default() -> Self { Self::pristine() }
}

/// Тип ткани для специфики стволовых ниш.
///
/// Объединяет биологические ниши (`Neural`, `Muscle`, …) и
/// тканеспецифичные типы клеток человека (`Liver`, `Kidney`, …).
/// Ранее дублировался как `HumanTissueType` в `human_development_module`
/// — теперь единый тип; `HumanTissueType` является псевдонимом.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TissueType {
    // ── Биологические ниши ────────────────────────────────────────────────
    /// Нейральные стволовые клетки (СВЗ, зубчатая извилина)
    Neural,
    /// Кровь / гемопоэтические стволовые клетки (красный костный мозг)
    Blood,
    /// Эпителиальные ниши (кишечные крипты, кожный базальный слой и т.п.)
    Epithelial,
    /// Мышечные клетки-сателлиты
    Muscle,
    /// Кожный эпителий (базальный слой)
    Skin,
    /// Половые клетки
    Germline,
    // ── Специфичные для человека ─────────────────────────────────────────
    /// Соединительная ткань
    Connective,
    /// Костная ткань
    Bone,
    /// Хрящевая ткань
    Cartilage,
    /// Жировая ткань
    Adipose,
    /// Печень (гепатоциты / звёздчатые клетки)
    Liver,
    /// Почки (тубулярный эпителий)
    Kidney,
    /// Сердечная мышца (кардиомиоциты)
    Heart,
    /// Лёгочный эпителий (альвеолярный тип II)
    Lung,
}

/// Правило центриолярной асимметрии при делении стволовой клетки.
///
/// Определяет, какая из центриолей (материнская или дочерняя) наследуется
/// дочерней клеткой, остающейся в стволовом состоянии.
///
/// Источники: «Centrioles as determinants» (2026-01-27),
///            «Strategic Timekeepers» (2026-01-15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CentrioleAsymmetryRule {
    /// Материнская центриоль → стволовая дочь (HSC, нейральная радиальная глия,
    /// зародышевые клетки млекопитающих, гепатоциты).
    ///
    /// Следствие для CDATA: материнский комплект (M-set) несёт «историю» ниши
    /// и теряет индукторы быстрее (больше ПТМ, слабее связи) → `mother_bias > 0.5`.
    MotherToStem,
    /// Дочерняя центриоль → стволовая дочь (нейробласты Drosophila).
    ///
    /// Редкое исключение. Для млекопитающих не характерно.
    /// Следствие для CDATA: D-set = стволовой, поэтому D теряет медленнее → `mother_bias < 0.5`.
    DaughterToStem,
    /// Симметричное деление: обе центриоли равнозначны.
    ///
    /// Характерно для эпителия, мышечных сателлитов, кожи.
    /// `mother_bias = 0.5`.
    Symmetric,
}

impl TissueType {
    /// Правило асимметрии центриолей для данного типа ткани.
    pub fn asymmetry_rule(self) -> CentrioleAsymmetryRule {
        use CentrioleAsymmetryRule::*;
        match self {
            // MotherToStem: материнская → долгосрочная стволовая ниша
            TissueType::Blood     => MotherToStem, // LT-HSC наследует мать. (Hinge et al. 2020)
            TissueType::Neural    => MotherToStem, // радиальная глия → нейрогенная нишa
            TissueType::Germline  => MotherToStem, // зародышевые стволовые клетки
            TissueType::Liver     => MotherToStem, // гепатоциты зоны 1 (перипортальные)
            // Symmetric: симметричное деление, оба пула равнозначны
            TissueType::Epithelial   => Symmetric,
            TissueType::Muscle       => Symmetric,
            TissueType::Skin         => Symmetric,
            TissueType::Connective   => Symmetric,
            TissueType::Bone         => Symmetric,
            TissueType::Cartilage    => Symmetric,
            TissueType::Adipose      => Symmetric,
            TissueType::Kidney       => Symmetric,
            TissueType::Heart        => Symmetric,
            TissueType::Lung         => Symmetric,
        }
    }

    /// Ткань-специфичный `mother_bias` для `InducerDetachmentParams`.
    ///
    /// Значение определяется правилом асимметрии:
    /// - `MotherToStem` → 0.65 (мать старше, больше ПТМ, связи слабее → теряет чаще)
    /// - `DaughterToStem` → 0.35 (дочь = стволовая, мать теряет меньше)
    /// - `Symmetric` → 0.50
    pub fn default_mother_bias(self) -> f32 {
        match self.asymmetry_rule() {
            CentrioleAsymmetryRule::MotherToStem   => 0.65,
            CentrioleAsymmetryRule::DaughterToStem => 0.35,
            CentrioleAsymmetryRule::Symmetric      => 0.50,
        }
    }
}

/// Состояние ткани — агрегированные метрики регенеративного потенциала
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TissueState {
    pub tissue_type: TissueType,
    /// Размер пула стволовых клеток (0..1, относительно молодого организма)
    pub stem_cell_pool: f32,
    /// Темп регенерации (0..1, относительно молодого организма).
    /// Вычисляется через Hill-функцию GLI (нелинейно от ciliary_function).
    pub regeneration_tempo: f32,
    /// Доля сенесцентных клеток [0..1]
    pub senescent_fraction: f32,
    /// Средний возраст материнской центриоли в нише (в делениях)
    pub mean_centriole_age: f32,
    /// Функциональная ёмкость ткани [0..1]
    pub functional_capacity: f32,

    // ── Морфогенные поля (P13) ────────────────────────────────────────────
    /// Локальная концентрация Sonic Hedgehog [0..1] в нише.
    /// Продуцируется поддерживающими клетками, потребляется через PTCH1 на реснички.
    /// Снижается с возрастом: cilia loss → нарушение рецепции → градиент размывается.
    pub shh_concentration: f32,
    /// Баланс BMP-активность / Noggin-ингибирование [0..1].
    /// 0.0 = полный Noggin-эффект (стволовость); 1.0 = максимальная BMP (дифференцировка).
    /// С возрастом: cilia loss → Noggin↓ → BMP_balance↑ → дифференцировка↑.
    pub bmp_balance: f32,
    /// Активность Wnt-пути в нише [0..1].
    /// Зависит от stem_cell_pool (больше клеток → больше Wnt-лигандов) и ресничек.
    pub wnt_activity: f32,
    /// Активация GLI (Hedgehog-ответ через ресничку) [0..1].
    /// Hill-нелинейная функция от ciliary_function, вычисляется в update_tissue_state.
    pub gli_activation: f32,
}

impl TissueState {
    pub fn new(tissue_type: TissueType) -> Self {
        Self {
            tissue_type,
            stem_cell_pool: 1.0,
            regeneration_tempo: 1.0,
            senescent_fraction: 0.0,
            mean_centriole_age: 0.0,
            functional_capacity: 1.0,
            // Морфогенные поля — инициализируются "молодым" паттерном
            shh_concentration: 0.8,  // высокий Shh в молодой нише
            bmp_balance: 0.3,        // умеренный BMP (Noggin преобладает)
            wnt_activity: 0.7,       // активный Wnt → самообновление
            gli_activation: 0.8,     // полный GLI-ответ при cilia=1.0
        }
    }

    /// Обновить функциональную ёмкость из текущих метрик
    pub fn update_functional_capacity(&mut self) {
        self.functional_capacity = self.stem_cell_pool
            * self.regeneration_tempo
            * (1.0 - self.senescent_fraction * 0.8);
    }

    /// Обновить морфогенные поля на основе текущего состояния ниши.
    ///
    /// Вызывается из `HumanDevelopmentModule::update_tissue_state()`.
    ///
    /// # Аргументы
    /// * `ciliary_function` — из `CentriolarDamageState`
    /// * `detail_level` — `tissue_detail_level` из `HumanDevelopmentParams`:
    ///   - 1: только GLI-активация (минимум, быстро)
    ///   - 2: GLI + BMP_balance
    ///   - 3+: полный морфогенный профиль (GLI + BMP + Wnt + Shh)
    pub fn update_morphogen_fields(&mut self, ciliary_function: f32, detail_level: usize) {
        // Уровень 1+: GLI-активация (Hedgehog) через Hill-нелинейность реснички.
        // Это основной механизм Трека A — всегда вычисляется.
        const K: f32 = 0.5;
        const N: f32 = 2.0;
        let c = ciliary_function;
        self.gli_activation = c.powf(N) / (K.powf(N) + c.powf(N));

        if detail_level < 2 { return; }

        // Уровень 2+: BMP/Noggin баланс.
        // С потерей ресничек падает Noggin → BMP-активность растёт → дифференцировка↑.
        // Биологически: Noggin-экспрессия в нише зависит от Hedgehog через ресничку.
        let noggin_activity = ciliary_function * 0.7;
        self.bmp_balance = ((1.0 - noggin_activity) * 0.8
            + self.senescent_fraction * 0.2)
            .clamp(0.0, 1.0);

        if detail_level < 3 { return; }

        // Уровень 3+: полный морфогенный профиль (Shh + Wnt).
        // Shh: продукция ∝ пулу стволовых клеток + вкладу Wnt
        self.shh_concentration = (self.stem_cell_pool * 0.8
            + self.wnt_activity * 0.2)
            .clamp(0.0, 1.0);

        // Wnt: синергия с GLI (Wnt-Hedgehog crosstalk в нише HSC/NSC)
        self.wnt_activity = (self.stem_cell_pool * 0.6
            + self.gli_activation * 0.4)
            .clamp(0.0, 1.0);
    }
}

/// Глобальное состояние организма (уровень организм/особь)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganismState {
    /// Возраст в годах
    pub age_years: f64,
    /// Текущая стадия развития
    pub developmental_stage: DevelopmentalStage,
    /// Накопленный уровень системного воспаления (inflammaging) [0..1]
    pub inflammaging_score: f32,
    /// Интегральный индекс дряхлости [0..1]
    pub frailty_index: f32,
    /// Когнитивный индекс [0..1]
    pub cognitive_index: f32,
    /// Иммунный резерв [0..1]
    pub immune_reserve: f32,
    /// Мышечная масса (саркопения) [0..1]
    pub muscle_mass: f32,
    /// Жив ли организм
    pub is_alive: bool,
    /// Уровень ИФР-1/ГР (ось IGF-1/GH) [0..1].
    /// Пик в ~20 лет, линейное снижение до 0.3 к 90 годам.
    /// Влияет на `regeneration_tempo` всех тканей.
    pub igf1_level: f32,
    /// Уровень системного SASP [0..1] — среднее sasp_output всех ниш.
    /// Паракринный сигнал: ускоряет повреждения соседних тканей.
    pub systemic_sasp: f32,
    /// CAII-индекс организма [0..1] — среднее CAII по всем нишам.
    /// 1.0 = все ниши здоровы, 0.0 = полное повреждение центриолей.
    pub caii_organism: f32,
    /// Биологический возраст [лет] = хронологический × (1 + (1 − CAII) × 0.50).
    pub biological_age: f32,
}

impl OrganismState {
    pub fn new() -> Self {
        Self {
            age_years: 0.0,
            developmental_stage: DevelopmentalStage::Zygote,
            inflammaging_score: 0.0,
            frailty_index: 0.0,
            cognitive_index: 1.0,
            immune_reserve: 1.0,
            muscle_mass: 1.0,
            is_alive: true,
            igf1_level: 1.0,
            systemic_sasp: 0.0,
            caii_organism: 1.0,
            biological_age: 0.0,
        }
    }
}

impl Default for OrganismState {
    fn default() -> Self {
        Self::new()
    }
}

/// Маркер мёртвой сущности.
///
/// Вставляется модулями (например, `human_development_module`) при гибели клетки.
/// `SimulationManager::cleanup_dead_entities()` периодически удаляет все сущности
/// с этим компонентом из ECS-мира.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dead;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_enum() {
        let phases = vec![Phase::G1, Phase::S, Phase::G2, Phase::M];
        assert_eq!(phases.len(), 4);
    }

    #[test]
    fn test_centriole_creation() {
        let mother = Centriole::new_mature();
        let daughter = Centriole::new_daughter();
        
        assert_eq!(mother.maturity, 1.0);
        assert_eq!(daughter.maturity, 0.0);
        assert_eq!(mother.associated_cafds.len(), 0);
    }

    #[test]
    fn test_centriole_pair_default() {
        let pair = CentriolePair::default();
        assert_eq!(pair.mother.maturity, 1.0);
        assert_eq!(pair.daughter.maturity, 0.0);
        assert_eq!(pair.mtoc_activity, 0.5);
        assert!(!pair.cilium_present);
    }

    #[test]
    fn test_cafd_creation() {
        let cafd = CAFD::new("YAP");
        assert_eq!(cafd.name, "YAP");
        assert_eq!(cafd.activity, 0.0);
        assert_eq!(cafd.concentration, 0.0);
    }

    #[test]
    fn test_ptm_profile_default() {
        let ptm = PTMProfile::default();
        assert_eq!(ptm.acetylation_level, 0.0);
        assert_eq!(ptm.oxidation_level, 0.0);
    }
}

// ---------------------------------------------------------------------------
// Inflammaging — канал обратной связи myeloid_shift_module → human_development_module
// ---------------------------------------------------------------------------

/// Состояние воспалительного старения (inflammaging).
///
/// Пишется из `myeloid_shift_module` каждый шаг.
/// Читается из `human_development_module` для коррекции скорости повреждений.
/// При отсутствии `myeloid_shift_module` компонент остаётся нулевым — поведение как раньше.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InflammagingState {
    /// Дополнительный множитель скорости ROS-повреждения [0..0.5].
    /// Применяется как: `effective_ros_rate = base_ros_rate × (1 + ros_boost)`
    pub ros_boost: f32,
    /// Снижение темпа регенерации ниши [0..0.5].
    /// Применяется как: `regeneration_tempo *= (1 - niche_impairment)`
    pub niche_impairment: f32,
    /// Интенсивность SASP (Senescence-Associated Secretory Phenotype) [0..1].
    pub sasp_intensity: f32,
}

/// Shared ECS-компонент статистики делений.
///
/// Пишется из `asymmetric_division_module` каждый шаг.
/// Читается из `human_development_module` для коррекции `stem_cell_pool`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DivisionExhaustionState {
    /// Число делений типа Differentiation (оба потомка дифференцируются — истощение пула).
    pub exhaustion_count: u32,
    /// Число асимметричных делений (нормальных).
    pub asymmetric_count: u32,
    /// Суммарное число завершённых делений.
    pub total_divisions: u32,
}

impl DivisionExhaustionState {
    /// Доля делений-истощений [0..1].
    /// 0 — только асимметричные; 1 — только дифференцировка.
    pub fn exhaustion_ratio(&self) -> f32 {
        let total = self.exhaustion_count + self.asymmetric_count;
        if total == 0 { 0.0 } else { self.exhaustion_count as f32 / total as f32 }
    }
}

/// Shared ECS-компонент для ключевых уровней экспрессии генов.
///
/// Пишется из `transcriptome_module` каждый шаг.
/// Читается из `cell_cycle_module` для p21/p16-арестов и модуляции G1.
/// При отсутствии `transcriptome_module` компонент остаётся дефолтным — поведение как раньше.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneExpressionState {
    /// CDKN1A (p21) — ингибитор Cdk1/2/4/6 [0..1].
    /// p21 > 0.7 → временный G1/S арест (ДНК-повреждение, стресс).
    pub p21_level: f32,
    /// CDKN2A (p16/INK4a) — ингибитор Cdk4/6 [0..1].
    /// p16 > 0.8 → постоянный арест (сенесценция).
    pub p16_level: f32,
    /// CCND1 (Cyclin D1) — промотор G1→S перехода [0..1].
    /// Высокий уровень укорачивает G1.
    pub cyclin_d_level: f32,
    /// CCNE1/2 (Cyclin E) — промотор G1→S (поздняя G1) [0..1].
    /// Синергичен с Cyclin D: ускоряет переход G1→S при высоких значениях.
    /// Записывается из `transcriptome_module` (при наличии), иначе дефолтный.
    pub cyclin_e_level: f32,
    /// MYC — общий транскрипционный активатор пролиферации [0..1].
    pub myc_level: f32,
}

impl Default for GeneExpressionState {
    fn default() -> Self {
        Self {
            p21_level:      0.0,
            p16_level:      0.0,
            cyclin_d_level: 0.5, // умеренный базальный уровень
            cyclin_e_level: 0.4, // умеренный базальный уровень
            myc_level:      0.3,
        }
    }
}

// ---------------------------------------------------------------------------
// Трек C: Теломеры
// ---------------------------------------------------------------------------

/// Состояние теломер стволовой клетки (Трек C CDATA).
///
/// Теломеры укорачиваются при каждом делении (лимит Хейфлика).
/// В рамках CDATA ускорение укорачивания обусловлено:
/// - `spindle_fidelity ↓` → хромосомная нестабильность → двойные разрывы у теломер
/// - `ros_level ↑` → окислительное повреждение теломерной ДНК
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelomereState {
    /// Средняя длина теломер в единицах T/S ratio [0..1]. Зигота = 1.0.
    pub mean_length: f32,
    /// Укорачивание за одно деление (≈50 п.н. → ~0.002 в T/S единицах).
    pub shortening_per_division: f32,
    /// true когда mean_length < 0.3 (Хейфликовский предел → сенесценция).
    pub is_critically_short: bool,
}

impl Default for TelomereState {
    fn default() -> Self {
        Self {
            mean_length: 1.0,
            shortening_per_division: 0.002,
            is_critically_short: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Трек D: Эпигенетические часы
// ---------------------------------------------------------------------------

/// Эпигенетические часы (Трек D CDATA) — биологический возраст по CpG-метилированию.
///
/// `methylation_age` догоняет хронологический возраст в молодости,
/// обгоняет его при высоком суммарном повреждении центриоли.
/// Ускорение часов отражает кумулятивный молекулярный стресс.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpigeneticClockState {
    /// Биологический возраст по эпигенетическим часам (лет).
    pub methylation_age: f32,
    /// Коэффициент ускорения часов (1.0 = норма; >1.0 = ускорены).
    /// `clock_acceleration = 1.0 + total_damage_score × 0.5`
    pub clock_acceleration: f32,
    /// Вклад эпигенетического ускорения в ROS следующего шага [0..0.05].
    /// Аналог `InflammagingState::ros_boost`, но от эпигенетических часов.
    /// Читается в начале step() и передаётся в `accumulate_damage()` вместе с infl_ros_boost.
    pub epi_ros_contribution: f32,
    /// Число делений на момент последнего эпигенетического сброса.
    /// Используется для детекции новых делений (сравнивается с `DivisionExhaustionState::total_divisions`).
    /// При делении дочерняя клетка наследует только половину «лишнего» метилирования:
    /// `methylation_age = (methylation_age + chron_age) / 2`.
    pub last_division_count: u32,
}

impl Default for EpigeneticClockState {
    fn default() -> Self {
        Self {
            methylation_age: 0.0,
            clock_acceleration: 1.0,
            epi_ros_contribution: 0.0,
            last_division_count: 0,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Циркадный ритм (P18) — нарушение через потерю ресничек (CEP164↓)
// ──────────────────────────────────────────────────────────────────────────────

/// Состояние циркадного ритма стволовой ниши.
///
/// # Механизм связи с CDATA
///
/// Первичные реснички у стволовых клеток необходимы для трансдукции
/// ночных сигналов SCN (супрахиазматического ядра) через SHH-рецептор PTCH1
/// и цАМФ-каскад. CEP164 — ключевой белок перехода зоны — обеспечивает
/// барьер, необходимый для правильной сборки и функции реснички.
///
/// С потерей CEP164:
///  - нарушается PTCH1-канализация → Shh/Wnt сигналы ослабевают
///  - периферическая циркадная синхронизация нарушается → амплитуда↓
///  - BMAL1/CLOCK-зависимая активация протеасомы в ночной фазе снижается
///  - NF-κB получает конститутивную активацию → SASP↑
///
/// Биологические ссылки:
///  - Baggs & Green (2003): Clock proteins at centrosome
///  - Yeh et al. (2013): Primary cilia in circadian input
///  - Lipton et al. (2015): Circadian proteasome regulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircadianState {
    /// Циркадная амплитуда [0..1].
    /// 1.0 = здоровый ритм; снижается при потере CEP164 и накоплении агрегатов.
    /// `amplitude = (cep164 × 0.6 + (1 − aggregates) × 0.4) × (1 − ros × 0.2)`
    pub amplitude: f32,

    /// Когерентность клеточных часов [0..1] — синхронность BMAL1/CLOCK цикла.
    /// Нарушается при высоком ROS (окисление CLOCK-белков).
    pub phase_coherence: f32,

    /// Циркадный буст протеасомы в ночной фазе [0..1].
    /// Модулирует агрегасомный клиренс: `proteasome_night_boost = amplitude × 0.25`.
    /// В норме UPS активность на 25% выше в ночной фазе (Lipton et al. 2015).
    pub proteasome_night_boost: f32,

    /// Вклад циркадного нарушения в SASP [0..0.3].
    /// `= (1 − amplitude) × 0.3`
    /// Биологически: нарушение циркадных часов активирует NF-κB конститутивно.
    pub circadian_sasp_contribution: f32,
}

impl Default for CircadianState {
    fn default() -> Self {
        Self {
            amplitude:                  1.0,
            phase_coherence:            1.0,
            proteasome_night_boost:     0.25,
            circadian_sasp_contribution: 0.0,
        }
    }
}

impl CircadianState {
    /// Обновить циркадное состояние на основе текущих повреждений центриоли.
    pub fn update(&mut self, dam: &CentriolarDamageState) {
        // Амплитуда: определяется целостностью CEP164 (ресничка) и уровнем агрегатов
        // (агрегаты нарушают BMAL1-транслокацию), модулируется ROS (окисление Clock)
        self.amplitude = (
            dam.cep164_integrity * 0.6
            + (1.0 - dam.protein_aggregates) * 0.4
        ) * (1.0 - dam.ros_level * 0.2);
        self.amplitude = self.amplitude.clamp(0.0, 1.0);

        // Когерентность: нарушается ROS → окисление CLOCK/BMAL1
        self.phase_coherence = (1.0 - dam.ros_level * 0.5)
            .clamp(0.1, 1.0);

        // Ночной буст протеасомы: пропорционален амплитуде
        self.proteasome_night_boost = self.amplitude * 0.25;

        // Циркадный вклад в SASP (конститутивная NF-κB при десинхронизации)
        self.circadian_sasp_contribution = (1.0 - self.amplitude) * 0.3;
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Аутофагия / mTOR (P19)
// ──────────────────────────────────────────────────────────────────────────────

/// Состояние пути аутофагии и mTOR-регуляции.
///
/// mTOR — главный ингибитор аутофагии. С возрастом mTOR-активность растёт
/// (nutrient sensing desensitization), аутофагический поток падает → агрегаты↑.
///
/// Связь с CDATA:
/// - Аутофагия чистит агрегаты независимо от агрегасомного пути (дополнительный клиренс)
/// - CR (Caloric Restriction) → mTOR↓ → аутофагия↑ → CDATA↓ (механизм интервенции P11)
/// - NadPlus → SIRT1↑ → AMPK↑ → mTOR↓ → аутофагия↑
/// - Mitophagy (MitochondrialModule) — частный случай: аутофагия специфически митохондрий
///
/// Биологические ссылки:
/// - Rubinsztein et al. (2011): Autophagy and ageing — Nature Rev Mol Cell Biol
/// - Harrison et al. (2009): Rapamycin-extended lifespan — Nature
/// - Madeo et al. (2015): Spermidine/AMPK/autophagy — Science
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutophagyState {
    /// Активность mTOR комплекса 1 [0..1].
    /// 0.0 = полное подавление (CR/рапамицин); 1.0 = максимальная активность.
    /// `mtor_activity_base = 0.3 + age/100 × 0.5`; модулируется питанием/интервенциями.
    pub mtor_activity: f32,

    /// Аутофагический поток [0..1].
    /// Обратно пропорционален mTOR: `autophagy_flux = 1 − mtor_activity × 0.8`
    pub autophagy_flux: f32,

    /// Вклад аутофагии в клиренс агрегатов [0..1].
    /// = `autophagy_flux × 0.10` — максимально 10% агрегатов/год при полной аутофагии.
    pub aggregate_autophagy_clearance: f32,

    /// Связь с митофагией: аутофагия усиливает базовую митофагию [0..0.3].
    /// = `autophagy_flux × 0.3`; суммируется с `MitochondrialState::mitophagy_flux`.
    pub mitophagy_coupling: f32,
}

impl Default for AutophagyState {
    fn default() -> Self {
        Self {
            mtor_activity:               0.3,   // молодой организм: умеренный mTOR
            autophagy_flux:              0.76,  // = 1 − 0.3×0.8
            aggregate_autophagy_clearance: 0.076,
            mitophagy_coupling:          0.228,
        }
    }
}

impl AutophagyState {
    /// Обновить аутофагическое состояние.
    ///
    /// # Аргументы
    /// * `age_years`         — хронологический возраст (лет)
    /// * `cr_active`         — флаг Caloric Restriction интервенции
    /// * `nad_plus_active`   — флаг NadPlus интервенции
    pub fn update(&mut self, age_years: f32, cr_active: bool, nad_plus_active: f32) {
        // Базовая mTOR: растёт с возрастом (десенсибилизация нутриент-сенсинга)
        let age_mtor = (0.3 + age_years / 100.0 * 0.5).clamp(0.3, 0.8);

        // Интервенции снижают mTOR:
        // CR: ~30% снижение mTOR (AMPK↑)
        // NadPlus: ~20% снижение (SIRT1→AMPK)
        let cr_reduction    = if cr_active { 0.30 } else { 0.0 };
        let nadp_reduction  = nad_plus_active * 0.20;

        self.mtor_activity = (age_mtor - cr_reduction - nadp_reduction).clamp(0.05, 1.0);

        // Аутофагический поток: обратно пропорционален mTOR
        self.autophagy_flux = (1.0 - self.mtor_activity * 0.8).clamp(0.0, 1.0);

        // Клиренс агрегатов через аутофагию (независим от протеасомного пути)
        self.aggregate_autophagy_clearance = self.autophagy_flux * 0.10;

        // Буст митофагии (суммируется с базовым потоком MitochondrialModule)
        self.mitophagy_coupling = self.autophagy_flux * 0.30;
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Протеостаз (P16) — белковый гомеостаз и клиренс агрегатов
// ──────────────────────────────────────────────────────────────────────────────

/// Состояние системы протеостаза (белкового гомеостаза).
///
/// Центросома = организующий центр агрегасом (Johnston et al. 1998).
/// Повреждение CEP164/spindle → нарушение агрегасом → агрегаты накапливаются быстрее.
/// HSP70/90 оксидируются ROS → потеря шапероновой ёмкости.
/// Протеасома перегружается агрегатами → активность падает.
///
/// Этот компонент модифицирует скорость накопления `protein_aggregates` в `CentriolarDamageState`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProteostasisState {
    /// Ёмкость протеасомы UPS [0..1].
    /// Снижается при перегрузке белковыми агрегатами: `= max(0, 1 - aggregates × 0.8)`.
    pub proteasome_activity: f32,

    /// Шапероновая ёмкость (HSP70/90/110) [0..1].
    /// Окисляется ROS: `= max(0.1, 1 - ros × 0.6)`.
    pub hsp_capacity: f32,

    /// Индекс формирования агрегасом [0..1].
    /// Зависит от целостности центросомы: `= cep164 × 0.6 + spindle_fidelity × 0.4`.
    /// Высокий индекс = центросома активно маршрутизирует мисфолдированные белки к лизосомам.
    pub aggresome_index: f32,

    /// Нагрузка несложенных белков (UPR-стресс) [0..1].
    /// `= aggregates × (1 - proteasome_activity × 0.7 - hsp_capacity × 0.3)`
    pub unfolded_protein_load: f32,

    /// Эффективный клиренс агрегатов [0..1] — суммарный защитный эффект.
    /// Применяется как `aggregate_clearance_rate = aggresome_index × hsp_capacity`.
    pub aggregate_clearance_rate: f32,
}

impl Default for ProteostasisState {
    fn default() -> Self {
        Self {
            proteasome_activity:     1.0,
            hsp_capacity:            1.0,
            aggresome_index:         1.0,
            unfolded_protein_load:   0.0,
            aggregate_clearance_rate: 1.0,
        }
    }
}

impl ProteostasisState {
    /// Обновить состояние протеостаза на основе текущих повреждений.
    ///
    /// Вызывается в `HumanDevelopmentModule::step()` ПОСЛЕ `accumulate_damage()`.
    pub fn update(&mut self, dam: &CentriolarDamageState) {
        // Протеасома: перегружается при высоких агрегатах
        self.proteasome_activity = (1.0 - dam.protein_aggregates * 0.8).max(0.1);

        // Шапероны: оксидируются ROS
        self.hsp_capacity = (1.0 - dam.ros_level * 0.6).max(0.1);

        // Агрегасомный индекс: зависит от CEP164 (переходная зона) и spindle_fidelity (MTOC)
        self.aggresome_index = dam.cep164_integrity * 0.6
            + dam.spindle_fidelity * 0.4;

        // UPR-нагрузка: агрегаты, которые не удаётся разобрать
        let clearance_capacity = self.proteasome_activity * 0.7 + self.hsp_capacity * 0.3;
        self.unfolded_protein_load =
            (dam.protein_aggregates * (1.0 - clearance_capacity)).clamp(0.0, 1.0);

        // Суммарный клиренс: агрегасомы + шапероны работают совместно
        self.aggregate_clearance_rate =
            (self.aggresome_index * self.hsp_capacity).clamp(0.0, 1.0);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// NK-клеточный иммунный надзор (P15)
// ──────────────────────────────────────────────────────────────────────────────

/// Состояние NK-клеточного иммунного надзора стволовой ниши.
///
/// NK-клетки распознают повреждённые стволовые клетки через NKG2D-лиганды
/// (MICA/MICB/ULBP1-3), экспрессия которых индуцируется при клеточном стрессе
/// (ATM/ATR → NF-κB → лиганды), и элиминируют их.
///
/// С возрастом активность NK-клеток снижается (иммуносенесценция): меньше
/// функциональных NK-клеток → дефектные HSC выживают → CDATA ускоряется.
///
/// Обратная связь с myeloid_shift: миелоидный сдвиг подавляет NK-функцию
/// через ИЛ-13/ИЛ-4 (Th2-поляризация) и TGF-β (Трeg-экспансия).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NKSurveillanceState {
    /// Поверхностная экспрессия NKG2D-лигандов [0..1].
    /// Растёт пропорционально ROS и белковым агрегатам (стресс-ответ).
    pub nkg2d_ligand_expression: f32,

    /// Активность NK-клеток в нише [0..1].
    /// 1.0 — молодой здоровый организм; снижается с возрастом и при миелоидном сдвиге.
    pub nk_activity: f32,

    /// Вероятность NK-элиминации клетки за один шаг [0..1].
    /// `kill_prob = nk_activity × nkg2d_expression × (1 − immune_escape_fraction)`
    pub nk_kill_probability: f32,

    /// Фракция клеток с иммунным ускользанием [0..1].
    /// Возникает из-за снижения MHC-I (иммунное ускользание при высоких повреждениях).
    /// При protein_carbonylation > 0.6: MHC-I нарушается → частичное ускользание.
    pub immune_escape_fraction: f32,

    /// Общее число NK-элиминаций, пережитых нишей (для мониторинга).
    pub total_eliminations: u32,
}

impl Default for NKSurveillanceState {
    fn default() -> Self {
        Self {
            nkg2d_ligand_expression: 0.0,
            nk_activity:             1.0,
            nk_kill_probability:     0.0,
            immune_escape_fraction:  0.0,
            total_eliminations:      0,
        }
    }
}

impl NKSurveillanceState {
    /// Обновить NK-состояние на основе текущих повреждений и возраста.
    ///
    /// # Аргументы
    /// * `ros`           — `CentriolarDamageState::ros_level`
    /// * `aggregates`    — `CentriolarDamageState::protein_aggregates`
    /// * `carbonylation` — `CentriolarDamageState::protein_carbonylation`
    /// * `age_years`     — хронологический возраст (лет)
    /// * `myeloid_bias`  — из `MyeloidShiftComponent::myeloid_bias` (опционально, 0 если нет)
    pub fn update(
        &mut self,
        ros: f32,
        aggregates: f32,
        carbonylation: f32,
        age_years: f32,
        myeloid_bias: f32,
    ) {
        // NKG2D-лиганды индуцируются при стрессе (ROS + агрегаты).
        // ВАЖНО: нормальные клетки не экспрессируют NKG2D-лиганды — только клетки
        // под серьёзным стрессом (>30% от максимума). Поэтому вычитаем базовый уровень 0.30.
        // Биологически: MICA/MICB/ULBP1-3 индуцируются только при активации ATM/ATR/NF-κB,
        // что требует значимого повреждения ДНК или белкового стресса (Raulet et al. 2013).
        let raw_ligand = ros * 0.6 + aggregates * 0.4;
        self.nkg2d_ligand_expression = (raw_ligand - 0.30).max(0.0).clamp(0.0, 1.0);

        // NK-активность: 1.0 в молодости, линейно снижается после 40 лет до 0.3 к 90 годам.
        // Дополнительно подавляется миелоидным сдвигом (TGF-β/ИЛ-13).
        let age_decline = if age_years > 40.0 {
            ((age_years - 40.0) / 50.0 * 0.7).clamp(0.0, 0.7)
        } else {
            0.0
        };
        let myeloid_suppression = myeloid_bias * 0.25;
        self.nk_activity = (1.0 - age_decline - myeloid_suppression).clamp(0.1, 1.0);

        // Иммунное ускользание: при высоком карбонилировании нарушается MHC-I
        self.immune_escape_fraction = (carbonylation * 0.5).clamp(0.0, 0.5);

        // Итоговая вероятность элиминации за один шаг.
        // Ненулевая только если лиганды выше базового уровня.
        self.nk_kill_probability = (
            self.nk_activity
            * self.nkg2d_ligand_expression
            * (1.0 - self.immune_escape_fraction)
        ).clamp(0.0, 1.0);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Ответ на повреждение ДНК (P20) — DDR / ATM / p53
// ──────────────────────────────────────────────────────────────────────────────

/// Состояние пути ответа на повреждение ДНК (DDR).
///
/// Spindle fidelity↓ → хромосомная нестабильность → анеуплоидия → DSB (двунитевые разрывы) →
/// ATM-киназа → p53-стабилизация → p21-транскрипция → G1-арест.
///
/// Этот компонент закрывает петлю CDATA → клеточный цикл:
/// - `p53_stabilization × 0.3` пишется в `GeneExpressionState.p21_level`
/// - `cell_cycle_module` читает `p21_level` и применяет G1SRestriction
///
/// Связь с другими компонентами:
/// - ROS (от `CentriolarDamageState`) вносит прямой вклад в γH2AX (окислительные разрывы)
/// - Агрегаты снижают `dna_repair_capacity` (протеасомная нагрузка мешает NHEJ/HR)
/// - Возраст снижает `dna_repair_capacity` (уменьшение MRN-комплекса, Ku70/Ku80)
///
/// Биологические ссылки:
/// - Jackson & Bartek (2009): DNA-damage response in human biology — Nature
/// - Rodier et al. (2009): Persistent DNA damage signalling — Nature Cell Biol
/// - Bakkenist & Kastan (2003): DNA damage activates ATM — Nature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDRState {
    /// Уровень γH2AX (маркер двунитевых разрывов ДНК) [0..1].
    /// `= (1 − spindle_fidelity)^1.5 × 0.8 + ros_level × 0.2`
    /// Spindle↓ → анеуплоидия → DSB; ROS→ окислительные разрывы (8-OHdG).
    pub gamma_h2ax_level: f32,

    /// Активность ATM-киназы [0..1].
    /// `= gamma_h2ax_level × dna_repair_capacity`
    /// ATM — главный сенсор DSB; активность ограничена ёмкостью репарации.
    pub atm_activity: f32,

    /// Стабилизация p53 [0..1].
    /// `= atm_activity × 0.7`
    /// ATM фосфорилирует MDM2, что стабилизирует p53 (снижает деградацию).
    /// Высокий p53 → транскрипция p21 → G1-арест (антипролиферативный барьер).
    pub p53_stabilization: f32,

    /// Ёмкость репарации ДНК (NHEJ + HR) [0..1].
    /// `= max(0.3, 1 − age_years/150 − aggregates × 0.2)`
    /// Снижается с возрастом (уменьшение Ku70/Ku80, Rad51) и при перегрузке протеасомы.
    /// Пол: 0.3 — минимальный уровень (репарация никогда полностью не отключается).
    pub dna_repair_capacity: f32,
}

impl Default for DDRState {
    fn default() -> Self {
        Self {
            gamma_h2ax_level:    0.0,
            atm_activity:        0.0,
            p53_stabilization:   0.0,
            dna_repair_capacity: 1.0,
        }
    }
}

impl DDRState {
    /// Обновить DDR-состояние на основе текущих повреждений и возраста.
    ///
    /// # Аргументы
    /// * `spindle_fidelity` — `CentriolarDamageState::spindle_fidelity` [0..1]
    /// * `ros_level`        — `CentriolarDamageState::ros_level` [0..1]
    /// * `aggregates`       — `CentriolarDamageState::protein_aggregates` [0..1]
    /// * `age_years`        — хронологический возраст (лет)
    pub fn update(
        &mut self,
        spindle_fidelity: f32,
        ros_level: f32,
        aggregates: f32,
        age_years: f32,
    ) {
        // ёмкость репарации: снижается с возрастом и перегрузкой агрегатами
        self.dna_repair_capacity =
            (1.0 - age_years / 150.0 - aggregates * 0.2).max(0.3);

        // γH2AX: spindle↓ → хромосомная нестабильность → DSB; ROS → окислительные разрывы
        let spindle_damage = (1.0 - spindle_fidelity).powf(1.5);
        self.gamma_h2ax_level = (spindle_damage * 0.8 + ros_level * 0.2).clamp(0.0, 1.0);

        // ATM: активируется DSB, ограничен ёмкостью репарации
        self.atm_activity = (self.gamma_h2ax_level * self.dna_repair_capacity).clamp(0.0, 1.0);

        // p53: стабилизируется ATM (фосфорилирование MDM2 → деградация MDM2↓)
        self.p53_stabilization = (self.atm_activity * 0.7).clamp(0.0, 1.0);
    }

    /// Вычислить вклад p53 в p21 для передачи в `GeneExpressionState`.
    ///
    /// Возвращает значение, которое нужно **добавить** к `GeneExpressionState.p21_level`:
    /// `p53_stabilization × 0.3` (максимальный вклад DDR в p21 = 0.3).
    #[inline]
    pub fn p21_contribution(&self) -> f32 {
        self.p53_stabilization * 0.3
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Трек F — Снижение темпа деления стволовых клеток со временем
// ──────────────────────────────────────────────────────────────────────────────

/// Состояние темпа деления стволовых клеток (Track F).
///
/// С возрастом стволовые клетки делятся всё реже — даже если ниша ещё не истощена
/// (Track B). Это самостоятельный механизм старения: замедление обновления тканей
/// при сохранном пуле. Ткань деградирует медленнее, чем при Track B, но неизбежно.
///
/// ## Молекулярные причины снижения темпа (Tkemaladze, 2024)
/// 1. **CEP164↓ → цилии↓ → Wnt/Shh↓ → Cyclin D1↓** — морфогенная стимуляция G1→S
///    снижается при потере первичных ресничек.
/// 2. **Spindle damage → p21↑ → G1-арест удлиняется** — клетки проводят больше времени
///    в G1, сокращая частоту делений в единицу времени.
/// 3. **ROS-индуцированное старение** — высокий ROS стабилизирует p16/p21, создавая
///    квазисенесцентные состояния без полного арреста.
/// 4. **mTOR-зависимое замедление** — с возрастом mTOR-активность растёт, подавляя
///    автофагию и перераспределяя ресурсы от пролиферации к репарации.
/// 5. **Накопление PTM на центриолях** — затруднение разборки PCM (перицентросомного
///    материала) перед делением → удлинение G2/M.
///
/// ## Связь с другими треками
/// - Усиливает Track B: медленное + уменьшающееся пул = двойной удар.
/// - Взаимодействует с Track A: потеря морфогенного сигнала снижает пролиферативный стимул.
/// - Связан с P20 (DDR): ATM → p53 → p21 → удлинение G1 → снижение division_rate.
/// - Взаимодействует с P19 (mTOR): высокий mTOR подавляет пролиферацию стволовых клеток.
///
/// ## Биологические ссылки
/// - Tkemaladze (2024): The rate of stem cell division decreases with age
/// - Janzen et al. (2006): Stem cell ageing is regulated by p21 — Aging Cell
/// - Beerman & Rossi (2015): Epigenetic regulation of HSC aging — Exp Cell Res
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemCellDivisionRateState {
    /// Нормализованный темп деления [0..1].
    /// 1.0 = молодая клетка; снижается со временем по совокупности факторов.
    /// Применяется как мультипликатор к `TissueState::regeneration_tempo`.
    pub division_rate: f32,

    /// Вклад цилиарного/морфогенного сигнала в темп деления [0..1].
    /// `cilia_drive = 0.25 + ciliary_function × 0.75`
    /// При ciliary_function=0: минимальный сигнал 0.25 (паракринные пути).
    pub cilia_drive: f32,

    /// Вклад качества веретена (через длительность G1) [0..1].
    /// `spindle_drive = 0.3 + spindle_fidelity × 0.7`
    /// При spindle=0: фракция клеток в длительном G1-аресте максимальна.
    pub spindle_drive: f32,

    /// Возрастной коэффициент торможения [0..1].
    /// `age_factor = max(0.15, 1.0 - (age - 20).max(0) / 100)`
    /// Линейное снижение с 20 лет; достигает 0.15 примерно к 105 годам.
    pub age_factor: f32,

    /// Вклад ROS-индуцированного квазисенесцентного торможения [0..1].
    /// `ros_brake = 1.0 - ros_level × 0.4`  (ROS подавляет пролиферацию)
    pub ros_brake: f32,

    /// Вклад mTOR-зависимого торможения [0..1].
    /// `mtor_brake = 1.0 - (mtor_activity - 0.3) × 0.35`
    /// Базовый mTOR=0.3 → нет торможения; высокий mTOR → снижение темпа.
    pub mtor_brake: f32,

    /// Интегральный индекс снижения темпа деления по сравнению с молодой нормой [0..1].
    /// `decline_index = 1.0 - division_rate`
    /// 0 = нет снижения; 1 = деления полностью прекратились.
    pub decline_index: f32,

    // --- Калибровочные коэффициенты (configurable via set_module_params) ---
    /// Нижняя граница age_factor (минимальный темп деления у очень пожилых) [0..1].
    /// По умолчанию: 0.15. Управляет ползунком `division_rate_floor` в GUI.
    pub age_factor_floor: f32,

    /// Коэффициент торможения ROS: `ros_brake = 1.0 - ros_level × ros_brake_strength`.
    /// По умолчанию: 0.40 (калибровано по Passos et al. 2010; SA: Δdiv ≈ ±18% при ±50%).
    pub ros_brake_strength: f32,

    /// Коэффициент торможения mTOR: `mtor_brake = 1.0 - (mtor-0.3).max(0) × mtor_brake_strength`.
    /// По умолчанию: 0.35 (Harrison et al. 2009 — rapamycin +28% lifespan → ~0.35 эффект).
    pub mtor_brake_strength: f32,
}

impl Default for StemCellDivisionRateState {
    fn default() -> Self {
        Self {
            division_rate:     1.0,
            cilia_drive:       1.0,
            spindle_drive:     1.0,
            age_factor:        1.0,
            ros_brake:         1.0,
            mtor_brake:        1.0,
            decline_index:     0.0,
            age_factor_floor:  0.15,
            ros_brake_strength: 0.40,
            mtor_brake_strength: 0.35,
        }
    }
}

impl StemCellDivisionRateState {
    /// Обновить темп деления на основе текущего состояния повреждений.
    ///
    /// # Аргументы
    /// * `ciliary_function` — `CentriolarDamageState::ciliary_function` [0..1]
    /// * `spindle_fidelity` — `CentriolarDamageState::spindle_fidelity` [0..1]
    /// * `ros_level`        — `CentriolarDamageState::ros_level` [0..1]
    /// * `age_years`        — хронологический возраст (лет)
    /// * `mtor_activity`    — из `AutophagyState::mtor_activity` [0..1]; 0.3 если недоступен
    pub fn update(
        &mut self,
        ciliary_function: f32,
        spindle_fidelity: f32,
        ros_level:        f32,
        age_years:        f32,
        mtor_activity:    f32,
    ) {
        // 1. Цилиарный/морфогенный вклад: Wnt/Shh стимулируют G1→S через Cyclin D1
        self.cilia_drive = (0.25 + ciliary_function * 0.75).clamp(0.0, 1.0);

        // 2. Качество веретена: повреждение → более длительный G1, арест (p21↑)
        self.spindle_drive = (0.30 + spindle_fidelity * 0.70).clamp(0.0, 1.0);

        // 3. Возрастной фактор: линейное снижение с 20 лет; нижняя граница = age_factor_floor
        self.age_factor = (1.0 - (age_years - 20.0).max(0.0) / 100.0)
            .clamp(self.age_factor_floor, 1.0);

        // 4. ROS-тормоз: высокий ROS → p21/p16 стабилизация → квазисенесценция
        // Коэффициент ros_brake_strength задаётся через set_module_params (GUI ползунок).
        // Нижний предел clamp = (1 - strength): при ros=1.0 тормоз не сильнее коэффициента.
        let ros_floor = (1.0 - self.ros_brake_strength).max(0.0);
        self.ros_brake = (1.0 - ros_level * self.ros_brake_strength).clamp(ros_floor, 1.0);

        // 5. mTOR-тормоз: повышенный mTOR перераспределяет ресурсы от пролиферации
        // Базовый mTOR=0.3 → нет торможения; mTOR=0.8 → -mtor_brake_strength×0.5 к темпу
        // Коэффициент mtor_brake_strength задаётся через set_module_params (GUI ползунок)
        self.mtor_brake = (1.0 - (mtor_activity - 0.3).max(0.0) * self.mtor_brake_strength).clamp(0.6, 1.0);

        // Интегральный темп: произведение всех компонентов
        self.division_rate = (self.cilia_drive
            * self.spindle_drive
            * self.age_factor
            * self.ros_brake
            * self.mtor_brake)
            .clamp(0.01, 1.0);

        self.decline_index = 1.0 - self.division_rate;
    }
}

impl CellCycleStateExtended {
    /// Получить активность конкретного комплекса
    pub fn get_complex_activity(&self, cyclin_type: CyclinType, cdk_type: CdkType) -> f32 {
        for complex in &self.cyclin_cdk_complexes {
            if complex.cyclin_type == cyclin_type && complex.cdk_type == cdk_type {
                return complex.activity;
            }
        }
        0.0
    }
    
    /// Учет влияния центриоли (заглушка)
    pub fn apply_centriole_influence(&mut self, _centriole: &CentriolePair) {
        // Будет реализовано позже
    }
    
    /// Обновление циклинов (заглушка)
    pub fn update_cyclins(&mut self, _dt: f32) {
        // Будет реализовано позже
    }
}

// ---------------------------------------------------------------------------
// Необратимая дифференцировка и обратимая модуляция (CDATA)
// ---------------------------------------------------------------------------

/// Необратимый уровень дифференцировки клетки.
///
/// Определяется отщеплением индукторов дифференцировки от центриолей.
/// Каждый переход запускается **внутренним** фактором — потерей индуктора —
/// а не внешними сигналами. После фиксации уровень не может регрессировать.
///
/// Соответствует [`PotencyLevel`] молекулярному состоянию:
/// `Totipotent → Pluripotent → Multipotent → Committed → Terminal`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DifferentiationTier {
    /// Зигота: все индукторы интактны, оба комплекта полны.
    Totipotent,
    /// Плюрипотент: оба комплекта имеют оставшиеся индукторы, но уже потеряли часть.
    Pluripotent,
    /// Мультипотент (олигопотент): один комплект исчерпан.
    Multipotent,
    /// Коммитированная: последние индукторы почти исчерпаны (унипотент).
    Committed,
    /// Терминально дифференцированная: оба комплекта пусты, деления невозможны.
    Terminal,
}

impl DifferentiationTier {
    /// Производить `DifferentiationTier` из текущего потентностного состояния.
    pub fn from_potency(potency: PotencyLevel) -> Self {
        match potency {
            PotencyLevel::Totipotent  => DifferentiationTier::Totipotent,
            PotencyLevel::Pluripotent => DifferentiationTier::Pluripotent,
            PotencyLevel::Oligopotent => DifferentiationTier::Multipotent,
            PotencyLevel::Unipotent   => DifferentiationTier::Committed,
            PotencyLevel::Apoptosis   => DifferentiationTier::Terminal,
        }
    }
}

/// ECS-компонент необратимого статуса дифференцировки (CDATA).
///
/// Устанавливается однажды при первом отщеплении индуктора и может продвигаться
/// **только вперёд** по лестнице дифференцировки.
/// Отражает биологическую концепцию CDATA: при каждом отщеплении индуктора
/// он внедряется в ядерную ДНК → включаются генные сети нового статуса,
/// выключаются предыдущие. Этот процесс необратим.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentiationStatus {
    /// Текущий необратимый уровень (только вперёд).
    pub tier: DifferentiationTier,
    /// История переходов: `(новый_уровень, возраст_в_годах)`.
    pub tier_history: Vec<(DifferentiationTier, f64)>,
    /// Количество необратимых переходов (событий коммитирования).
    pub commitment_events: u32,
    /// Активны ли индукторы дифференцировки (создаются de novo при n-м делении).
    /// `false` до достижения стадии de novo — клетка не может коммитироваться.
    pub inductors_active: bool,
    /// Произошла ли элиминация центриолей в прелептотенной стадии мейоза.
    /// `true` — элиминация зарегистрирована (для текущего поколения).
    pub meiotic_reset_done: bool,
}

impl DifferentiationStatus {
    pub fn new(initial_potency: PotencyLevel) -> Self {
        Self {
            tier: DifferentiationTier::from_potency(initial_potency),
            tier_history: Vec::new(),
            commitment_events: 0,
            inductors_active: false,
            meiotic_reset_done: false,
        }
    }

    /// Продвинуть tier вперёд, если `new_potency` даёт более высокий уровень дифференцировки.
    /// Возвращает `true` если произошёл переход (commitment event).
    /// Никогда не позволяет регрессировать.
    pub fn try_advance(&mut self, new_potency: PotencyLevel, age_years: f64) -> bool {
        let new_tier = DifferentiationTier::from_potency(new_potency);
        if new_tier > self.tier {
            self.tier_history.push((new_tier, age_years));
            self.tier = new_tier;
            self.commitment_events += 1;
            true
        } else {
            false
        }
    }

    /// Сброс статуса дифференцировки при элиминации центриолей в прелептотенной стадии мейоза.
    /// Индукторы элиминируются → следующее поколение начнёт с Totipotent.
    /// История сохраняется для аудита; счётчик коммитирований сбрасывается.
    /// `meiotic_reset_done` сбрасывается в `false` — следующее поколение может снова пройти мейоз.
    pub fn reset_for_meiosis(&mut self) {
        self.tier = DifferentiationTier::Totipotent;
        self.commitment_events = 0;
        self.inductors_active = false;
        self.meiotic_reset_done = false; // сброс флага — новое поколение может снова пройти этот этап
    }
}

impl Default for DifferentiationStatus {
    fn default() -> Self { Self::new(PotencyLevel::Totipotent) }
}

/// ECS-компонент обратимой модуляции клетки (CDATA).
///
/// Изменяется под влиянием **внешних** сигналов: нишевых факторов, паракрина,
/// воспаления (InflammagingState), ростовых факторов.
/// Не меняет [`DifferentiationStatus`] — только адаптирует поведение клетки
/// в рамках уже зафиксированного статуса дифференцировки.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulationState {
    /// Уровень активности [0..1]: 0 = покой (G0), 1 = максимальная активность.
    pub activity_level: f32,
    /// Обратимый покой (G0-квесценция): `true` при `activity_level < 0.2`.
    pub is_quiescent: bool,
    /// Сила нишевых сигналов, получаемых клеткой [0..1].
    pub niche_signal_strength: f32,
    /// Ответ на острый стресс [0..1]: шаперонный стресс-ответ (HSP70, HSP90).
    pub stress_response: f32,
    /// SASP-вклад этой клетки в окружающую нишу [0..1].
    /// Ненулевой только у сенесцентных клеток.
    pub sasp_output: f32,
    /// Эпигенетическая пластичность [0..1]: насколько клетка может модулировать
    /// экспрессию в рамках текущего дифференцировочного статуса.
    /// Снижается по мере прохождения уровней дифференцировки.
    pub epigenetic_plasticity: f32,
}

impl Default for ModulationState {
    fn default() -> Self {
        Self {
            activity_level: 1.0,
            is_quiescent: false,
            niche_signal_strength: 1.0,
            stress_response: 0.0,
            sasp_output: 0.0,
            epigenetic_plasticity: 1.0,
        }
    }
}

// ============================================================
// Митохондриальное состояние (Трек E)
// ============================================================

/// ECS-компонент митохондриального здоровья (CDATA Трек E).
///
/// Митохондрии формируют кислородный щит вокруг центросомы.
/// При дисфункции митохондрий (мутации мтДНК, фрагментация, избыток ROS)
/// щит ослабевает → больше O₂ проникает к центриолям → ускоряется
/// отщепление индукторов.
/// Клональное состояние стволовой ниши.
///
/// Каждая основательская ниша получает уникальный `clone_id` при инициализации.
/// При симметричном делении (заполнение пустого слота пула) дочь наследует
/// тот же `clone_id` и инкрементирует `generation`.
///
/// Используется для моделирования клонального гемопоэза (CHIP):
/// клоны с более медленным истощением индукторов постепенно вытесняют
/// стареющие линии — демографический дрейф без отбора по fitness.
///
/// Источник: «Centrioles as determinants» (2026-01-27) + Jaiswal et al. 2014 (CHIP).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClonalState {
    /// Уникальный ID клональной линии (назначается при основании ниши, не меняется).
    pub clone_id: u64,
    /// Номер поколения от основателя (0 = основатель, 1 = первое дочернее, ...).
    pub generation: u32,
    /// Возраст организма (дни) на момент основания данной клональной линии.
    pub founder_age_days: f64,
}

impl ClonalState {
    /// Создать нового основателя.
    pub fn founder(clone_id: u64) -> Self {
        Self { clone_id, generation: 0, founder_age_days: 0.0 }
    }

    /// Создать клональную дочь (тот же clone_id, generation+1).
    pub fn daughter(&self) -> Self {
        Self {
            clone_id: self.clone_id,
            generation: self.generation + 1,
            founder_age_days: self.founder_age_days,
        }
    }
}

/// Маркер для lazy-init HumanDevelopmentModule.
///
/// Добавляется `AsymmetricDivisionModule` при NichePool-спавне новой ниши.
/// `HumanDevelopmentModule` обнаруживает его в начале `step()`, инициализирует
/// `HumanDevelopmentComponent` и удаляет маркер.
/// Это позволяет NichePool-заменам стареть как полноценные ниши.
#[derive(Debug, Clone, Copy, Default)]
pub struct NeedsHumanDevInit;

///
/// # Петли обратной связи
/// 1. `mtdna_mutations ↑` → `ros_production ↑` → `mtdna_mutations ↑` (цикл)
/// 2. `ros_production ↑` → `fusion_index ↓` (фрагментация) → митофагия менее эффективна
/// 3. `ros_production ↑` → `ros_boost` → `CentriolarDamageState.ros_level ↑`
///    (через `human_development_module`, лаг 1 шаг)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitochondrialState {
    /// Накопление мутаций мтДНК [0..1].
    /// 0 = здоровый геном; 1 = критическая мутационная нагрузка.
    pub mtdna_mutations: f32,
    /// Индекс слияния митохондрий [0..1].
    /// 1.0 = нитевидная сеть (молодые, здоровые);
    /// 0.0 = полная фрагментация (стареющие).
    pub fusion_index: f32,
    /// Продукция ROS митохондриями [0..1].
    /// 0.0 = нормальный уровень; 1.0 = критическая суперпродукция.
    pub ros_production: f32,
    /// Мембранный потенциал (ΔΨm) [0..1].
    /// 1.0 = максимальный потенциал (молодые митохондрии);
    /// снижается при дисфункции → митофагия через PINK1/Parkin теряет эффективность.
    pub membrane_potential: f32,
    /// Поток митофагии [0..1]: скорость очистки дисфункциональных митохондрий.
    /// Снижается при низком `membrane_potential`.
    pub mitophagy_flux: f32,
    /// Вклад митохондрий в кислородный щит центросомы [0..1].
    /// 1.0 = полный щит; 0.0 = щита нет.
    pub mito_shield_contribution: f32,
    /// Плотность перинуклеарного митохондриального кластера [0..1] (P9).
    /// Зависит от `fusion_index` и локального ROS; добавляет пространственный
    /// барьер диффузии O₂ к центросоме поверх скалярного `mito_shield`.
    /// `perinuclear_barrier = perinuclear_density × 0.15`
    pub perinuclear_density: f32,
}

impl Default for MitochondrialState {
    fn default() -> Self {
        Self {
            perinuclear_density: 1.0,
            mtdna_mutations: 0.0,
            fusion_index: 1.0,
            ros_production: 0.0,
            membrane_potential: 1.0,
            mitophagy_flux: 1.0,
            mito_shield_contribution: 1.0,
        }
    }
}

impl MitochondrialState {
    /// Создать молодое митохондриальное состояние (синоним `default()`).
    pub fn pristine() -> Self { Self::default() }

    /// Вычислить вклад в ROS-буст центриолярных повреждений.
    /// Масштаб задаётся параметром `ros_production_boost`.
    pub fn ros_boost(&self, ros_production_boost: f32) -> f32 {
        self.ros_production * ros_production_boost
    }
}


// ---------------------------------------------------------------------------
// Трек G: Life-History Trade-off / Гормональный Часовой Механизм
// ---------------------------------------------------------------------------

/// Фаза репродуктивного цикла (Трек G)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReproductivePhase {
    /// Препубертатный период: половые гормоны не вырабатываются
    Prepubertal,
    /// Репродуктивный период: нарастание и плато гормонального фона
    Fertile,
    /// Перименопауза/Перименарш: постепенное снижение гормонального фона
    Perimenopausal,
    /// Постменопауза / Андропауза: полная потеря гормонального фона
    Postmenopausal,
}

/// Состояние гормонального фона (Трек G: Life-History Trade-off).
///
/// Моделирует гормональную ось HPG (гипоталамус–гипофиз–гонады):
/// - Начало половой зрелости активирует «часовой механизм» старения
///   (Life-History Trade-off: r=0.78 с продолжительностью жизни, R²=0.92 на библейских генеалогиях)
/// - Эстрогены/тестостерон защищают центриоли: снижают ROS до 20%, укрепляют антиоксидантную защиту
/// - Менопауза/андропауза → потеря гормональной защиты → ускорение накопления центриолярных повреждений
///
/// Источник: Tkemaladze J. "Theory of Lifespan Decline" (2026)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HormonalFertilityState {
    /// Текущая фаза репродуктивного цикла
    pub phase: ReproductivePhase,
    /// Уровень половых гормонов [0..1]
    /// 0.0 = препубертат или менопауза; 1.0 = пик фертильности (~25 лет)
    pub hormone_level: f32,
    /// Гормональная защита центриолей [0..1]
    /// Снижает ros_level: effective_ros = ros - (hormonal_protection × 0.20)
    pub hormonal_protection: f32,
    /// Мультипликатор Life-History Trade-off [1.0..1.20]
    /// До пубертата: 1.0; после наступления половозрелости: нарастает до 1.20
    /// (репродуктивные инвестиции снижают ресурсы репарации центриолей)
    pub life_history_factor: f32,
    /// Возраст наступления половозрелости (лет).
    /// Положительно коррелирует с продолжительностью жизни (r=0.78).
    pub puberty_age_years: f32,
    /// Возраст менопаузы / андропаузы (лет)
    pub menopause_age_years: f32,
}

impl Default for HormonalFertilityState {
    fn default() -> Self {
        Self {
            phase: ReproductivePhase::Prepubertal,
            hormone_level: 0.0,
            hormonal_protection: 0.0,
            life_history_factor: 1.0,
            puberty_age_years: 14.0,    // женский вариант по умолчанию
            menopause_age_years: 51.0,
        }
    }
}

impl HormonalFertilityState {
    /// Мужской вариант (андропауза ~70 лет)
    pub fn male() -> Self {
        Self {
            puberty_age_years: 15.0,
            menopause_age_years: 70.0,
            ..Self::default()
        }
    }

    /// Снижение ROS за счёт гормональной защиты (вычитается из ros_level)
    pub fn ros_reduction(&self) -> f32 {
        self.hormonal_protection * 0.20
    }
}

#[cfg(test)]
mod asymmetry_tests {
    use super::*;

    #[test]
    fn test_blood_is_mother_to_stem() {
        assert_eq!(TissueType::Blood.asymmetry_rule(), CentrioleAsymmetryRule::MotherToStem);
    }

    #[test]
    fn test_neural_is_mother_to_stem() {
        assert_eq!(TissueType::Neural.asymmetry_rule(), CentrioleAsymmetryRule::MotherToStem);
    }

    #[test]
    fn test_epithelial_is_symmetric() {
        assert_eq!(TissueType::Epithelial.asymmetry_rule(), CentrioleAsymmetryRule::Symmetric);
    }

    #[test]
    fn test_mother_bias_blood_higher_than_skin() {
        // HSC-ниша: старая мать → LT-HSC → больший mother_bias
        let blood_bias = TissueType::Blood.default_mother_bias();
        let skin_bias  = TissueType::Skin.default_mother_bias();
        assert!(blood_bias > skin_bias,
            "Blood mother_bias={} должен быть > Skin mother_bias={}", blood_bias, skin_bias);
    }

    #[test]
    fn test_all_biases_in_valid_range() {
        let tissues = [
            TissueType::Neural, TissueType::Blood, TissueType::Epithelial,
            TissueType::Muscle, TissueType::Skin,  TissueType::Germline,
            TissueType::Connective, TissueType::Bone, TissueType::Cartilage,
            TissueType::Adipose, TissueType::Liver, TissueType::Kidney,
            TissueType::Heart, TissueType::Lung,
        ];
        for t in tissues {
            let b = t.default_mother_bias();
            assert!(b >= 0.0 && b <= 1.0,
                "{:?}: mother_bias={} вне диапазона [0,1]", t, b);
        }
    }
}

// ---------------------------------------------------------------------------
// Тесты морфогенных механизмов (P13)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod morphogen_tests {
    use super::*;

    /// GLI-активация: Hill-нелинейность (n=2, K=0.5)
    #[test]
    fn test_gli_activation_hill_nonlinearity() {
        let mut dam = CentriolarDamageState::pristine();

        // При полных ресничках (1.0) → GLI ≈ 0.80 (1.0² / (0.25 + 1.0) = 0.80)
        dam.ciliary_function = 1.0;
        let gli_full = dam.gli_activation();
        assert!((gli_full - 0.80).abs() < 0.01,
            "GLI при cilia=1.0 должен быть ≈0.80, получено {:.4}", gli_full);

        // При EC50 (0.5) → GLI = 0.5
        dam.ciliary_function = 0.5;
        let gli_half = dam.gli_activation();
        assert!((gli_half - 0.5).abs() < 0.01,
            "GLI при cilia=0.5 (EC50) должен быть 0.5, получено {:.4}", gli_half);

        // При нуле → GLI = 0
        dam.ciliary_function = 0.0;
        let gli_zero = dam.gli_activation();
        assert_eq!(gli_zero, 0.0, "GLI при cilia=0 должен быть 0");

        // Нелинейность: при cilia=0.3 ответ < 0.3 (порог работает)
        dam.ciliary_function = 0.3;
        let gli_low = dam.gli_activation();
        assert!(gli_low < 0.3,
            "Hill-нелинейность: GLI({:.1}) < {:.1} при n=2, K=0.5", gli_low, 0.3);
    }

    /// GLI строго монотонно растёт с ciliary_function
    #[test]
    fn test_gli_activation_monotone() {
        let mut dam = CentriolarDamageState::pristine();
        let mut prev = -1.0f32;
        for i in 0..=10 {
            dam.ciliary_function = i as f32 / 10.0;
            let gli = dam.gli_activation();
            assert!(gli >= prev, "GLI должен быть монотонно возрастающим");
            assert!(gli >= 0.0 && gli <= 1.0, "GLI вне [0..1]: {}", gli);
            prev = gli;
        }
    }

    /// Старение (CEP164↓) → GLI резко падает из-за Hill-нелинейности
    #[test]
    fn test_gli_drops_faster_than_linear_with_cilia_loss() {
        let mut dam = CentriolarDamageState::pristine();

        dam.ciliary_function = 0.8;
        let gli_80 = dam.gli_activation();

        dam.ciliary_function = 0.2;
        let gli_20 = dam.gli_activation();

        // Линейный ответ: 0.2/0.8 = 0.25 (25% относительно 80%)
        // Hill-ответ должен быть значительно меньше (нелинейный порог)
        let ratio_gli = gli_20 / gli_80;
        let ratio_linear = 0.2 / 0.8;
        assert!(ratio_gli < ratio_linear,
            "Hill-нелинейность: ratio_gli={:.3} должен быть < ratio_linear={:.3}",
            ratio_gli, ratio_linear);
    }

    /// TissueState: морфогенные поля инициализируются в правильных диапазонах
    #[test]
    fn test_tissue_state_morphogen_fields_default() {
        let ts = TissueState::new(TissueType::Blood);
        assert!(ts.gli_activation >= 0.0 && ts.gli_activation <= 1.0);
        assert!(ts.shh_concentration >= 0.0 && ts.shh_concentration <= 1.0);
        assert!(ts.bmp_balance >= 0.0 && ts.bmp_balance <= 1.0);
        assert!(ts.wnt_activity >= 0.0 && ts.wnt_activity <= 1.0);
        // Молодая ткань: низкий BMP (Noggin преобладает)
        assert!(ts.bmp_balance < 0.5,
            "Молодая ткань должна иметь низкий bmp_balance, получено {}", ts.bmp_balance);
    }

    /// detail_level=1: только GLI, остальные поля не обновляются
    #[test]
    fn test_morphogen_detail_level_1_updates_only_gli() {
        let mut ts = TissueState::new(TissueType::Blood);
        // Установим необычные значения BMP/Wnt/Shh — они не должны меняться при level=1
        ts.bmp_balance = 0.99;
        ts.wnt_activity = 0.01;
        ts.shh_concentration = 0.01;

        ts.update_morphogen_fields(0.5, 1);

        // GLI должен быть обновлён (cilia=0.5 → GLI=0.5)
        assert!((ts.gli_activation - 0.5).abs() < 0.01,
            "GLI должен быть обновлён при detail_level=1");
        // BMP и Wnt — без изменений
        assert!((ts.bmp_balance - 0.99).abs() < 0.01,
            "bmp_balance не должен меняться при detail_level=1");
        assert!((ts.wnt_activity - 0.01).abs() < 0.01,
            "wnt_activity не должна меняться при detail_level=1");
    }

    /// detail_level=3: все поля обновляются
    #[test]
    fn test_morphogen_detail_level_3_updates_all() {
        let mut ts = TissueState::new(TissueType::Blood);
        ts.bmp_balance = 0.99;

        ts.update_morphogen_fields(0.5, 3);

        // При cilia=0.5 BMP-баланс должен снизиться (noggin_activity = 0.35 → bmp ≈ 0.52)
        assert!(ts.bmp_balance < 0.99,
            "bmp_balance должен обновиться при detail_level=3, получено {}", ts.bmp_balance);
        assert!(ts.wnt_activity > 0.0 && ts.wnt_activity <= 1.0,
            "wnt_activity должна быть в [0..1]");
    }

    /// С потерей ресничек BMP_balance растёт (Noggin ↓ → BMP ↑)
    #[test]
    fn test_bmp_increases_with_cilia_loss() {
        let mut ts = TissueState::new(TissueType::Blood);

        ts.update_morphogen_fields(1.0, 2);
        let bmp_healthy = ts.bmp_balance;

        ts.update_morphogen_fields(0.1, 2);
        let bmp_damaged = ts.bmp_balance;

        assert!(bmp_damaged > bmp_healthy,
            "BMP-баланс должен расти при потере ресничек: {:.3} > {:.3}",
            bmp_damaged, bmp_healthy);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OrganType + OrganState — ECS-компонент (Уровень +2: органы)
//
// Моделирует функциональный резерв отдельного органа как агрегатор
// стволовых ниш одного тканевого типа.
//
// Полиорганная недостаточность (≥2 органов в failure) → дополнительный
// критерий смерти организма (альтернатива frailty ≥ 0.95).
//
// Связь органов:
//   Heart: cardiac_output → oxygen_delivery ко всем органам
//   Kidney: filtration → clearance токсинов/SASP из крови
//   Liver: detox → снижение systemic_ros
// ─────────────────────────────────────────────────────────────────────────────

/// Тип органа (11 органов в модели).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrganType {
    Heart,
    Kidney,
    Liver,
    Lung,
    Brain,
    Intestine,
    Skin,
    Bone,
    ImmuneSystem,
    EndocrineSystem,
    Muscle,
}

impl OrganType {
    /// Порог недостаточности (functional_reserve < threshold → орган в failure).
    pub fn failure_threshold(self) -> f32 {
        match self {
            OrganType::Heart         => 0.20,  // сердечная недостаточность — фатальна рано
            OrganType::Brain         => 0.15,  // нейродегенерация — длительная, но фатальна
            OrganType::Kidney        => 0.15,  // ХПН — диализ компенсирует долго
            OrganType::Lung          => 0.20,
            OrganType::Liver         => 0.15,  // большой резерв паренхимы
            OrganType::Intestine     => 0.25,
            OrganType::Skin          => 0.10,  // кожа не фатальна как орган
            OrganType::Bone          => 0.20,
            OrganType::ImmuneSystem  => 0.20,
            OrganType::EndocrineSystem => 0.25,
            OrganType::Muscle        => 0.15,
        }
    }

    /// Компенсаторная ёмкость — насколько другие органы могут взять функцию.
    pub fn compensation_capacity(self) -> f32 {
        match self {
            OrganType::Liver    => 0.60,  // регенерирует, другая доля берёт функцию
            OrganType::Kidney   => 0.40,  // одна почка = 70% функции
            OrganType::Lung     => 0.35,  // лёгочная гипертензия компенсирует частично
            OrganType::Skin     => 0.80,  // слизистые компенсируют
            OrganType::Bone     => 0.50,
            OrganType::Muscle   => 0.40,
            OrganType::ImmuneSystem  => 0.30,
            OrganType::EndocrineSystem => 0.50,
            OrganType::Heart    => 0.10,  // сердце не заменяется
            OrganType::Brain    => 0.05,
            OrganType::Intestine => 0.30,
        }
    }
}

/// Состояние органа (Уровень +2).
///
/// Агрегирует функциональный резерв всех стволовых ниш данного органа.
/// Компенсаторная гипертрофия поддерживает функцию при потере части ниш.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganState {
    /// Тип органа.
    pub organ_type: OrganType,

    /// Функциональный резерв [0..1].
    ///
    /// = mean(functional_capacity ниш) × (1 − fibrosis_penalty)
    /// При резерве ниже failure_threshold → орган в состоянии недостаточности.
    pub functional_reserve: f32,

    /// Компенсаторная ёмкость: доля функции, которую берут соседние органы [0..1].
    ///
    /// Задаётся константой OrganType::compensation_capacity().
    /// При частичной недостаточности: effective_reserve += compensation_factor × deficit.
    pub compensation_capacity: f32,

    /// Порог недостаточности [0..1].
    pub failure_threshold: f32,

    /// Орган в состоянии недостаточности.
    pub is_failing: bool,

    /// Количество ниш, внёсших данные в последнем агрегировании.
    pub niche_count: u32,
}

impl OrganState {
    /// Создать новый OrganState для данного типа органа.
    pub fn new(organ_type: OrganType) -> Self {
        Self {
            organ_type,
            functional_reserve:    1.0,
            compensation_capacity: organ_type.compensation_capacity(),
            failure_threshold:     organ_type.failure_threshold(),
            is_failing:            false,
            niche_count:           0,
        }
    }

    /// Обновить статус недостаточности.
    pub fn update_failure_status(&mut self) {
        self.is_failing = self.functional_reserve < self.failure_threshold;
    }
}

impl Default for OrganState {
    fn default() -> Self { Self::new(OrganType::Muscle) }
}

// ─────────────────────────────────────────────────────────────────────────────
// CloneEpigeneticState — ECS-компонент (Уровень 0: клетка — эпигенетическая память)
//
// Клон-специфический эпигенетический дрейф:
// разные клоны имеют разные базовые линии methylation_age.
// CHIP-клоны с TET2/DNMT3A мутациями: ускоренный дрейф.
//
// Связи:
//   EpigeneticClockState.methylation_age += clone_drift × dt каждый шаг
//   ChromatinState: methylation_age → tad_integrity
// ─────────────────────────────────────────────────────────────────────────────

/// Клон-специфическая эпигенетическая память (Уровень 0: клетка).
///
/// Каждый клон (clone_id) начинает с разной базовой линии methylation_age
/// и имеет уникальный темп дрейфа.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneEpigeneticState {
    /// Базовая линия methylation_age для данного клона [0..1].
    ///
    /// Наследуется при делении + небольшой шум (эпигенетическая гетерогенность).
    /// TET2-мутантные клоны: baseline ≈ 0.15 (гипометилирование).
    pub clone_baseline: f32,

    /// Клон-специфический темп эпигенетического дрейфа [/год].
    ///
    /// Нейтральный клон: drift ≈ 0.003/год.
    /// TET2-мут.: drift ≈ 0.006/год (ускоренный дрейф у CHIP-клонов).
    pub clone_drift_rate: f32,

    /// Накопленный клон-специфический дрейф [0..1].
    ///
    /// Добавляется к EpigeneticClockState.methylation_age:
    ///   effective_age = epi_clock + clone_drift_accumulated
    pub clone_drift_accumulated: f32,
}

impl CloneEpigeneticState {
    /// Нейтральный клон (средний дрейф).
    pub fn neutral() -> Self {
        Self {
            clone_baseline:           0.0,
            clone_drift_rate:         0.003,
            clone_drift_accumulated:  0.0,
        }
    }

    /// TET2-мутантный CHIP-клон (ускоренное деметилирование).
    pub fn tet2_chip() -> Self {
        Self {
            clone_baseline:           0.10,
            clone_drift_rate:         0.006,
            clone_drift_accumulated:  0.0,
        }
    }

    /// Эффективный вклад в methylation_age.
    pub fn effective_methylation_contribution(&self) -> f32 {
        (self.clone_baseline + self.clone_drift_accumulated).clamp(0.0, 1.0)
    }
}

impl Default for CloneEpigeneticState {
    fn default() -> Self { Self::neutral() }
}

// ─────────────────────────────────────────────────────────────────────────────
// FateSwitchingState — ECS-компонент (Уровень 0: клетка — стохастика судьбы)
//
// Стохастическое переключение типа деления с ε-шумом.
// Даже при идентичных damage_params разные ниши могут выбрать разные судьбы.
// Моделирует biological noise в решениях самообновление/дифференцировка.
//
// Связи:
//   noise_scale из DamageParams → пертурбация fate_bias
//   fate_bias > switch_threshold → клетка «склоняется» к симм. делению
//   Интегрируется с asymmetric_division_module через DivisionExhaustionState
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние стохастического переключения судьбы (Уровень 0: клетка).
///
/// Каждая ниша имеет непрерывно флуктуирующий fate_bias.
/// При достижении порога (switch_threshold) — смещение типа деления.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateSwitchingState {
    /// Текущее смещение судьбы [-1..+1].
    ///
    /// > 0 : смещение к самообновлению (симметричному экспансионному делению)
    /// < 0 : смещение к дифференцировке (истощению пула)
    /// 0   : нейтральное состояние
    pub fate_bias: f32,

    /// Порог переключения [0..1].
    ///
    /// При |fate_bias| > switch_threshold — тип деления смещается.
    /// Нормальное значение: 0.5. При повреждениях: порог снижается.
    pub switch_threshold: f32,

    /// Накопленный шум за текущий шаг (Ланжевен).
    ///
    /// Обновляется каждый шаг: noise_accumulator += N(0, σ) × sqrt(dt).
    pub noise_accumulator: f32,

    /// Количество переключений судьбы за всё время жизни ниши.
    pub switch_count: u32,
}

impl FateSwitchingState {
    pub fn neutral() -> Self {
        Self {
            fate_bias:         0.0,
            switch_threshold:  0.5,
            noise_accumulator: 0.0,
            switch_count:      0,
        }
    }

    /// Активно ли переключение (|fate_bias| превышает порог).
    pub fn is_switching(&self) -> bool {
        self.fate_bias.abs() > self.switch_threshold
    }

    /// Смещение к истощению (fate_bias < -threshold).
    pub fn is_exhaustion_biased(&self) -> bool {
        self.fate_bias < -self.switch_threshold
    }
}

impl Default for FateSwitchingState {
    fn default() -> Self { Self::neutral() }
}

// ─────────────────────────────────────────────────────────────────────────────
// GammaRingState — ECS-компонент (Уровень -3: γ-TuRC нуклеация)
//
// γ-тубулиновые кольцевые комплексы (γ-TuRC) нуклеируют микротрубочки
// на центриолях. Ninein-integrity из AppendageProteinState определяет
// плотность γ-TuRC на субдистальных придатках.
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние γ-тубулиновых кольцевых комплексов (γ-TuRC) на центриолях.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaRingState {
    /// Эффективность нуклеации микротрубочек [0..1].
    pub nucleation_efficiency: f32,
    /// Целостность γ-TuRC кольца [0..1].
    pub ring_integrity: f32,
    /// Перицентриолярная плотность γ-TuRC [0..1].
    pub pericentriolar_density: f32,
}

impl Default for GammaRingState {
    fn default() -> Self {
        Self {
            nucleation_efficiency: 0.81, // 0.90 × 0.90
            ring_integrity: 0.90,
            pericentriolar_density: 0.90,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SaspDiffusionState — ECS-компонент (Уровень 0: пространственная диффузия SASP)
//
// Паракринный SASP-сигнал от сенесцентных клеток распространяется
// на соседние ниши. Эффективный SASP = local + diffused from neighbors.
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние пространственной диффузии SASP в нише.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaspDiffusionState {
    /// Локальный SASP, генерируемый данной нишей [0..1].
    pub local_sasp: f32,
    /// SASP, полученный от соседних ниш через диффузию [0..1].
    pub received_sasp: f32,
    /// Эффективный SASP = local + received [0..1].
    pub effective_sasp: f32,
    /// Радиус диффузии (количество соседних ниш) [единиц].
    pub diffusion_radius: u32,
}

impl Default for SaspDiffusionState {
    fn default() -> Self {
        Self {
            local_sasp: 0.0,
            received_sasp: 0.0,
            effective_sasp: 0.0,
            diffusion_radius: 3,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// NeuromuscularJunctionState — ECS-компонент (Уровень +1: нейромышечный синапс)
//
// Деградация нервно-мышечного соединения при старении:
// денервация → снижение плотности АХ-рецепторов → нарушение нейромышечной
// передачи → саркопения.
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние нервно-мышечного соединения (НМС).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuromuscularJunctionState {
    /// Плотность АХ-рецепторов [0..1].
    pub ach_receptor_density: f32,
    /// Эффективность синаптической передачи [0..1].
    pub synaptic_transmission: f32,
    /// Индекс денервации [0..1]; 0 = норма, 1 = полная денервация.
    pub denervation_index: f32,
    /// Способность к реиннервации [0..1].
    pub reinnervation_capacity: f32,
}

impl Default for NeuromuscularJunctionState {
    fn default() -> Self {
        Self {
            ach_receptor_density: 1.0,
            synaptic_transmission: 1.0,
            denervation_index: 0.0,
            reinnervation_capacity: 1.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SocialStressState — ECS-компонент (Уровень +4: социальный стресс)
//
// Социальная изоляция и социально-экономический стресс ускоряют
// биологическое старение через ось HPA и воспалительные пути.
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние социального стресса и его влияние на биологическое старение.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialStressState {
    /// Индекс одиночества [0..1]; 0 = полная социальная включённость.
    pub loneliness_index: f32,
    /// Социально-экономический стресс [0..1].
    pub socioeconomic_stress: f32,
    /// Социальная сплочённость [0..1]; = 1 − loneliness_index.
    pub social_cohesion: f32,
    /// Уровень окситоцина [0..1].
    pub oxytocin_level: f32,
    /// Аллостатическая нагрузка [0..1].
    pub allostatic_load: f32,
}

impl Default for SocialStressState {
    fn default() -> Self {
        Self {
            loneliness_index: 0.0,
            socioeconomic_stress: 0.0,
            social_cohesion: 1.0,
            oxytocin_level: 0.40,
            allostatic_load: 0.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// P52 — SenescenceAccumulationState
// Накопление сенесцентных клеток в нише и связанный SASP-сигнал.
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние накопления сенесцентных клеток в стволовой нише.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenescenceAccumulationState {
    /// Доля сенесцентных клеток в нише [0..1].
    pub senescent_fraction: f32,
    /// SASP-сигнал, производимый нишей [0..1].
    pub sasp_output: f32,
    /// Регенеративная ёмкость ниши — угнетение сенесцентными клетками [0..1].
    pub niche_regenerative_capacity: f32,
    /// Кумулятивная нагрузка сенесцентными клетками [0..∞, clamp→1].
    pub cumulative_burden: f32,
}

impl Default for SenescenceAccumulationState {
    fn default() -> Self {
        Self {
            senescent_fraction: 0.01,
            sasp_output: 0.0,
            niche_regenerative_capacity: 1.0,
            cumulative_burden: 0.0,
        }
    }
}

/// Параметры накопления сенесцентных клеток.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenescenceAccumulationParams {
    /// Базовый темп накопления за шаг.
    pub attrition_rate: f32,
    /// Эффективность замены сенесцентных клеток [0..1].
    pub replacement_efficiency: f32,
    /// Масштаб SASP-сигнала.
    pub sasp_scale: f32,
    /// Коэффициент угнетения регенерации ниши.
    pub niche_suppression_coeff: f32,
    /// Доля удаления сенесцентных клеток (сенолитики) [0..1].
    pub senolytic_clearance: f32,
}

impl Default for SenescenceAccumulationParams {
    fn default() -> Self {
        Self {
            attrition_rate: 0.0003,
            replacement_efficiency: 0.85,
            sasp_scale: 0.60,
            niche_suppression_coeff: 0.50,
            senolytic_clearance: 0.0,
        }
    }
}

/// Обновить состояние накопления сенесцентных клеток за один шаг dt.
///
/// * `ros_level`      — текущий уровень ROS в нише [0..1]
/// * `stem_cell_pool` — размер пула стволовых клеток [0..1] (не используется в текущей формуле,
///                      зарезервировано для расширения)
/// * `caii`           — индекс целостности придатков центриолей [0..1]
/// * `dt`             — размер шага [лет]
pub fn update_senescence_accumulation_state(
    state: &mut SenescenceAccumulationState,
    params: &SenescenceAccumulationParams,
    ros_level: f32,
    _stem_cell_pool: f32,
    caii: f32,
    dt: f32,
) {
    // Накопление: ROS и низкий CAII ускоряют индукцию сенесценции
    let induction = (params.attrition_rate + ros_level * 0.002) * (1.0 - caii * 0.5) * dt;
    let clearance = state.senescent_fraction * params.replacement_efficiency * 0.0001 * dt;
    let senolytic  = state.senescent_fraction * params.senolytic_clearance * dt;

    state.senescent_fraction = (state.senescent_fraction + induction - clearance - senolytic)
        .clamp(0.0, 1.0);

    state.sasp_output = state.senescent_fraction * params.sasp_scale;
    state.niche_regenerative_capacity =
        1.0 - state.senescent_fraction * params.niche_suppression_coeff;
    state.cumulative_burden =
        (state.cumulative_burden + state.senescent_fraction * dt).clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests_senescence_accumulation {
    use super::*;

    #[test]
    fn young_low_ros_high_caii_stays_small() {
        let mut state = SenescenceAccumulationState {
            senescent_fraction: 0.01,
            ..Default::default()
        };
        let params = SenescenceAccumulationParams::default();
        // Низкий ROS, высокий CAII, 10 шагов по 1 году
        for _ in 0..10 {
            update_senescence_accumulation_state(&mut state, &params, 0.05, 1.0, 0.95, 1.0);
        }
        assert!(
            state.senescent_fraction < 0.10,
            "young: senescent_fraction={:.4} должно оставаться малым",
            state.senescent_fraction
        );
    }

    #[test]
    fn aged_high_ros_accumulates_fast() {
        let mut state = SenescenceAccumulationState {
            senescent_fraction: 0.05,
            ..Default::default()
        };
        let params = SenescenceAccumulationParams::default();
        // Высокий ROS, низкий CAII, 30 шагов по 1 году
        for _ in 0..30 {
            update_senescence_accumulation_state(&mut state, &params, 0.80, 0.5, 0.30, 1.0);
        }
        assert!(
            state.senescent_fraction > 0.07,
            "aged high-ROS: senescent_fraction={:.4} должно расти выше начального 0.05",
            state.senescent_fraction
        );
    }

    #[test]
    fn senolytic_clearance_reduces_fraction() {
        let mut state_control = SenescenceAccumulationState {
            senescent_fraction: 0.30,
            ..Default::default()
        };
        let mut state_senolytic = SenescenceAccumulationState {
            senescent_fraction: 0.30,
            ..Default::default()
        };
        let params_control = SenescenceAccumulationParams::default();
        let params_senolytic = SenescenceAccumulationParams {
            senolytic_clearance: 0.50,
            ..Default::default()
        };
        for _ in 0..5 {
            update_senescence_accumulation_state(&mut state_control, &params_control, 0.2, 0.8, 0.7, 1.0);
            update_senescence_accumulation_state(&mut state_senolytic, &params_senolytic, 0.2, 0.8, 0.7, 1.0);
        }
        assert!(
            state_senolytic.senescent_fraction < state_control.senescent_fraction,
            "senolytic: fraction={:.4} < control={:.4}",
            state_senolytic.senescent_fraction,
            state_control.senescent_fraction
        );
    }

    #[test]
    fn sasp_output_proportional_to_fraction() {
        let mut state = SenescenceAccumulationState {
            senescent_fraction: 0.40,
            ..Default::default()
        };
        let params = SenescenceAccumulationParams::default();
        update_senescence_accumulation_state(&mut state, &params, 0.1, 0.8, 0.8, 0.1);
        let expected_sasp = state.senescent_fraction * params.sasp_scale;
        assert!(
            (state.sasp_output - expected_sasp).abs() < 1e-5,
            "sasp_output={:.4} должно ≈ fraction × scale={:.4}",
            state.sasp_output,
            expected_sasp
        );
    }

    #[test]
    fn niche_capacity_decreases_with_high_fraction() {
        let mut state = SenescenceAccumulationState {
            senescent_fraction: 0.60,
            ..Default::default()
        };
        let params = SenescenceAccumulationParams {
            attrition_rate: 0.0,
            senolytic_clearance: 0.0,
            ..Default::default()
        };
        update_senescence_accumulation_state(&mut state, &params, 0.0, 1.0, 1.0, 0.001);
        assert!(
            state.niche_regenerative_capacity < 0.80,
            "niche_capacity={:.4} должна снижаться при высокой сенесценции",
            state.niche_regenerative_capacity
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// P54 — PTMBurdenProfile
// Профиль суммарного PTM-бремени ткани как функция возраста.
// ─────────────────────────────────────────────────────────────────────────────

/// Профиль PTM-бремени для одного временного среза.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PTMBurdenProfile {
    /// Возраст [лет].
    pub age_years: f32,
    /// Уровень карбонилирования [0..1].
    pub carbonylation: f32,
    /// Гиперацетилирование [0..1].
    pub hyperacetylation: f32,
    /// Агрегация белков [0..1].
    pub aggregation: f32,
    /// Нарушение фосфорилирования [0..1].
    pub phospho_dysreg: f32,
    /// Потеря придатков (1 − CAII) [0..1].
    pub appendage_loss: f32,
    /// Суммарное взвешенное бремя [0..1].
    pub total_burden: f32,
}

impl PTMBurdenProfile {
    /// Создать профиль из состояния повреждения центриоли.
    pub fn from_damage_state(age: f32, damage: &CentriolarDamageState) -> Self {
        let appendage_loss = 1.0
            - (damage.cep164_integrity
                + damage.cep89_integrity
                + damage.ninein_integrity
                + damage.cep170_integrity)
                / 4.0;
        let total = damage.protein_carbonylation * 0.25
            + damage.tubulin_hyperacetylation * 0.20
            + damage.protein_aggregates * 0.25
            + damage.phosphorylation_dysregulation * 0.15
            + appendage_loss * 0.15;
        Self {
            age_years: age,
            carbonylation: damage.protein_carbonylation,
            hyperacetylation: damage.tubulin_hyperacetylation,
            aggregation: damage.protein_aggregates,
            phospho_dysreg: damage.phosphorylation_dysregulation,
            appendage_loss,
            total_burden: total,
        }
    }

    /// Найти возраст, при котором указанное поле пересекает порог 0.50.
    pub fn age_at_50_percent(trajectory: &[PTMBurdenProfile], field: &str) -> Option<f32> {
        for i in 1..trajectory.len() {
            let prev = Self::get_field(&trajectory[i - 1], field);
            let curr = Self::get_field(&trajectory[i], field);
            if prev < 0.50 && curr >= 0.50 {
                return Some(trajectory[i].age_years);
            }
        }
        None
    }

    fn get_field(p: &PTMBurdenProfile, field: &str) -> f32 {
        match field {
            "carbonylation"   => p.carbonylation,
            "hyperacetylation" => p.hyperacetylation,
            "aggregation"     => p.aggregation,
            "phospho_dysreg"  => p.phospho_dysreg,
            "appendage_loss"  => p.appendage_loss,
            _                 => p.total_burden,
        }
    }
}

#[cfg(test)]
mod tests_ptm_burden {
    use super::*;

    fn zero_damage() -> CentriolarDamageState {
        CentriolarDamageState::pristine()
    }

    #[test]
    fn from_damage_state_zeros_give_zero_fields() {
        let dam = zero_damage();
        let p = PTMBurdenProfile::from_damage_state(0.0, &dam);
        assert!(p.carbonylation < 1e-5, "carbonylation должно быть 0");
        assert!(p.aggregation < 1e-5,   "aggregation должно быть 0");
        assert!(p.total_burden < 1e-5,  "total_burden должно быть 0");
        // appendage_loss = 1 - (1+1+1+1)/4 = 0
        assert!(p.appendage_loss < 1e-5, "appendage_loss должно быть 0");
    }

    #[test]
    fn total_burden_is_correct_weighted_sum() {
        let mut dam = zero_damage();
        dam.protein_carbonylation       = 0.8;
        dam.tubulin_hyperacetylation    = 0.6;
        dam.protein_aggregates          = 0.4;
        dam.phosphorylation_dysregulation = 0.5;
        // appendages все 1.0 → appendage_loss = 0
        let p = PTMBurdenProfile::from_damage_state(50.0, &dam);
        let expected = 0.8 * 0.25 + 0.6 * 0.20 + 0.4 * 0.25 + 0.5 * 0.15 + 0.0 * 0.15;
        assert!(
            (p.total_burden - expected).abs() < 1e-5,
            "total_burden={:.4} ожидалось={:.4}",
            p.total_burden,
            expected
        );
    }

    #[test]
    fn age_at_50_percent_finds_correct_age() {
        // Строим траекторию с ростом карбонилирования
        let mut trajectory = Vec::new();
        for i in 0..=100 {
            let age = i as f32;
            let carb = (age / 100.0).powi(2); // 0→1 квадратично; пересекает 0.50 на ≈70.7 лет
            trajectory.push(PTMBurdenProfile {
                age_years: age,
                carbonylation: carb,
                hyperacetylation: 0.0,
                aggregation: 0.0,
                phospho_dysreg: 0.0,
                appendage_loss: 0.0,
                total_burden: carb,
            });
        }
        let age50 = PTMBurdenProfile::age_at_50_percent(&trajectory, "carbonylation");
        assert!(age50.is_some(), "должен найти возраст пересечения 0.50");
        let a = age50.unwrap();
        assert!(
            (a - 71.0).abs() < 2.0,
            "возраст пересечения ≈71 лет, получено={:.1}",
            a
        );
    }

    #[test]
    fn age_at_50_percent_returns_none_if_not_reached() {
        let trajectory: Vec<PTMBurdenProfile> = (0..=10)
            .map(|i| PTMBurdenProfile {
                age_years: i as f32,
                carbonylation: i as f32 * 0.03, // max = 0.30 < 0.50
                hyperacetylation: 0.0,
                aggregation: 0.0,
                phospho_dysreg: 0.0,
                appendage_loss: 0.0,
                total_burden: 0.0,
            })
            .collect();
        let age50 = PTMBurdenProfile::age_at_50_percent(&trajectory, "carbonylation");
        assert!(age50.is_none(), "не должен найти пересечения 0.50");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// P57 — TrackABCrossState
// Перекрёстная обратная связь между Track A (цилии) и Track B (веретено).
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние перекрёстной обратной связи Трек A ↔ Трек B.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackABCrossState {
    /// Штраф веретена из-за снижения функции цилий [0..1].
    pub cilia_to_spindle_penalty: f32,
    /// Штраф цилий из-за снижения точности веретена [0..1].
    pub spindle_to_cilia_penalty: f32,
    /// Нелинейная комбинированная дисфункция [0..1].
    pub combined_dysfunction: f32,
    /// Активен ли cross-talk (порог дисфункции превышен).
    pub cross_talk_active: bool,
}

impl Default for TrackABCrossState {
    fn default() -> Self {
        Self {
            cilia_to_spindle_penalty: 0.0,
            spindle_to_cilia_penalty: 0.0,
            combined_dysfunction: 0.0,
            cross_talk_active: false,
        }
    }
}

/// Параметры перекрёстной обратной связи Трек A ↔ Трек B.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackABCrossParams {
    /// Коэффициент связи цилии → веретено.
    pub cilia_spindle_coupling: f32,
    /// Коэффициент связи веретено → цилии.
    pub spindle_cilia_coupling: f32,
    /// Показатель нелинейности комбинированного эффекта.
    pub nonlinearity_exponent: f32,
    /// Порог дисфункции для включения cross-talk.
    pub activation_threshold: f32,
}

impl Default for TrackABCrossParams {
    fn default() -> Self {
        Self {
            cilia_spindle_coupling: 0.25,
            spindle_cilia_coupling: 0.15,
            nonlinearity_exponent: 1.8,
            activation_threshold: 0.30,
        }
    }
}

/// Обновить состояние перекрёстной обратной связи.
///
/// * `ciliary_function`  — текущая функциональность цилий [0..1]
/// * `spindle_fidelity`  — текущая точность веретена [0..1]
pub fn update_track_ab_cross(
    state: &mut TrackABCrossState,
    params: &TrackABCrossParams,
    ciliary_function: f32,
    spindle_fidelity: f32,
) {
    let cilia_deficit   = (1.0 - ciliary_function).clamp(0.0, 1.0);
    let spindle_deficit = (1.0 - spindle_fidelity).clamp(0.0, 1.0);

    state.cilia_to_spindle_penalty = cilia_deficit   * params.cilia_spindle_coupling;
    state.spindle_to_cilia_penalty = spindle_deficit * params.spindle_cilia_coupling;

    let combined_linear = cilia_deficit * 0.5 + spindle_deficit * 0.5;
    state.combined_dysfunction = combined_linear.powf(params.nonlinearity_exponent);

    state.cross_talk_active = cilia_deficit   > params.activation_threshold
        || spindle_deficit > params.activation_threshold;
}

#[cfg(test)]
mod tests_track_ab_cross {
    use super::*;

    #[test]
    fn both_healthy_no_crosstalk() {
        let mut state = TrackABCrossState::default();
        let params = TrackABCrossParams::default();
        update_track_ab_cross(&mut state, &params, 1.0, 1.0);
        assert!(!state.cross_talk_active, "оба здоровы — cross-talk не активен");
        assert!(state.combined_dysfunction < 1e-5, "нет дисфункции при здоровых треках");
        assert!(state.cilia_to_spindle_penalty < 1e-5);
        assert!(state.spindle_to_cilia_penalty < 1e-5);
    }

    #[test]
    fn low_cilia_activates_spindle_penalty() {
        let mut state = TrackABCrossState::default();
        let params = TrackABCrossParams::default();
        update_track_ab_cross(&mut state, &params, 0.50, 1.0); // cilia_deficit=0.50
        assert!(
            state.cilia_to_spindle_penalty > 0.0,
            "low cilia → spindle penalty > 0: {:.4}",
            state.cilia_to_spindle_penalty
        );
        assert!(
            state.cross_talk_active,
            "cilia_deficit=0.50 > threshold=0.30 → cross_talk_active"
        );
    }

    #[test]
    fn combined_dysfunction_worse_than_linear() {
        let mut state = TrackABCrossState::default();
        let params = TrackABCrossParams {
            nonlinearity_exponent: 1.8,
            ..Default::default()
        };
        // Средний дефицит = 0.50 (оба трека по 0.50)
        update_track_ab_cross(&mut state, &params, 0.50, 0.50);
        let linear = 0.50f32;
        // combined = linear^1.8 < linear → нелинейность снижает, но тест проверяет экспоненту
        assert!(
            (state.combined_dysfunction - linear.powf(1.8)).abs() < 1e-5,
            "combined_dysfunction={:.4} ожидалось linear^1.8={:.4}",
            state.combined_dysfunction,
            linear.powf(1.8)
        );
        // combined < linear (нелинейность работает)
        assert!(
            state.combined_dysfunction < linear,
            "combined({:.4}) < linear({:.4}) при exponent>1",
            state.combined_dysfunction,
            linear
        );
    }

    #[test]
    fn threshold_activates_cross_talk() {
        let mut state = TrackABCrossState::default();
        let params = TrackABCrossParams {
            activation_threshold: 0.30,
            ..Default::default()
        };
        // Чуть ниже порога → не активен
        update_track_ab_cross(&mut state, &params, 0.75, 1.0); // cilia_deficit=0.25
        assert!(!state.cross_talk_active, "deficit=0.25 < threshold=0.30 → не активен");

        // Чуть выше порога → активен
        update_track_ab_cross(&mut state, &params, 0.65, 1.0); // cilia_deficit=0.35
        assert!(state.cross_talk_active, "deficit=0.35 > threshold=0.30 → активен");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// P58 — CytoplasmQCState
// Контроль качества цитоплазмы при асимметричном делении.
// ─────────────────────────────────────────────────────────────────────────────

/// Состояние контроля качества цитоплазмы при асимметричном делении.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CytoplasmQCState {
    /// Эффективность контроля качества [0..1].
    pub qc_efficiency: f32,
    /// Доля повреждённого груза в цитоплазме [0..1].
    pub damaged_cargo_fraction: f32,
    /// Качество сортировки митохондрий [0..1].
    pub mitochondrial_sort_quality: f32,
    /// Чистота стволовой дочерней клетки [0..1].
    pub stem_daughter_purity: f32,
}

impl Default for CytoplasmQCState {
    fn default() -> Self {
        Self {
            qc_efficiency: 0.90,
            damaged_cargo_fraction: 0.0,
            mitochondrial_sort_quality: 0.80,
            stem_daughter_purity: 1.0,
        }
    }
}

/// Параметры контроля качества цитоплазмы.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CytoplasmQCParams {
    /// Базовая эффективность QC [0..1].
    pub qc_efficiency_base: f32,
    /// Коэффициент связи повреждений с QC.
    pub damage_qc_coupling: f32,
    /// Базовая эффективность сортировки митохондрий [0..1].
    pub mitochondrial_sort_efficiency: f32,
    /// Скорость снижения QC с возрастом (за год).
    pub age_qc_decline: f32,
}

impl Default for CytoplasmQCParams {
    fn default() -> Self {
        Self {
            qc_efficiency_base: 0.90,
            damage_qc_coupling: 0.60,
            mitochondrial_sort_efficiency: 0.80,
            age_qc_decline: 0.003,
        }
    }
}

/// Обновить состояние контроля качества цитоплазмы.
///
/// * `total_damage`  — суммарный индекс повреждений (из CentriolarDamageState) [0..1]
/// * `aggregates`    — уровень агрегатов белков [0..1]
/// * `age_years`     — возраст клетки/организма [лет]
/// * `_dt`           — размер шага [лет] (зарезервировано)
pub fn update_cytoplasm_qc(
    state: &mut CytoplasmQCState,
    params: &CytoplasmQCParams,
    total_damage: f32,
    aggregates: f32,
    age_years: f32,
    _dt: f32,
) {
    state.qc_efficiency = (params.qc_efficiency_base
        - age_years * params.age_qc_decline
        - total_damage * params.damage_qc_coupling * 0.30)
        .clamp(0.05, 1.0);

    state.damaged_cargo_fraction = aggregates * (1.0 - state.qc_efficiency);
    state.mitochondrial_sort_quality =
        params.mitochondrial_sort_efficiency * state.qc_efficiency;
    state.stem_daughter_purity =
        state.qc_efficiency * (1.0 - state.damaged_cargo_fraction * 0.5);
}

#[cfg(test)]
mod tests_cytoplasm_qc {
    use super::*;

    #[test]
    fn young_healthy_high_qc_and_purity() {
        let mut state = CytoplasmQCState::default();
        let params = CytoplasmQCParams::default();
        update_cytoplasm_qc(&mut state, &params, 0.0, 0.0, 0.0, 1.0);
        assert!(
            state.qc_efficiency > 0.85,
            "молодой здоровый: qc_efficiency={:.4} должно быть высоким",
            state.qc_efficiency
        );
        assert!(
            state.stem_daughter_purity > 0.85,
            "stem_daughter_purity={:.4} должно быть высоким",
            state.stem_daughter_purity
        );
    }

    #[test]
    fn old_age_reduces_qc_efficiency() {
        let mut state_young = CytoplasmQCState::default();
        let mut state_old   = CytoplasmQCState::default();
        let params = CytoplasmQCParams::default();
        update_cytoplasm_qc(&mut state_young, &params, 0.0, 0.0, 10.0,  1.0);
        update_cytoplasm_qc(&mut state_old,   &params, 0.0, 0.0, 80.0, 1.0);
        assert!(
            state_old.qc_efficiency < state_young.qc_efficiency,
            "old({:.4}) < young({:.4})",
            state_old.qc_efficiency,
            state_young.qc_efficiency
        );
    }

    #[test]
    fn high_damage_increases_damaged_cargo() {
        let mut state_low  = CytoplasmQCState::default();
        let mut state_high = CytoplasmQCState::default();
        let params = CytoplasmQCParams::default();
        update_cytoplasm_qc(&mut state_low,  &params, 0.0, 0.10, 40.0, 1.0);
        update_cytoplasm_qc(&mut state_high, &params, 0.8, 0.80, 40.0, 1.0);
        assert!(
            state_high.damaged_cargo_fraction > state_low.damaged_cargo_fraction,
            "high damage: damaged_cargo={:.4} > low_damage={:.4}",
            state_high.damaged_cargo_fraction,
            state_low.damaged_cargo_fraction
        );
    }

    #[test]
    fn stem_daughter_purity_function_of_qc_and_cargo() {
        let mut state = CytoplasmQCState::default();
        let params = CytoplasmQCParams::default();
        update_cytoplasm_qc(&mut state, &params, 0.5, 0.5, 50.0, 1.0);
        let expected = state.qc_efficiency * (1.0 - state.damaged_cargo_fraction * 0.5);
        assert!(
            (state.stem_daughter_purity - expected).abs() < 1e-5,
            "purity={:.4} ожидалось={:.4}",
            state.stem_daughter_purity,
            expected
        );
    }

    #[test]
    fn mito_sort_quality_equals_base_times_qc() {
        let mut state = CytoplasmQCState::default();
        let params = CytoplasmQCParams::default();
        update_cytoplasm_qc(&mut state, &params, 0.3, 0.2, 30.0, 1.0);
        let expected = params.mitochondrial_sort_efficiency * state.qc_efficiency;
        assert!(
            (state.mitochondrial_sort_quality - expected).abs() < 1e-5,
            "mito_sort={:.4} ожидалось={:.4}",
            state.mitochondrial_sort_quality,
            expected
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// P60 — Межвидовые профили (InterspeciesScaling)
// ─────────────────────────────────────────────────────────────────────────────

/// Видовой профиль для межвидового масштабирования CDATA.
#[derive(Debug, Clone)]
pub struct SpeciesProfile {
    /// Название вида.
    pub name: &'static str,
    /// Известная продолжительность жизни [лет].
    pub lifespan_years: f32,
    /// Относительный метаболизм (1.0 = человек).
    pub metabolic_rate_relative: f32,
    /// Масса тела [кг].
    pub body_mass_kg: f32,
    /// Масштаб base_detach_probability относительно человека.
    pub base_detach_scale: f32,
    /// Масштаб продукции ROS относительно человека.
    pub ros_scale: f32,
}

/// Известные виды с откалиброванными CDATA-параметрами.
pub const SPECIES_PROFILES: &[SpeciesProfile] = &[
    SpeciesProfile {
        name: "mouse",
        lifespan_years: 2.5,
        metabolic_rate_relative: 7.0,
        body_mass_kg: 0.025,
        base_detach_scale: 8.0,
        ros_scale: 3.5,
    },
    SpeciesProfile {
        name: "human",
        lifespan_years: 78.0,
        metabolic_rate_relative: 1.0,
        body_mass_kg: 70.0,
        base_detach_scale: 1.0,
        ros_scale: 1.0,
    },
    SpeciesProfile {
        name: "bat",
        lifespan_years: 40.0,
        metabolic_rate_relative: 6.5,
        body_mass_kg: 0.020,
        base_detach_scale: 1.5,
        ros_scale: 0.4,
    },
    SpeciesProfile {
        name: "naked_mole_rat",
        lifespan_years: 30.0,
        metabolic_rate_relative: 3.0,
        body_mass_kg: 0.035,
        base_detach_scale: 0.8,
        ros_scale: 0.3,
    },
];

/// Предсказать продолжительность жизни из параметров CDATA.
///
/// Обратная функция: чем выше detach_scale × ros_scale, тем короче жизнь.
pub fn predicted_lifespan_from_cdata(species: &SpeciesProfile) -> f32 {
    78.0 / (species.base_detach_scale * species.ros_scale).sqrt()
}

// ─────────────────────────────────────────────────────────────────────────────
// P63 — SystemicCircadianState
// Агрегатор циркадных состояний всех ниш на уровне организма.
// ─────────────────────────────────────────────────────────────────────────────

/// Системное циркадное состояние организма — агрегатор всех ниш.
///
/// На уровне ниши используется `CircadianState`.
/// `SystemicCircadianState` агрегирует амплитуды и фазы всех ниш,
/// моделирует системную синхронизацию и «социальный джетлаг».
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemicCircadianState {
    /// Среднее значение amplitude по всем нишам [0..1].
    pub global_amplitude: f32,
    /// Фазовая когерентность: синхронизация между нишами [0..1].
    /// 1.0 = все ниши синхронны; 0.0 = полная десинхронизация.
    pub phase_coherence: f32,
    /// Дополнительный SASP-буст от нарушения циркадных ритмов [0..sasp_circadian_factor].
    pub circadian_sasp_boost: f32,
    /// Накопленный «социальный джетлаг» [0..1].
    /// Растёт при десинхронизации, отражает хронический циркадный стресс.
    pub jet_lag_index: f32,
    /// Эффективность мелатонина [0..1].
    /// Снижается с возрастом (age × melatonin_age_decline).
    pub melatonin_efficacy: f32,
}

impl Default for SystemicCircadianState {
    fn default() -> Self {
        Self {
            global_amplitude:       1.0,
            phase_coherence:        1.0,
            circadian_sasp_boost:   0.0,
            jet_lag_index:          0.0,
            melatonin_efficacy:     1.0,
        }
    }
}

/// Параметры системного циркадного состояния.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemicCircadianParams {
    /// Сила синхронизации между нишами. По умолчанию: 0.30.
    pub phase_coupling_strength: f32,
    /// Вклад десинхронизации в системный SASP. По умолчанию: 0.20.
    pub sasp_circadian_factor: f32,
    /// Скорость снижения мелатонина с возрастом [/год]. По умолчанию: 0.008.
    pub melatonin_age_decline: f32,
    /// Скорость накопления джетлага [/шаг при десинхронизации]. По умолчанию: 0.0002.
    pub jet_lag_accumulation: f32,
}

impl Default for SystemicCircadianParams {
    fn default() -> Self {
        Self {
            phase_coupling_strength: 0.30,
            sasp_circadian_factor:   0.20,
            melatonin_age_decline:   0.008,
            jet_lag_accumulation:    0.0002,
        }
    }
}

/// Обновить системное циркадное состояние за один шаг.
///
/// # Аргументы
/// * `state`           — изменяемое системное состояние.
/// * `params`          — параметры.
/// * `niche_amplitudes` — вектор амплитуд из `CircadianState.amplitude` всех ниш.
/// * `age_years`       — возраст организма в годах (для расчёта мелатонина).
/// * `dt`              — шаг времени [лет].
pub fn update_systemic_circadian(
    state: &mut SystemicCircadianState,
    params: &SystemicCircadianParams,
    niche_amplitudes: &[f32],
    age_years: f32,
    dt: f32,
) {
    // 1. Глобальная амплитуда = среднее по нишам; при пустом срезе = 1.0
    state.global_amplitude = if niche_amplitudes.is_empty() {
        1.0
    } else {
        niche_amplitudes.iter().copied().sum::<f32>() / niche_amplitudes.len() as f32
    };

    // 2. Phase coherence: чем меньше стандартное отклонение амплитуд,
    //    тем выше coherence.
    //    sd = sqrt(mean((amp - mean)²))
    //    coherence = (1.0 - sd × 2.0).clamp(0.0, 1.0)
    state.phase_coherence = if niche_amplitudes.len() < 2 {
        1.0
    } else {
        let mean = state.global_amplitude;
        let variance = niche_amplitudes
            .iter()
            .map(|&a| (a - mean).powi(2))
            .sum::<f32>()
            / niche_amplitudes.len() as f32;
        let sd = variance.sqrt();
        (1.0 - sd * 2.0).clamp(0.0, 1.0)
    };

    // 3. SASP-буст от десинхронизации
    state.circadian_sasp_boost =
        (1.0 - state.phase_coherence) * params.sasp_circadian_factor;

    // 4. Мелатонин снижается с возрастом
    state.melatonin_efficacy =
        (1.0 - age_years * params.melatonin_age_decline).clamp(0.1, 1.0);

    // 5. Накопление джетлага
    state.jet_lag_index = (state.jet_lag_index
        + (1.0 - state.phase_coherence) * params.jet_lag_accumulation * dt)
        .clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests_systemic_circadian {
    use super::*;

    /// 1. Все ниши с одинаковой амплитудой → phase_coherence = 1.0, sasp_boost = 0.
    #[test]
    fn synchronized_niches() {
        let mut state = SystemicCircadianState::default();
        let params = SystemicCircadianParams::default();
        let niches = vec![0.8, 0.8, 0.8, 0.8];

        update_systemic_circadian(&mut state, &params, &niches, 30.0, 1.0);

        assert!(
            (state.phase_coherence - 1.0).abs() < 1e-5,
            "phase_coherence должен быть 1.0 при одинаковых амплитудах: {:.6}",
            state.phase_coherence
        );
        assert!(
            state.circadian_sasp_boost.abs() < 1e-5,
            "sasp_boost должен быть 0 при синхронизации: {:.6}",
            state.circadian_sasp_boost
        );
    }

    /// 2. Разные амплитуды → phase_coherence < 1.0, sasp_boost > 0.
    #[test]
    fn desynchronized_niches() {
        let mut state = SystemicCircadianState::default();
        let params = SystemicCircadianParams::default();
        let niches = vec![1.0, 0.5, 0.2, 0.9]; // разброс

        update_systemic_circadian(&mut state, &params, &niches, 30.0, 1.0);

        assert!(
            state.phase_coherence < 1.0,
            "phase_coherence должен быть < 1.0 при десинхронизации: {:.6}",
            state.phase_coherence
        );
        assert!(
            state.circadian_sasp_boost > 0.0,
            "sasp_boost должен быть > 0 при десинхронизации: {:.6}",
            state.circadian_sasp_boost
        );
    }

    /// 3. Мелатонин снижается с возрастом.
    #[test]
    fn melatonin_declines_with_age() {
        let params = SystemicCircadianParams::default();
        let niches = vec![0.8, 0.8];

        let mut state20 = SystemicCircadianState::default();
        update_systemic_circadian(&mut state20, &params, &niches, 20.0, 1.0);

        let mut state70 = SystemicCircadianState::default();
        update_systemic_circadian(&mut state70, &params, &niches, 70.0, 1.0);

        assert!(
            state70.melatonin_efficacy < state20.melatonin_efficacy,
            "melatonin_efficacy должен быть ниже в 70 лет ({:.4}) чем в 20 ({:.4})",
            state70.melatonin_efficacy,
            state20.melatonin_efficacy
        );
    }

    /// 4. Несколько шагов с десинхронизацией → jet_lag_index растёт.
    #[test]
    fn jet_lag_accumulates() {
        let mut state = SystemicCircadianState::default();
        let params = SystemicCircadianParams::default();
        let niches = vec![1.0, 0.3, 0.7, 0.2]; // разброс → десинхронизация

        let initial = state.jet_lag_index;
        for _ in 0..100 {
            update_systemic_circadian(&mut state, &params, &niches, 40.0, 1.0);
        }
        assert!(
            state.jet_lag_index > initial,
            "jet_lag_index должен нарастать: {:.6} > {:.6}",
            state.jet_lag_index,
            initial
        );
    }

    /// 5. Пустой срез → global_amplitude = 1.0.
    #[test]
    fn empty_niches() {
        let mut state = SystemicCircadianState::default();
        let params = SystemicCircadianParams::default();

        update_systemic_circadian(&mut state, &params, &[], 40.0, 1.0);

        assert!(
            (state.global_amplitude - 1.0).abs() < 1e-5,
            "При пустом срезе global_amplitude должен быть 1.0: {:.6}",
            state.global_amplitude
        );
    }

    /// 6. melatonin_efficacy не падает ниже 0.1 (clamp).
    #[test]
    fn melatonin_clamped_at_minimum() {
        let mut state = SystemicCircadianState::default();
        let params = SystemicCircadianParams::default();
        let niches = vec![0.8];

        // age = 200 → 1.0 - 200 × 0.008 = -0.6 → должно быть clamp до 0.1
        update_systemic_circadian(&mut state, &params, &niches, 200.0, 1.0);

        assert!(
            state.melatonin_efficacy >= 0.1,
            "melatonin_efficacy не должен падать ниже 0.1: {:.6}",
            state.melatonin_efficacy
        );
        assert!(
            (state.melatonin_efficacy - 0.1).abs() < 1e-5,
            "При age=200 melatonin должен быть 0.1 (clamp): {:.6}",
            state.melatonin_efficacy
        );
    }
}

#[cfg(test)]
mod tests_interspecies {
    use super::*;

    fn find_species(name: &str) -> &'static SpeciesProfile {
        SPECIES_PROFILES.iter().find(|s| s.name == name).unwrap()
    }

    #[test]
    fn mouse_shorter_than_human() {
        let mouse = find_species("mouse");
        let human = find_species("human");
        let pred_mouse = predicted_lifespan_from_cdata(mouse);
        let pred_human = predicted_lifespan_from_cdata(human);
        assert!(
            pred_mouse < pred_human,
            "mouse predicted({:.2}) < human predicted({:.2})",
            pred_mouse,
            pred_human
        );
    }

    #[test]
    fn bat_longer_than_mouse_despite_high_metabolic_rate() {
        let bat   = find_species("bat");
        let mouse = find_species("mouse");
        let pred_bat   = predicted_lifespan_from_cdata(bat);
        let pred_mouse = predicted_lifespan_from_cdata(mouse);
        assert!(
            pred_bat > pred_mouse,
            "bat({:.2}) > mouse({:.2}) несмотря на высокий метаболизм",
            pred_bat,
            pred_mouse
        );
    }

    #[test]
    fn naked_mole_rat_longevity_from_low_ros() {
        let nmr   = find_species("naked_mole_rat");
        let mouse = find_species("mouse");
        let pred_nmr   = predicted_lifespan_from_cdata(nmr);
        let pred_mouse = predicted_lifespan_from_cdata(mouse);
        assert!(
            pred_nmr > pred_mouse,
            "naked_mole_rat({:.2}) > mouse({:.2}) из-за низкого ROS",
            pred_nmr,
            pred_mouse
        );
    }

    #[test]
    fn human_prediction_close_to_known() {
        let human = find_species("human");
        let pred = predicted_lifespan_from_cdata(human);
        // human: detach_scale=1.0, ros_scale=1.0 → pred = 78.0
        assert!(
            (pred - 78.0).abs() < 1.0,
            "human predicted={:.2} ожидалось ≈78",
            pred
        );
    }
}
