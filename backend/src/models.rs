use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;

// Parameter entity from PARAMETERS.md
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Parameter {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub value: f64,
    pub units: String,
    pub source: String,
    pub status: ParameterStatus,
    pub description: Option<String>,
    #[serde(default = "default_gamma_i")]
    pub gamma_i: f64, // Default 0 per CORRECTIONS_2026-04-22
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_gamma_i() -> f64 {
    0.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ParameterCreate {
    #[validate(length(min = 1, max = 20))]
    pub symbol: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub value: f64,
    #[validate(length(min = 1, max = 20))]
    pub units: String,
    #[validate(length(min = 1, max = 200))]
    pub source: String,
    pub status: ParameterStatus,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    pub gamma_i: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ParameterUpdate {
    #[validate(length(min = 1, max = 20))]
    pub symbol: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub value: Option<f64>,
    #[validate(length(min = 1, max = 20))]
    pub units: Option<String>,
    #[validate(length(min = 1, max = 200))]
    pub source: Option<String>,
    pub status: Option<ParameterStatus>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    pub gamma_i: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "parameter_status", rename_all = "snake_case")]
pub enum ParameterStatus {
    Estimated,
    Measured,
    Tbd,
}

// MCOA Counter registry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Counter {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub alpha: f64, // α_i - division-dependent kinetics
    pub beta: f64,  // β_i - time-dependent kinetics
    #[serde(default = "default_gamma_i")]
    pub gamma_i: f64, // Default 0 per CORRECTIONS_2026-04-22
    pub tissue_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CounterCreate {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 500))]
    pub description: String,
    pub alpha: f64,
    pub beta: f64,
    pub gamma_i: Option<f64>,
    #[validate(length(min = 1, max = 50))]
    pub tissue_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CounterUpdate {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 500))]
    pub description: Option<String>,
    pub alpha: Option<f64>,
    pub beta: Option<f64>,
    pub gamma_i: Option<f64>,
    #[validate(length(min = 1, max = 50))]
    pub tissue_type: Option<String>,
}

// CDATA-specific counter extension
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CdataCounter {
    pub id: Uuid,
    pub counter_id: Uuid,
    pub hayflick_limit_hypoxia: i32,
    pub d_crit: f64,
    pub rescue_half_life: i32,
    pub inheritance_ratio_hsc: Option<f64>,
    pub asymmetry_index: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CdataCounterCreate {
    pub counter_id: Uuid,
    #[validate(range(min = 1, max = 1000))]
    pub hayflick_limit_hypoxia: i32,
    pub d_crit: f64,
    #[validate(range(min = 1, max = 200))]
    pub rescue_half_life: i32,
    #[validate(range(min = 0.0, max = 1.0))]
    pub inheritance_ratio_hsc: Option<f64>,
    #[validate(range(min = 0.0))]
    pub asymmetry_index: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CdataCounterUpdate {
    pub counter_id: Option<Uuid>,
    #[validate(range(min = 1, max = 1000))]
    pub hayflick_limit_hypoxia: Option<i32>,
    pub d_crit: Option<f64>,
    #[validate(range(min = 1, max = 200))]
    pub rescue_half_life: Option<i32>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub inheritance_ratio_hsc: Option<f64>,
    #[validate(range(min = 0.0))]
    pub asymmetry_index: Option<f64>,
}

// Tissue types for MCOA
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Tissue {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub weight_hsc: Option<f64>, // w_HSC(tissue)
    pub transformation_function: Option<String>, // f_HSC(D)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TissueCreate {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 500))]
    pub description: String,
    #[validate(range(min = 0.0, max = 1.0))]
    pub weight_hsc: Option<f64>,
    pub transformation_function: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TissueUpdate {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 500))]
    pub description: Option<String>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub weight_hsc: Option<f64>,
    pub transformation_function: Option<String>,
}

// HSC transplant arm tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TransplantArm {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub generation: i32,
    pub division_rate: f64,
    pub damage_accumulated: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Sobol sensitivity analysis storage
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SensitivityAnalysis {
    pub id: Uuid,
    pub parameter_id: Uuid,
    pub sobol_first_order: f64,
    pub sobol_total_order: f64,
    pub confidence_interval_lower: f64,
    pub confidence_interval_upper: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// MCOA L_tissue computation results
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct McoaComputation {
    pub id: Uuid,
    pub tissue_id: Uuid,
    pub l_tissue: f64,
    pub computation_time_ms: i64,
    pub parameters_used: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// FCLC privacy budget tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FclcData {
    pub id: Uuid,
    pub epsilon: f64, // Privacy budget ε
    pub delta: f64,
    pub secure_aggregation_result: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// BioSense raw EEG/HRV data (NO χ_Ze computation on server)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BiosenseData {
    pub id: Uuid,
    pub eeg_raw: serde_json::Value,
    pub hrv_raw: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// Scaffold counters time-series
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct ScaffoldCounter {
    pub id: Uuid,
    pub counter_type: ScaffoldCounterType,
    pub d_i: f64, // D_i damage accumulation
    pub timestamp: DateTime<Utc>,
    pub parameters: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "scaffold_counter_type", rename_all = "snake_case")]
pub enum ScaffoldCounterType {
    Telomere,
    MitoRos,
    EpigeneticDrift,
    Proteostasis,
}

// HAP hepatic+affective joint biomarkers
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HapData {
    pub id: Uuid,
    pub hepatic_biomarker: f64,
    pub affective_biomarker: f64,
    pub joint_score: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// Ontogenesis 0-25 year milestones
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct OntogenesisMilestone {
    pub id: Uuid,
    pub age_years: f64,
    pub milestone_type: MilestoneType,
    pub description: String,
    pub is_critical: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "milestone_type", rename_all = "snake_case")]
pub enum MilestoneType {
    Neurological,
    Immunological,
    Metabolic,
    Structural,
}