#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

SUCCEEDED=0
FAILED=0

run_script() {
    local script="$1"
    if python3 "$script"; then
        SUCCEEDED=$((SUCCEEDED + 1))
        return 0
    else
        FAILED=$((FAILED + 1))
        return 1
    fi
}

echo "=============================================="
echo "healthSpring: Regenerating All Python Baselines"
echo "=============================================="

# --- pkpd ---
echo ""
echo "--- pkpd ---"
for py in control/pkpd/exp001_hill_dose_response.py \
         control/pkpd/exp002_one_compartment_pk.py \
         control/pkpd/exp003_two_compartment_pk.py \
         control/pkpd/exp004_mab_pk_transfer.py \
         control/pkpd/exp005_population_pk.py \
         control/pkpd/exp006_pbpk_compartments.py \
         control/pkpd/exp077_michaelis_menten_pk.py; do
    run_script "$py" || true
done

# --- microbiome ---
echo ""
echo "--- microbiome ---"
for py in control/microbiome/exp010_diversity_indices.py \
         control/microbiome/exp011_anderson_gut_lattice.py \
         control/microbiome/exp012_cdiff_resistance.py \
         control/microbiome/exp013_fmt_rcdi.py \
         control/microbiome/exp078_antibiotic_perturbation.py \
         control/microbiome/exp079_scfa_production.py \
         control/microbiome/exp080_gut_brain_serotonin.py; do
    run_script "$py" || true
done

# --- biosignal ---
echo ""
echo "--- biosignal ---"
for py in control/biosignal/exp020_pan_tompkins_qrs.py \
         control/biosignal/exp021_hrv_metrics.py \
         control/biosignal/exp022_ppg_spo2.py \
         control/biosignal/exp023_fusion.py \
         control/biosignal/exp081_eda_stress.py \
         control/biosignal/exp082_arrhythmia_classification.py; do
    run_script "$py" || true
done

# --- endocrine ---
echo ""
echo "--- endocrine ---"
for py in control/endocrine/exp030_testosterone_im_pk.py \
         control/endocrine/exp031_testosterone_pellet_pk.py \
         control/endocrine/exp032_age_testosterone_decline.py \
         control/endocrine/exp033_trt_weight_trajectory.py \
         control/endocrine/exp034_trt_cardiovascular.py \
         control/endocrine/exp035_trt_diabetes.py \
         control/endocrine/exp036_population_trt_montecarlo.py \
         control/endocrine/exp037_testosterone_gut_axis.py \
         control/endocrine/exp038_hrv_trt_cardiovascular.py; do
    run_script "$py" || true
done

# --- validation ---
echo ""
echo "--- validation ---"
run_script "control/validation/exp040_barracuda_cpu.py" || true

# --- update provenance ---
echo ""
echo "--- update provenance ---"
if python3 control/update_provenance.py; then
    SUCCEEDED=$((SUCCEEDED + 1))
else
    FAILED=$((FAILED + 1))
fi

# --- summary ---
TOTAL=$((SUCCEEDED + FAILED))
echo ""
echo "=============================================="
echo "SUMMARY: $SUCCEEDED succeeded, $FAILED failed (of $TOTAL scripts)"
echo "=============================================="

[ "$FAILED" -eq 0 ]
