use pyo3::prelude::*;
use cell_dt_core::FixedParameters;

#[pyfunction]
fn youth_protection(age_years: f64) -> f64 {
    FixedParameters::default().youth_protection(age_years)
}

#[pyfunction]
fn inheritance_probability(centriole_damage: f64, spindle_fidelity: f64) -> f64 {
    FixedParameters::default().inheritance_probability(centriole_damage, spindle_fidelity)
}

#[pyfunction]
fn sasp_hormetic_response(sasp: f64) -> f64 {
    FixedParameters::default().sasp_hormetic_response(sasp)
}

#[pymodule]
fn cell_dt_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(youth_protection, m)?)?;
    m.add_function(wrap_pyfunction!(inheritance_probability, m)?)?;
    m.add_function(wrap_pyfunction!(sasp_hormetic_response, m)?)?;
    Ok(())
}
