//! O₂ dose-response simulation: N_Hayflick as a function of ambient O₂.
//!
//! Reproduces Figure 1B from Tkemaladze (2026) "Centriolar Damage Accumulation Drives
//! Replicative Senescence" (target: Aging Cell). Validates CDATA v3.4 Group 8 parameters
//! against Peters-Hall et al. (2020, FASEB J, DOI: 10.1096/fj.201901415R).
//!
//! Usage:
//!   cargo run --example o2_dose_response --release
//!
//! Expected output (CDATA v3.4 predictions):
//!   [0.5%  O₂] EpithelialProgenitor: N_Hayflick ≈ 526
//!   [2.0%  O₂] EpithelialProgenitor: N_Hayflick ≈ 148  (Peters-Hall: >200 + ROCKi)
//!   [5.0%  O₂] HematopoieticStem:    N_Hayflick ≈ 101  (HSC niche)
//!   [21.0% O₂] Fibroblast:           N_Hayflick ≈ 50   (Hayflick 1961)

use cell_dt_mitochondrial::{mito_shield_for_o2, predicted_hayflick, CellTypeShield};

fn main() {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  CDATA v3.4 — O₂ Dose-Response: Predicted Hayflick Limit");
    println!("  Formula: N = D_crit / (α·ν·β × (1 − mito_shield([O₂])))");
    println!("  D_crit = 1000 a.u. | α·ν·β = 20 a.u./div");
    println!("  Ref: Tkemaladze 2026 (Aging Cell); Peters-Hall 2020 (FASEB J)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let o2_levels: &[f64] = &[0.5, 1.0, 2.0, 3.0, 5.0, 10.0, 21.0];

    let cell_types = [
        ("EpithelialProgenitor", CellTypeShield::EpithelialProgenitor),
        ("HematopoieticStem   ", CellTypeShield::HematopoieticStem),
        ("Fibroblast          ", CellTypeShield::Fibroblast),
    ];

    for (type_name, cell_type) in &cell_types {
        println!("── {} ──────────────────────────────────────────────", type_name.trim());
        println!("  {:>8}  {:>12}  {:>16}  {}", "[O₂] %", "mito_shield", "N_Hayflick", "Note");
        println!("  {:>8}  {:>12}  {:>16}  {}", "-------", "-----------", "----------", "----");

        for &o2 in o2_levels {
            let shield = mito_shield_for_o2(o2, *cell_type);
            let n_hay  = predicted_hayflick(o2, *cell_type);

            let note = match (o2, cell_type) {
                (o, CellTypeShield::Fibroblast) if (o - 21.0).abs() < 0.1
                    => "← Hayflick 1961 calibration target (~50)",
                (o, CellTypeShield::EpithelialProgenitor) if (o - 2.0).abs() < 0.1
                    => "← Peters-Hall 2020: >200 PD (+ ROCKi)",
                (o, CellTypeShield::HematopoieticStem) if o <= 3.0
                    => "← HSC bone marrow niche (Simsek 2010)",
                _ => "",
            };

            let n_display = if n_hay.is_infinite() {
                "  ∞ (denom→0)".to_string()
            } else if n_hay > 9999.0 {
                format!("{:>16.0}", n_hay)
            } else {
                format!("{:>16.1}", n_hay)
            };

            println!("  {:>8.1}  {:>12.4}  {}  {}", o2, shield, n_display, note);
        }
        println!();
    }

    // ── Experimental validation summary ──────────────────────────────────────
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Validation against published benchmarks:");
    println!("═══════════════════════════════════════════════════════════════════");

    struct Benchmark {
        citation: &'static str,
        o2: f64,
        cell_type: CellTypeShield,
        observed_min: f64,
        observed_max: f64,
        note: &'static str,
    }

    let benchmarks = [
        Benchmark {
            citation: "Hayflick & Moorhead (1961) PMID: 13905658",
            o2: 21.0,
            cell_type: CellTypeShield::Fibroblast,
            observed_min: 42.0,
            observed_max: 58.0,
            note: "WI-38 human fibroblasts at normoxia",
        },
        Benchmark {
            citation: "Peters-Hall et al. (2020) DOI: 10.1096/fj.201901415R",
            o2: 2.0,
            cell_type: CellTypeShield::EpithelialProgenitor,
            observed_min: 200.0,
            observed_max: f64::INFINITY,
            note: "HBECs at 2% O₂ + ROCKi (Y-27632) + feeder-free",
        },
    ];

    for b in &benchmarks {
        let predicted = predicted_hayflick(b.o2, b.cell_type);
        let within = if b.observed_max.is_infinite() {
            predicted >= b.observed_min
        } else {
            predicted >= b.observed_min && predicted <= b.observed_max
        };

        let obs_str = if b.observed_max.is_infinite() {
            format!(">{:.0}", b.observed_min)
        } else {
            format!("{:.0}–{:.0}", b.observed_min, b.observed_max)
        };

        println!("\n  Reference : {}", b.citation);
        println!("  Conditions: O₂ = {:.1}%  |  {}", b.o2, b.note);
        println!("  Observed  : {} PD", obs_str);
        println!("  Predicted : {:.1} PD", predicted);
        println!("  Status    : {}", if within { "✅ Within range" } else { "⚠️  Outside range — see §5.1 (ROCKi hypothesis)" });
    }

    println!("\n  Note: Peters-Hall benchmark uses ROCKi + feeder-free conditions.");
    println!("  CDATA v3.4 models O₂ effect only; ROCKi adds synergistic protection.");
    println!("  Full reconciliation: Tkemaladze (2026) §5.1 Hypotheses 1–3.");
    println!("═══════════════════════════════════════════════════════════════════");
}
