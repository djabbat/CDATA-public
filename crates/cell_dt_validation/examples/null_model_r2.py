#!/usr/bin/env python3
"""
CDATA Null Model R² Comparison (AC-B1)
=======================================
Computes R²_null (linear age model) for comparison against R²_CDATA=0.84.

Without a null model baseline, R²=0.84 is not interpretable for Aging Cell.
This script:
  1. Generates cross-sectional MCAI vs age data calibrated to published values
     (R²_CDATA=0.84, RMSE=0.07, mean age 45±20 years)
  2. Fits a linear age model (null) → R²_null
  3. Computes ΔR² = R²_CDATA − R²_null
  4. Reports statistical significance of improvement (F-test)

Usage:
    python3 null_model_r2.py [--n 500] [--seed 42]

Note: Currently uses synthetic data calibrated to CDATA parameters.
Replace simulated_data() with real NHANES / cohort data for publication.

Author: Tkemaladze / CDATA v4.1 — 2026-04-11
Reference: CDATA/CONCEPT.md §Validation, AC-B1 fix
"""

import numpy as np
import argparse
from scipy import stats


def simulate_cohort(n: int, seed: int = 42) -> tuple:
    """
    Simulate cross-sectional cohort calibrated to CDATA validation data.

    CDATA prediction: MCAI = f(D_accumulated) where D depends on age
    nonlinearly via D(t) = D₀·exp(α·ν·t) — exponential damage accumulation.

    Linear age model (null): MCAI = a·age + b

    Calibration targets:
    - R²_CDATA ≈ 0.84 (from CONCEPT.md §Validation)
    - RMSE_CDATA ≈ 0.07
    - Age range: 20–85 years
    - MCAI range: 0.0–1.0 (normalized)
    """
    rng = np.random.default_rng(seed)
    age = rng.uniform(20, 85, n)

    # True relationship: MCAI ∝ D(t) = exp(α·ν·t) normalized
    # α=0.0082, ν=0.01/day → α·ν = 8.2e-5/day = 0.030/year
    k_year = 0.030
    D_norm = (np.exp(k_year * (age - 20)) - 1) / (np.exp(k_year * 65) - 1)

    # MCAI_true = 0.05 + 0.85·D_norm  (baseline 0.05, max 0.90)
    mcai_true = 0.05 + 0.85 * D_norm

    # Add residual noise calibrated to RMSE=0.07
    noise = rng.normal(0, 0.07, n)
    mcai_obs = np.clip(mcai_true + noise, 0.0, 1.0)

    return age, mcai_obs, mcai_true


def r2_score(y_true: np.ndarray, y_pred: np.ndarray) -> float:
    ss_res = np.sum((y_true - y_pred) ** 2)
    ss_tot = np.sum((y_true - np.mean(y_true)) ** 2)
    return float(1.0 - ss_res / ss_tot) if ss_tot > 0 else 0.0


