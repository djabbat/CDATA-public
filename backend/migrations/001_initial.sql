-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create custom enum types
CREATE TYPE parameter_status AS ENUM ('estimated', 'measured', 'tbd');
CREATE TYPE scaffold_counter_type AS ENUM ('telomere', 'mito_ros', 'epigenetic_drift', 'proteostasis');
CREATE TYPE milestone_type AS ENUM ('neurological', 'immunological', 'metabolic', 'structural');

-- Parameters table (from PARAMETERS.md)
CREATE TABLE parameters (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    symbol VARCHAR(20) NOT NULL,
    name VARCHAR(100) NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    units VARCHAR(20) NOT NULL,
    source VARCHAR(200) NOT NULL,
    status parameter_status NOT NULL,
    description TEXT,
    gamma_i DOUBLE PRECISION DEFAULT 0.0 CHECK (gamma_i >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(symbol)
);

CREATE INDEX idx_parameters_status ON parameters(status);
CREATE INDEX idx_parameters_gamma_i ON parameters(gamma_i);

-- MCOA Counters registry
CREATE TABLE counters (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    alpha DOUBLE PRECISION NOT NULL CHECK (alpha >= 0),
    beta DOUBLE PRECISION NOT NULL CHECK (beta >= 0),
    gamma_i DOUBLE PRECISION DEFAULT 0.0 CHECK (gamma_i >= 0),
    tissue_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, tissue_type)
);

CREATE INDEX idx_counters_tissue ON counters(tissue_type);
CREATE INDEX idx_counters_gamma ON counters(gamma_i);

-- CDATA-specific counter extension
CREATE TABLE cdata_counters (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    counter_id UUID NOT NULL REFERENCES counters(id) ON DELETE CASCADE,
    hayflick_limit_hypoxia INTEGER NOT NULL CHECK (hayflick_limit_hypoxia > 0),
    d_crit DOUBLE PRECISION NOT NULL CHECK (d_crit > 0),
    rescue_half_life INTEGER NOT NULL CHECK (rescue_half_life > 0),
    inheritance_ratio_hsc DOUBLE PRECISION CHECK (inheritance_ratio_hsc >= 0 AND inheritance_ratio_hsc <= 1),
    asymmetry_index DOUBLE PRECISION CHECK (asymmetry_index >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(counter_id)
);

CREATE INDEX idx_cdata_counter_id ON cdata_counters(counter_id);

-- Tissue types for MCOA
CREATE TABLE tissues (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    weight_hsc DOUBLE PRECISION CHECK (weight_hsc >= 0 AND weight_hsc <= 1),
    transformation_function TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name)
);

-- HSC transplant arm tracking
CREATE TABLE transplant_arms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    generation INTEGER NOT NULL CHECK (generation >= 0),
    division_rate DOUBLE PRECISION NOT NULL CHECK (division_rate >= 0),
    damage_accumulated DOUBLE PRECISION NOT NULL CHECK (damage_accumulated >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_transplant_arms_generation ON transplant_arms(generation);

-- Sobol sensitivity analysis storage
CREATE TABLE sensitivity_analyses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    parameter_id UUID NOT NULL REFERENCES parameters(id) ON DELETE CASCADE,
    sobol_first_order DOUBLE PRECISION NOT NULL CHECK (sobol_first_order >= 0 AND sobol_first_order <= 1),
    sobol_total_order DOUBLE PRECISION NOT NULL CHECK (sobol_total_order >= 0 AND sobol_total_order <= 1),
    confidence_interval_lower DOUBLE PRECISION NOT NULL,
    confidence_interval_upper DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (confidence_interval_lower <= confidence_interval_upper)
);

CREATE INDEX idx_sensitivity_parameter ON sensitivity_analyses(parameter_id);

-- MCOA L_tissue computation results
CREATE TABLE mcoa_computations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tissue_id UUID NOT NULL REFERENCES tissues(id) ON DELETE CASCADE,
    l_tissue DOUBLE PRECISION NOT NULL CHECK (l_tissue >= 0),
    computation_time_ms BIGINT NOT NULL CHECK (computation_time_ms > 0),
    parameters_used JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_mcoa_tissue ON mcoa_computations(tissue_id);
CREATE INDEX idx_mcoa_created ON mcoa_computations(created_at);

-- FCLC privacy budget tracking
CREATE TABLE fclc_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    epsilon DOUBLE PRECISION NOT NULL CHECK (epsilon >= 0),
    delta DOUBLE PRECISION NOT NULL CHECK (delta >= 0 AND delta <= 1),
    secure_aggregation_result JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_fclc_epsilon ON fclc_data(epsilon);

-- BioSense raw EEG/HRV data (NO χ_Ze computation on server)
CREATE TABLE biosense_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    eeg_raw JSONB NOT NULL,
    hrv_raw JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_biosense_timestamp ON biosense_data(timestamp);

-- Scaffold counters time-series
CREATE TABLE scaffold_counters (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    counter_type scaffold_counter_type NOT NULL,
    d_i DOUBLE PRECISION NOT NULL CHECK (d_i >= 0),
    timestamp TIMESTAMPTZ NOT NULL,
    parameters JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_scaffold_type ON scaffold_counters(counter_type);
CREATE INDEX idx_scaffold_timestamp ON scaffold_counters(timestamp);

-- HAP hepatic+affective joint biomarkers
CREATE TABLE hap_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    hepatic_biomarker DOUBLE PRECISION NOT NULL,
    affective_biomarker DOUBLE PRECISION NOT NULL,
    joint_score DOUBLE PRECISION,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_hap_timestamp ON hap_data(timestamp);

-- Ontogenesis 0-25 year milestones
CREATE TABLE ontogenesis_milestones (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    age_years DOUBLE PRECISION NOT NULL CHECK (age_years >= 0 AND age_years <= 25),
    milestone_type milestone_type NOT NULL,
    description TEXT NOT NULL,
    is_critical BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_milestone_age ON ontogenesis_milestones(age_years);
CREATE INDEX idx_milestone_type ON ontogenesis_milestones(milestone_type);
CREATE INDEX idx_milestone_critical ON ontogenesis_milestones(is_critical);

-- Create update timestamp triggers
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply triggers to all tables with updated_at
CREATE TRIGGER update_parameters_updated_at BEFORE UPDATE ON parameters
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_counters_updated_at BEFORE UPDATE ON counters
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_cdata_counters_updated_at BEFORE UPDATE ON cdata_counters
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tissues_updated_at BEFORE UPDATE ON tissues
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_transplant_arms_updated_at BEFORE UPDATE ON transplant_arms
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sensitivity_analyses_updated_at BEFORE UPDATE ON sensitivity_analyses
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_fclc_data_updated_at BEFORE UPDATE ON fclc_data
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ontogenesis_milestones_updated_at BEFORE UPDATE ON ontogenesis_milestones
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();