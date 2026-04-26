//! Minimal CDATA CLI wrapper producing MCOA-compliant trajectory CSV.
//!
//! Parameters calibrated from CDATA meta-analysis (Asymmetric centriole
//! inheritance + polyglutamylation aging):
//!
//! - α = 0.60 — centriolar polyGlu per division (division-dominant with t component)
//! - β = 0.15 — G0 polyGlu accumulation
//! - n_star = 50 divisions (fibroblast Hayflick equivalent)
//! - τ = 30 years (centriolar tubulin half-life)
//! - d_critical = 0.65

use serde::{Deserialize, Serialize};

pub const COUNTER_NUMBER: u8 = 1;
pub const COUNTER_NAME: &str = "CDATA (Centriolar polyglutamylation)";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CounterParams {
    pub d0: f64,
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    pub n_star: f64,
    pub tau_days: f64,
    pub d_critical: f64,
}

impl Default for CounterParams {
    fn default() -> Self {
        Self {
            d0: 0.0,
            alpha: 0.60,
            beta:  0.15,
            gamma: 0.0,
            n_star: 50.0,
            tau_days: 10950.0,  // 30 years
            d_critical: 0.65,
        }
    }
}

pub fn compute_damage(p: &CounterParams, n: f64, t_days: f64, coupling: f64) -> f64 {
    p.d0
        + p.alpha * (n / p.n_star)
        + p.beta  * (t_days / p.tau_days)
        + p.gamma * coupling
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tissue {
    HSC, Fibroblast, Neuron, Cardiomyocyte, Hepatocyte, IntestinalCrypt,
}

impl Tissue {
    pub fn params(self) -> CounterParams {
        let mut p = CounterParams::default();
        match self {
            Tissue::Neuron | Tissue::Cardiomyocyte => {
                p.alpha *= 0.05;   // near-post-mitotic: α nearly off
                p.beta  *= 1.5;    // time-dominant polyGlu accumulates
            }
            Tissue::IntestinalCrypt => {
                p.alpha *= 1.5;    // high turnover
                p.beta  *= 0.8;
            }
            Tissue::HSC => {
                p.alpha *= 1.2;    // stem cell-specific boost per Ugale 2024
            }
            _ => {}
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params_nonzero() {
        let p = CounterParams::default();
        assert!(p.alpha > 0.0);
        assert!(p.beta > 0.0);
    }

    #[test]
    fn damage_monotone_in_n() {
        let p = CounterParams::default();
        let a = compute_damage(&p, 10.0, 0.0, 0.0);
        let b = compute_damage(&p, 50.0, 0.0, 0.0);
        assert!(b > a);
    }

    #[test]
    fn tissue_panel_valid() {
        for t in [Tissue::HSC, Tissue::Fibroblast, Tissue::Neuron,
                  Tissue::Cardiomyocyte, Tissue::Hepatocyte, Tissue::IntestinalCrypt] {
            let p = t.params();
            assert!(p.alpha >= 0.0);
            assert!(p.beta  >= 0.0);
        }
    }
}
