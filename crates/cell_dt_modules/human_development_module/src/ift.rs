//! Внутрижгутиковый транспорт (IFT) — уровень -2: цитоскелет.
//!
//! IFT-B (антероградный): KIF3A/KIF3B → SMO, GLI к кончику.
//! IFT-A (ретроградный):  DYNC2 → возврат «пустых» комплексов.
//! Без IFT: Shh-ответ = 0 даже при интактной CEP164.
//!
//! # Формулы
//!
//! ```text
//! anterograde_velocity = cep164 × ninein × (1 − aggregates × 0.4)
//! retrograde_velocity  = 1.0 − phospho_dysreg × 0.5
//! cargo_delivery = min(antero, retro) × vesicle_availability
//! ```

use cell_dt_core::components::IFTState;

/// Параметры IFT-динамики.
#[derive(Debug, Clone)]
pub struct IFTParams {
    /// Вклад агрегатов в торможение антероградного IFT [0..1].
    pub aggregates_antero_inhibition: f32,
    /// Вклад phospho_dysreg в торможение ретроградного IFT [0..1].
    pub phospho_retro_inhibition:     f32,
}

impl Default for IFTParams {
    fn default() -> Self {
        Self {
            aggregates_antero_inhibition: 0.40,
            phospho_retro_inhibition:     0.50,
        }
    }
}

/// Обновить IFTState на один шаг.
///
/// # Аргументы
/// * `ift`                  — состояние (in/out).
/// * `cep164`               — целостность CEP164 [0..1].
/// * `ninein`               — целостность Ninein [0..1].
/// * `aggregates`           — уровень белковых агрегатов [0..1].
/// * `phospho_dysreg`       — дисрегуляция фосфорилирования [0..1].
/// * `vesicle_availability` — доступность везикул из Гольджи [0..1].
/// * `params`               — параметры.
pub fn update_ift_state(
    ift:                  &mut IFTState,
    cep164:               f32,
    ninein:               f32,
    aggregates:           f32,
    phospho_dysreg:       f32,
    vesicle_availability: f32,
    params:               &IFTParams,
) {
    // Антероградный IFT: CEP164 (докинг IFT) × Ninein (якорение МТ) × агрегаты
    ift.anterograde_velocity =
        (cep164 * ninein * (1.0 - aggregates * params.aggregates_antero_inhibition))
            .clamp(0.0, 1.0);

    // Ретроградный IFT: менее чувствителен к CEP164, больше к PLK4-дисрегуляции
    ift.retrograde_velocity =
        (1.0 - phospho_dysreg * params.phospho_retro_inhibition)
            .clamp(0.0, 1.0);

    ift.update_derived(vesicle_availability);
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> IFTParams { IFTParams::default() }

    #[test]
    fn test_pristine_full_ift() {
        let s = IFTState::pristine();
        assert!((s.anterograde_velocity - 1.0).abs() < 1e-5);
        assert!((s.cargo_delivery - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cep164_loss_reduces_anterograde() {
        let mut s = IFTState::pristine();
        update_ift_state(&mut s, 0.3, 1.0, 0.0, 0.0, 1.0, &p());
        assert!(s.anterograde_velocity < 0.4,
            "CEP164↓ → антероградный IFT↓");
    }

    #[test]
    fn test_aggregates_block_anterograde() {
        let mut s = IFTState::pristine();
        update_ift_state(&mut s, 1.0, 1.0, 0.8, 0.0, 1.0, &p());
        assert!(s.anterograde_velocity < 0.70,
            "Агрегаты → блок антероградного IFT");
    }

    #[test]
    fn test_phospho_blocks_retrograde() {
        let mut s = IFTState::pristine();
        update_ift_state(&mut s, 1.0, 1.0, 0.0, 1.0, 1.0, &p());
        assert!(s.retrograde_velocity < 0.55,
            "phospho_dysreg → ретроградный IFT↓");
    }

    #[test]
    fn test_golgi_vesicles_limit_cargo() {
        let mut s = IFTState::pristine();
        update_ift_state(&mut s, 1.0, 1.0, 0.0, 0.0, 0.3, &p());
        assert!(s.cargo_delivery <= 0.31,
            "Мало везикул Гольджи → cargo_delivery↓");
    }

    #[test]
    fn test_cargo_is_min_of_velocities_times_vesicle() {
        let mut s = IFTState::pristine();
        update_ift_state(&mut s, 0.6, 1.0, 0.0, 0.4, 0.8, &p());
        let expected = s.anterograde_velocity.min(s.retrograde_velocity) * 0.8;
        assert!((s.cargo_delivery - expected).abs() < 1e-5,
            "cargo = min(antero, retro) × vesicle");
    }
}
