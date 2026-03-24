//! CDATA-экспортёр: собирает данные CDATA-компонентов из ECS и сохраняет в CSV.
//!
//! Колонки: step, entity_id, tissue, age_years, stage, damage_score,
//!          myeloid_bias, spindle_fidelity, ciliary_function, frailty, phenotype_count

use crate::IoResult;
use csv::Writer;
use human_development_module::HumanDevelopmentComponent;
use myeloid_shift_module::MyeloidShiftComponent;
use cell_dt_core::{CdataCollect, hecs::World};
use std::path::{Path, PathBuf};

/// Одна строка CDATA-экспорта (одна сущность, один шаг)
#[derive(Debug, Clone)]
pub struct CdataRecord {
    pub step: u64,
    pub entity_id: u64,
    pub tissue: String,
    pub age_years: f64,
    pub stage: String,
    pub damage_score: f32,
    /// Миелоидный сдвиг (0.0, если `MyeloidShiftComponent` отсутствует)
    pub myeloid_bias: f32,
    pub spindle_fidelity: f32,
    pub ciliary_function: f32,
    /// Frailty = 1 − functional_capacity
    pub frailty: f32,
    pub phenotype_count: usize,
    // P67: PTM-траектории
    /// Карбонилирование белков (окислительный стресс) [0..1]
    pub ptm_carbonylation: f32,
    /// Гиперацетилирование тубулина [0..1]
    pub ptm_hyperacetylation: f32,
    /// Агрегаты белков (CPAP, CEP290) [0..1]
    pub ptm_aggregation: f32,
    /// Нарушение фосфорилирования (PLK4, NEK2) [0..1]
    pub ptm_phospho_dysreg: f32,
    /// Потеря придатков = 1 − mean(cep164, cep89, ninein, cep170) [0..1]
    pub ptm_appendage_loss: f32,
}

impl CdataRecord {
    pub fn csv_headers() -> Vec<&'static str> {
        vec![
            "step", "entity_id", "tissue", "age_years", "stage",
            "damage_score", "myeloid_bias", "spindle_fidelity",
            "ciliary_function", "frailty", "phenotype_count",
            "ptm_carbonylation", "ptm_hyperacetylation", "ptm_aggregation",
            "ptm_phospho_dysreg", "ptm_appendage_loss",
        ]
    }

    pub fn to_csv_record(&self) -> Vec<String> {
        vec![
            self.step.to_string(),
            self.entity_id.to_string(),
            self.tissue.clone(),
            format!("{:.4}", self.age_years),
            self.stage.clone(),
            format!("{:.6}", self.damage_score),
            format!("{:.6}", self.myeloid_bias),
            format!("{:.6}", self.spindle_fidelity),
            format!("{:.6}", self.ciliary_function),
            format!("{:.6}", self.frailty),
            self.phenotype_count.to_string(),
            format!("{:.6}", self.ptm_carbonylation),
            format!("{:.6}", self.ptm_hyperacetylation),
            format!("{:.6}", self.ptm_aggregation),
            format!("{:.6}", self.ptm_phospho_dysreg),
            format!("{:.6}", self.ptm_appendage_loss),
        ]
    }
}

/// Экспортёр CDATA-данных из ECS-мира в CSV-файлы.
///
/// # Использование
/// ```ignore
/// use cell_dt_io::CdataExporter;
///
/// let mut exporter = CdataExporter::new("output/cdata", "run");
/// // в цикле симуляции:
/// exporter.collect(sim.world(), sim.current_step());
/// if step % 100 == 0 {
///     exporter.save_snapshot(step).unwrap();
/// }
/// ```
pub struct CdataExporter {
    output_dir: PathBuf,
    prefix: String,
    buffer: Vec<CdataRecord>,
}

impl CdataExporter {
    pub fn new(output_dir: impl AsRef<Path>, prefix: &str) -> Self {
        let output_dir = output_dir.as_ref().to_path_buf();
        let _ = std::fs::create_dir_all(&output_dir);
        Self {
            output_dir,
            prefix: prefix.to_string(),
            buffer: Vec::new(),
        }
    }