def main():
    parser = argparse.ArgumentParser(description="CDATA null model R² comparison")
    parser.add_argument("--n", type=int, default=500, help="Cohort size")
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    print("=" * 60)
    print("  CDATA Null Model R² Comparison (AC-B1)")
    print("=" * 60)
    print(f"  N = {args.n} subjects (SIMULATED — calibrated to CDATA params)")
    print()

    age, mcai_obs, mcai_true = simulate_cohort(args.n, args.seed)

    # ── NULL MODEL: Linear age → MCAI ─────────────────────────────
    slope_null, intercept_null, r_null, p_null, se_null = stats.linregress(age, mcai_obs)
    mcai_pred_null = slope_null * age + intercept_null
    r2_null = r2_score(mcai_obs, mcai_pred_null)
    rmse_null = float(np.sqrt(np.mean((mcai_obs - mcai_pred_null) ** 2)))

    print("  NULL MODEL: MCAI ~ β₀ + β₁·age (linear)")
    print(f"    β₀ = {intercept_null:.4f},  β₁ = {slope_null:.5f}/year")
    print(f"    R²_null  = {r2_null:.4f}")
    print(f"    RMSE_null = {rmse_null:.4f}")
    print(f"    Pearson r = {r_null:.4f},  p = {p_null:.2e}")
    print()

    # ── CDATA MODEL: Exponential damage accumulation ──────────────
    # Fit D_norm = (exp(k·(age−20)) − 1) / (exp(k·65) − 1)
    # k is the free parameter; estimate from data via linearisation
    # ln(D_norm + ε) ~ k·age  (approximate linearisation)
    k_fit = 0.030  # CDATA parameter: α·ν in 1/year = 0.030

    D_norm_pred = (np.exp(k_fit * (age - 20)) - 1) / (np.exp(k_fit * 65) - 1)
    slope_cdata, intercept_cdata, r_cdata, p_cdata, se_cdata = stats.linregress(
        D_norm_pred, mcai_obs
    )
    mcai_pred_cdata = slope_cdata * D_norm_pred + intercept_cdata
    r2_cdata = r2_score(mcai_obs, mcai_pred_cdata)
    rmse_cdata = float(np.sqrt(np.mean((mcai_obs - mcai_pred_cdata) ** 2)))

    print("  CDATA MODEL: MCAI ~ β₀ + β₁·D_norm(age),  D=exp(k·Δage)−1")
    print(f"    k = {k_fit:.4f}/year  (α·ν from CDATA parameters)")
    print(f"    R²_CDATA  = {r2_cdata:.4f}  (target: 0.84)")
    print(f"    RMSE_CDATA = {rmse_cdata:.4f}  (target: 0.07)")
    print()

    # ── COMPARISON ────────────────────────────────────────────────
    delta_r2 = r2_cdata - r2_null
    print("  COMPARISON")
    print(f"    ΔR² = R²_CDATA − R²_null = {r2_cdata:.4f} − {r2_null:.4f} = {delta_r2:+.4f}")
    print()

    # F-test for nested model improvement (null ⊂ CDATA if same df)
    # Both models have p=2 parameters → not nested in standard sense
    # Use F-test: F = ((SS_res_null - SS_res_cdata)/Δdf) / (SS_res_cdata/(n-p_cdata))
    n = args.n
    p_null_params = 2   # intercept + slope
    p_cdata_params = 2  # intercept + β₁·D_norm (k is fixed, not estimated)
    # Since both have 2 params, compare directly via likelihood ratio
    ss_null = float(np.sum((mcai_obs - mcai_pred_null) ** 2))
    ss_cdata = float(np.sum((mcai_obs - mcai_pred_cdata) ** 2))
    f_stat = ((ss_null - ss_cdata) / 1) / (ss_cdata / (n - p_cdata_params))
    p_f = float(1.0 - stats.f.cdf(f_stat, 1, n - p_cdata_params))

    print(f"    F-test (1, {n-p_cdata_params}): F = {f_stat:.2f},  p = {p_f:.4e}")
    print()

    # ── INTERPRETATION ───────────────────────────────────────────
    print("  INTERPRETATION FOR Aging Cell §Results:")
    if delta_r2 > 0.10:
        strength = "substantial"
        verdict = "The nonlinear CDATA model explains substantially more variance than a linear age proxy."
    elif delta_r2 > 0.03:
        strength = "modest"
        verdict = "CDATA modestly outperforms a linear age model — specify mechanistic contribution."
    elif delta_r2 > 0:
        strength = "marginal"
        verdict = "Marginal improvement over linear age — mechanistic justification is the key claim."
    else:
        strength = "none/negative"
        verdict = "⚠️  CDATA does not outperform linear age model — review model specification."

    print(f"    ΔR² = {delta_r2:+.4f} ({strength})")
    print(f"    → {verdict}")
    print()

    # ── REPORTING TEMPLATE ────────────────────────────────────────
    print("  PAPER TEMPLATE (Aging Cell Results §Validation):")
    print(f"""
    "A linear age model (null) explained {r2_null:.2f} (95% CI: [TBD]) of
    variance in MCAI. The nonlinear CDATA model (D(t) = exp(k·Δage) − 1,
    k = {k_fit:.3f}/year) explained {r2_cdata:.2f} of variance
    (ΔR² = {delta_r2:+.4f}, F(1,{n-2}) = {f_stat:.1f}, p = {p_f:.2e}).
    RMSE was reduced from {rmse_null:.3f} (null) to {rmse_cdata:.3f} (CDATA).
    Note: all comparisons are cross-sectional; longitudinal validation
    is a primary objective of the proposed WP3 (UK Biobank, N≥5000)."
    """)

    print("=" * 60)
    print("  ⚠️  SIMULATED DATA — replace with real NHANES/cohort data")
    print("      before including in Aging Cell submission.")
    print("=" * 60)


if __name__ == "__main__":
    main()
