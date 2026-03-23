//! Интеграция OrganState в ECS — уровень +2.
//!
//! После всех модульных шагов собирает HumanDevelopmentComponent по tissue_type,
//! вычисляет functional_capacity для каждого органа и обновляет OrganState
//! сущности в ECS-мире. При полиорганной недостаточности (≥2 органов ниже
//! порога) орган-уровень смерти записывается в OrganismState.
//!
//! # Алгоритм
//!
//! ```text
//! Для каждого tissue_type:
//!   fc_i = stem_cell_pool_i × (1 − senescent_fraction_i)
//!   functional_capacity_group = mean(fc_i)
//!
//! → OrganState.functional_reserve = functional_capacity_group (если exists)
//! → OrganState.update_failure_status()
//!
//! Если count(is_failing) >= 2 → OrganismState.is_alive = false
//! ```

use cell_dt_core::{
    hecs::World,
    components::{OrganState, OrganType, OrganismState, TissueType},
};
use std::collections::HashMap;

/// Маппинг TissueType → OrganType (для агрегации ниш по органам).
pub fn tissue_to_organ(tissue: TissueType) -> Option<OrganType> {
    match tissue {
        TissueType::Blood      => Some(OrganType::ImmuneSystem),
        TissueType::Neural     => Some(OrganType::Brain),
        TissueType::Muscle     => Some(OrganType::Muscle),
        TissueType::Epithelial => Some(OrganType::Intestine),
        TissueType::Heart      => Some(OrganType::Heart),
        TissueType::Kidney     => Some(OrganType::Kidney),
        TissueType::Liver      => Some(OrganType::Liver),
        TissueType::Lung       => Some(OrganType::Lung),
        TissueType::Skin       => Some(OrganType::Skin),
        TissueType::Bone       => Some(OrganType::Bone),
        _                      => None,
    }
}

/// Данные одной ниши для агрегации органа.
pub struct NicheCapacityData {
    pub organ_type: OrganType,
    pub stem_cell_pool: f32,
    pub senescent_fraction: f32,
}

impl Clone for NicheCapacityData {
    fn clone(&self) -> Self {
        Self {
            organ_type: self.organ_type,
            stem_cell_pool: self.stem_cell_pool,
            senescent_fraction: self.senescent_fraction,
        }
    }
}

/// Агрегировать данные ниш по органам.
///
/// Возвращает HashMap<OrganType, (mean_fc, niche_count)>.
pub fn aggregate_niches_by_organ(
    niches: &[NicheCapacityData],
) -> HashMap<OrganType, (f32, u32)> {
    let mut organ_fc: HashMap<OrganType, (f32, u32)> = HashMap::new();
    for n in niches {
        let fc = n.stem_cell_pool * (1.0 - n.senescent_fraction);
        let entry = organ_fc.entry(n.organ_type).or_insert((0.0, 0));
        entry.0 += fc;
        entry.1 += 1;
    }
    // Нормировать на количество ниш
    for (_, (sum, count)) in organ_fc.iter_mut() {
        if *count > 0 {
            *sum /= *count as f32;
        }
    }
    organ_fc
}

