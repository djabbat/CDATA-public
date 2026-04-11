//! CDATA v4.0 — The ¬R Argument: Saturable Centriolar Damage Kinetics
//!
//! Demonstrates two key claims of the ¬R logical argument:
//!
//! 1. INEVITABILITY: For any finite r > 0, D(n) = 1 − e^{−rn} → 1 as n → ∞.
//!    Even under perfect conditions (O₂ → 0, mito_shield → s_max), r never reaches
//!    zero because mito_shield < 1 always. Senescence is mathematically inevitable.
//!
//! 2. O₂-DEPENDENCE: Lowering O₂ reduces r (via mito_shield), extending N_Hayflick,
//!    but cannot abolish the limit. The ¬R argument holds at all O₂ concentrations.
//!
//! Outputs:
//!   - D(n) trajectory for 5 O₂ conditions (0.5%, 1%, 2%, 5%, 21%)
//!   - N_Hayflick (D = 0.99 threshold) for each condition
//!   - Proof that r > 0 for any O₂ > 0 (senescence inevitable)
//!
//! Formula:
//!   r([O₂]) = α·ν·β · (1 − mito_shield([O₂]))
//!   D(n)    = 1 − exp(−r·n)
//!   N_Hayflick = −ln(1 − D_crit_norm) / r  ≈ D_crit_norm / r  (for small D_crit_norm)
//!
//! Usage:
//!   cargo run --example not_r_argument --release

use cell_dt_mitochondrial::{mito_shield_for_o2, predicted_hayflick, CellTypeShield};

// Composite damage rate per division at given O₂ (normalised units)
// r = α·ν·β · (1 − mito_shield)
// Here α·ν·β is absorbed into D_crit/N_Hayflick via the calibrated constants.
// We compute r directly from the N_Hayflick formula: N = D_crit / (α·ν·β · (1−shield))
// → α·ν·β · (1−shield) = D_crit / N  → r_per_div = 1.0 / N_Hayflick (normalised)
fn damage_rate_per_division(o2: f64, cell_type: CellTypeShield) -> f64 {
    let n = predicted_hayflick(o2, cell_type);
    if n.is_infinite() || n < 1.0 { return 0.0; }
    1.0 / n  // normalised: D_crit = 1.0 in this representation
}

// D(n) = 1 − exp(−r·n), clamped to [0, 1]
fn damage_at_n(r: f64, n: f64) -> f64 {
    (1.0 - (-r * n).exp()).clamp(0.0, 1.0)
}

// Number of divisions to reach D = threshold (exact inversion)
fn divisions_to_threshold(r: f64, threshold: f64) -> f64 {
    if r <= 0.0 { return f64::INFINITY; }
    -(1.0 - threshold).ln() / r
}