    /// Собрать снимок всех сущностей с `HumanDevelopmentComponent` на данном шаге.
    pub fn collect(&mut self, world: &World, step: u64) {
        for (entity, (comp, myeloid_opt)) in world
            .query::<(&HumanDevelopmentComponent, Option<&MyeloidShiftComponent>)>()
            .iter()
        {
            let dam = &comp.centriolar_damage;
            let appendage_loss = 1.0 - (dam.cep164_integrity
                + dam.cep89_integrity
                + dam.ninein_integrity
                + dam.cep170_integrity) / 4.0;
            let record = CdataRecord {
                step,
                entity_id: entity.to_bits().get(),
                tissue: format!("{:?}", comp.tissue_type),
                age_years: comp.age_years(),
                stage: format!("{:?}", comp.stage),
                damage_score: comp.damage_score(),
                myeloid_bias: myeloid_opt.map_or(0.0, |m| m.myeloid_bias),
                spindle_fidelity: dam.spindle_fidelity,
                ciliary_function: dam.ciliary_function,
                frailty: comp.frailty(),
                phenotype_count: comp.active_phenotypes.len(),
                ptm_carbonylation:    dam.protein_carbonylation,
                ptm_hyperacetylation: dam.tubulin_hyperacetylation,
                ptm_aggregation:      dam.protein_aggregates,
                ptm_phospho_dysreg:   dam.phosphorylation_dysregulation,
                ptm_appendage_loss:   appendage_loss,
            };
            self.buffer.push(record);
        }
    }

    /// Сохранить буфер в CSV-файл и очистить буфер.
    /// Путь: `<output_dir>/<prefix>_cdata_step_<NNNNNN>.csv`
    pub fn save_snapshot(&mut self, step: u64) -> IoResult<PathBuf> {
        let path = self.output_dir.join(format!(
            "{}_cdata_step_{:06}.csv",
            self.prefix, step
        ));
        write_cdata_csv(&path, &self.buffer)?;
        self.buffer.clear();
        Ok(path)
    }

    /// Число записей в буфере (до сохранения)
    pub fn buffered_records(&self) -> usize {
        self.buffer.len()
    }
}

// ---------------------------------------------------------------------------
// P12: реализация трейта CdataCollect для CdataExporter
// ---------------------------------------------------------------------------

impl CdataCollect for CdataExporter {
    fn collect(&mut self, world: &World, step: u64) {
        self.collect(world, step);
    }

    fn write_csv(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        write_cdata_csv(path, &self.buffer)?;
        Ok(())
    }

    fn buffered(&self) -> usize {
        self.buffered_records()
    }
}

/// Записать `CdataRecord`-записи в CSV-файл по указанному пути.
pub fn write_cdata_csv(path: impl AsRef<Path>, records: &[CdataRecord]) -> IoResult<()> {
    let mut wtr = Writer::from_path(path)?;
    wtr.write_record(CdataRecord::csv_headers())?;
    for rec in records {
        wtr.write_record(rec.to_csv_record())?;
    }
    wtr.flush()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// P67: тесты PTM-колонок в CdataRecord
// ---------------------------------------------------------------------------

#[cfg(test)]
mod ptm_tests {
    use super::*;

    fn make_ptm_record(carb: f32, hyper: f32, aggr: f32, phospho: f32, app_loss: f32) -> CdataRecord {
        CdataRecord {
            step: 1,
            entity_id: 1,
            tissue: "Blood".to_string(),
            age_years: 40.0,
            stage: "Adult".to_string(),
            damage_score: 0.3,
            myeloid_bias: 0.2,
            spindle_fidelity: 0.8,
            ciliary_function: 0.9,
            frailty: 0.1,
            phenotype_count: 0,
            ptm_carbonylation: carb,
            ptm_hyperacetylation: hyper,
            ptm_aggregation: aggr,
            ptm_phospho_dysreg: phospho,
            ptm_appendage_loss: app_loss,
        }
    }

    #[test]
    fn test_ptm_csv_headers_include_all_five_columns() {
        let headers = CdataRecord::csv_headers();
        assert!(headers.contains(&"ptm_carbonylation"),    "missing ptm_carbonylation");
        assert!(headers.contains(&"ptm_hyperacetylation"), "missing ptm_hyperacetylation");
        assert!(headers.contains(&"ptm_aggregation"),      "missing ptm_aggregation");
        assert!(headers.contains(&"ptm_phospho_dysreg"),   "missing ptm_phospho_dysreg");
        assert!(headers.contains(&"ptm_appendage_loss"),   "missing ptm_appendage_loss");
        // Ensure the 5 PTM columns are present (16 total: 11 original + 5 new)
        assert_eq!(headers.len(), 16);
    }

    #[test]
    fn test_ptm_appendage_loss_serializes_correctly() {
        let rec = make_ptm_record(0.12, 0.05, 0.08, 0.03, 0.25);
        let row = rec.to_csv_record();
        // Headers order: ..., ptm_carbonylation(11), ptm_hyperacetylation(12),
        //                     ptm_aggregation(13), ptm_phospho_dysreg(14), ptm_appendage_loss(15)
        assert_eq!(row.len(), 16);
        assert!(row[11].starts_with("0.120"), "carbonylation mismatch: {}", row[11]);
        assert!(row[15].starts_with("0.250"), "appendage_loss mismatch: {}", row[15]);
    }
}