/// Обновить OrganState-сущности в ECS из агрегированных данных ниш.
///
/// Итерирует по всем сущностям с OrganState и обновляет их functional_reserve
/// из предвычисленных данных. Затем проверяет полиорганную недостаточность.
pub fn update_organ_states_in_world(
    world: &mut World,
    organ_capacities: &HashMap<OrganType, (f32, u32)>,
) {
    // Обновить OrganState сущности
    for (_, organ) in world.query_mut::<&mut OrganState>() {
        if let Some(&(mean_fc, niche_count)) = organ_capacities.get(&organ.organ_type) {
            organ.functional_reserve = mean_fc.clamp(0.0, 1.0);
            organ.niche_count = niche_count;
            organ.update_failure_status();
        }
    }

    // Проверить полиорганную недостаточность
    let failing_count: usize = world
        .query::<&OrganState>()
        .iter()
        .filter(|(_, o)| o.is_failing)
        .count();

    if failing_count >= 2 {
        for (_, organism) in world.query_mut::<&mut OrganismState>() {
            organism.is_alive = false;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Тесты
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_niches(organ: OrganType, count: usize, pool: f32, sf: f32) -> Vec<NicheCapacityData> {
        (0..count).map(|_| NicheCapacityData {
            organ_type: organ,
            stem_cell_pool: pool,
            senescent_fraction: sf,
        }).collect()
    }

    /// Один орган в недостаточности — смерть не наступает
    #[test]
    fn test_single_organ_failure_no_death() {
        let mut world = World::new();
        // Добавить OrganState (Heart — в недостаточности)
        let mut heart = OrganState::new(OrganType::Heart);
        heart.functional_reserve = 0.05; // ниже threshold 0.20
        heart.is_failing = true;
        world.spawn((heart,));
        // Добавить OrganState (Brain — здоровый)
        world.spawn((OrganState::new(OrganType::Brain),));
        // Добавить OrganismState
        world.spawn((OrganismState::new(),));

        // Агрегировать: 1 орган failing
        let capacities = HashMap::new();
        update_organ_states_in_world(&mut world, &capacities);

        // Только 1 failing → организм жив
        for (_, organism) in world.query::<&OrganismState>().iter() {
            assert!(organism.is_alive,
                "1 орган в failure → организм должен быть жив");
        }
    }

    /// Два органа в недостаточности → полиорганная недостаточность → смерть
    #[test]
    fn test_two_organs_failure_causes_death() {
        let mut world = World::new();
        let mut heart = OrganState::new(OrganType::Heart);
        heart.is_failing = true;
        let mut kidney = OrganState::new(OrganType::Kidney);
        kidney.is_failing = true;
        world.spawn((heart,));
        world.spawn((kidney,));
        world.spawn((OrganismState::new(),));

        let capacities = HashMap::new();
        update_organ_states_in_world(&mut world, &capacities);

        for (_, organism) in world.query::<&OrganismState>().iter() {
            assert!(!organism.is_alive,
                "2 органа в failure → организм должен умереть");
        }
    }

    /// Здоровые ниши обновляют functional_reserve органа
    #[test]
    fn test_organ_state_updates_from_niches() {
        let niches: Vec<_> = make_niches(OrganType::Liver, 3, 0.80, 0.10);
        let capacities = aggregate_niches_by_organ(&niches);

        // mean_fc = 0.80 × (1 − 0.10) = 0.72
        let (mean_fc, count) = capacities[&OrganType::Liver];
        assert!((mean_fc - 0.72).abs() < 1e-5,
            "mean_fc = pool×(1-sf): {:.4}", mean_fc);
        assert_eq!(count, 3, "Количество ниш: {}", count);
    }

    /// Здоровые органы → без недостаточности
    #[test]
    fn test_healthy_organs_no_failure() {
        let mut world = World::new();
        world.spawn((OrganState::new(OrganType::Heart),));
        world.spawn((OrganState::new(OrganType::Brain),));
        world.spawn((OrganismState::new(),));

        // Здоровые ниши
        let mut niches = make_niches(OrganType::Heart, 3, 0.90, 0.05);
        niches.extend(make_niches(OrganType::Brain, 3, 0.85, 0.08));
        let capacities = aggregate_niches_by_organ(&niches);
        update_organ_states_in_world(&mut world, &capacities);

        for (_, organism) in world.query::<&OrganismState>().iter() {
            assert!(organism.is_alive,
                "Здоровые органы → организм жив");
        }
        for (_, organ) in world.query::<&OrganState>().iter() {
            assert!(!organ.is_failing,
                "Здоровые ниши → орган не в недостаточности: {:?}", organ.organ_type);
        }
    }
}