fn main() {
    const D_CRIT_NORM: f64 = 0.99; // normalised D_crit (= 1.0 in unit scale)
    let cell_type = CellTypeShield::EpithelialProgenitor;

    let o2_conditions: &[(&str, f64)] = &[
        ("  0% O₂ (anoxic limit)", 0.0),
        ("0.5% O₂ (deep hypoxia)", 0.5),
        ("  1% O₂ (physiological)", 1.0),
        ("  2% O₂ (Peters-Hall)", 2.0),
        ("  5% O₂ (mild hypoxia)", 5.0),
        (" 21% O₂ (normoxia)", 21.0),
    ];

    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  CDATA v4.0 — The ¬R Argument: Centriolar Damage is Inevitable");
    println!("  Cell type: EpithelialProgenitor (φ = 1.00, highest protection)");
    println!("  D(n) = 1 − exp(−r·n),  r = α·ν·β·(1 − mito_shield([O₂]))");
    println!("═══════════════════════════════════════════════════════════════════════════════\n");

    // ── Part 1: r and N_Hayflick per O₂ ──────────────────────────────────────
    println!("Part 1: Damage rate r and Hayflick limit per O₂ condition");
    println!("{}", "─".repeat(80));
    println!("{:<28} {:>10} {:>12} {:>14} {:>12}",
        "O₂ condition", "shield", "r (1/div)", "N_Hayflick", "r = 0?");
    println!("{}", "─".repeat(80));

    for (label, o2) in o2_conditions {
        let shield = mito_shield_for_o2(*o2, cell_type);
        let r = damage_rate_per_division(*o2, cell_type);
        let n = predicted_hayflick(*o2, cell_type);
        let r_zero = if r < 1e-10 { "YES ← falsified" } else { "no" };

        let n_str = if n.is_infinite() {
            "         ∞".to_string()
        } else {
            format!("{:>14.1}", n)
        };

        println!("{:<28} {:>10.4} {:>12.6} {} {:>12}",
            label, shield, r, n_str, r_zero);
    }

    println!("\n  KEY: r = 0 would mean no damage accumulation → infinite proliferation.");
    println!("  r > 0 for ALL finite O₂ because mito_shield < 1 always (s_max = 0.99 < 1).");
    println!("  ∴ Senescence is INEVITABLE regardless of O₂. ¬R argument holds.\n");

    // ── Part 2: D(n) trajectory ───────────────────────────────────────────────
    println!("Part 2: D(n) trajectory — normalised centriolar damage vs. divisions");
    println!("{}", "─".repeat(90));

    let selected_o2 = [0.5_f64, 2.0, 5.0, 21.0];
    let division_points = [10u32, 25, 50, 100, 150, 200, 300, 500];

    // Header
    print!("{:>10}", "Divisions");
    for o2 in &selected_o2 {
        print!("  {:>12}", format!("D @ {:.1}%O₂", o2));
    }
    println!();
    println!("{}", "─".repeat(90));

    for &n in &division_points {
        print!("{:>10}", n);
        for &o2 in &selected_o2 {
            let r = damage_rate_per_division(o2, cell_type);
            let d = damage_at_n(r, n as f64);
            let marker = if d >= D_CRIT_NORM { " ← SEN" } else { "" };
            print!("  {:>12}", format!("{:.4}{}", d, marker));
        }
        println!();
    }

    // ── Part 3: N_Hayflick comparison ─────────────────────────────────────────
    println!("\nPart 3: Divisions to D = {:.0}% threshold per O₂", D_CRIT_NORM * 100.0);
    println!("{}", "─".repeat(80));
    println!("{:<28} {:>14} {:>14} {:>20}",
        "O₂ condition", "N_Hayflick", "vs. normoxia", "Note");
    println!("{}", "─".repeat(80));

    let r_normoxia = damage_rate_per_division(21.0, cell_type);
    let n_normoxia = divisions_to_threshold(r_normoxia, D_CRIT_NORM);

    for (label, o2) in o2_conditions {
        let r = damage_rate_per_division(*o2, cell_type);
        let n = divisions_to_threshold(r, D_CRIT_NORM);
        let fold = if n.is_infinite() || n_normoxia <= 0.0 {
            "∞".to_string()
        } else {
            format!("{:.1}×", n / n_normoxia)
        };
        let note = match *o2 as u32 {
            0  => "← theoretical limit (mito_shield = s_max = 0.99)",
            21 => "← Hayflick 1961 calibration (~50 PD)",
            2  => "← Peters-Hall 2020 baseline",
            _  => "",
        };

        let n_str = if n.is_infinite() { ">10000".to_string() } else { format!("{:.1}", n) };
        println!("{:<28} {:>14} {:>14} {:>20}", label, n_str, fold, note);
    }

    println!("\n  CONCLUSION: Even at 0% O₂ (theoretical maximum protection),");
    println!("  N_Hayflick is finite because mito_shield = s_max = {} < 1.", 0.99);
    println!("  The ¬R argument: the centriole is the only structure in ¬R,");
    println!("  therefore its damage accumulation is the only possible autonomous");
    println!("  cell-division clock. Senescence is a logical, not merely empirical, necessity.");

    // ── Part 4: Mathematical proof ─────────────────────────────────────────────
    println!("\n── Mathematical Proof (¬R Argument) ─────────────────────────────────────────");
    println!("  Given: mito_shield([O₂]) = s_max · φ · exp(−[O₂]/O₀)");
    println!("         s_max = 0.99, φ ≤ 1.00, O₀ = 5%");
    println!();
    println!("  For any [O₂] ≥ 0:");
    println!("    mito_shield ≤ s_max · φ = 0.99 × 1.00 = 0.99 < 1");
    println!("  ∴ (1 − mito_shield) ≥ 0.01 > 0");
    println!("  ∴ r = α·ν·β · (1 − mito_shield) > 0");
    println!("  ∴ D(n) = 1 − exp(−r·n) → 1  as n → ∞");
    println!("  ∴ N_Hayflick = −ln(1−D_crit) / r  is finite for any D_crit < 1  ∎");
    println!();
    println!("  Combined with ¬R = {{centriole}}: the Hayflick limit is a mathematical");
    println!("  consequence of centriolar biology, not a technical culture artifact.");
    println!("═══════════════════════════════════════════════════════════════════════════════");
}
