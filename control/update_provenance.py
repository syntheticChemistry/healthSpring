#!/usr/bin/env python3
"""Update baseline JSON files with provenance metadata."""

import json
import sys

GIT_COMMIT = "4138375e3973a6a95d25758ccfc5436b5e8d0ee1"

FILES = [
    ("control/pkpd/exp001_baseline.json", "python3 control/pkpd/exp001_hill_dose_response.py"),
    ("control/pkpd/exp002_baseline.json", "python3 control/pkpd/exp002_one_compartment_pk.py"),
    ("control/pkpd/exp003_baseline.json", "python3 control/pkpd/exp003_two_compartment_pk.py"),
    ("control/pkpd/exp004_baseline.json", "python3 control/pkpd/exp004_mab_pk_transfer.py"),
    ("control/pkpd/exp005_baseline.json", "python3 control/pkpd/exp005_population_pk.py"),
    ("control/pkpd/exp006_baseline.json", "python3 control/pkpd/exp006_pbpk_compartments.py"),
    ("control/microbiome/exp010_baseline.json", "python3 control/microbiome/exp010_diversity_indices.py"),
    ("control/microbiome/exp011_baseline.json", "python3 control/microbiome/exp011_anderson_gut_lattice.py"),
    ("control/microbiome/exp012_baseline.json", "python3 control/microbiome/exp012_cdiff_resistance.py"),
    ("control/microbiome/exp013_baseline.json", "python3 control/microbiome/exp013_fmt_rcdi.py"),
    ("control/biosignal/exp020_baseline.json", "python3 control/biosignal/exp020_pan_tompkins_qrs.py"),
    ("control/biosignal/exp021_baseline.json", "python3 control/biosignal/exp021_hrv_metrics.py"),
    ("control/endocrine/exp030_baseline.json", "python3 control/endocrine/exp030_testosterone_im_pk.py"),
    ("control/endocrine/exp031_baseline.json", "python3 control/endocrine/exp031_testosterone_pellet_pk.py"),
    ("control/endocrine/exp032_baseline.json", "python3 control/endocrine/exp032_age_testosterone_decline.py"),
    ("control/endocrine/exp033_baseline.json", "python3 control/endocrine/exp033_trt_weight_trajectory.py"),
    ("control/endocrine/exp034_baseline.json", "python3 control/endocrine/exp034_trt_cardiovascular.py"),
    ("control/endocrine/exp035_baseline.json", "python3 control/endocrine/exp035_trt_diabetes.py"),
    ("control/endocrine/exp036_baseline.json", "python3 control/endocrine/exp036_population_trt_montecarlo.py"),
    ("control/endocrine/exp037_baseline.json", "python3 control/endocrine/exp037_testosterone_gut_axis.py"),
    ("control/endocrine/exp038_baseline.json", "python3 control/endocrine/exp038_hrv_trt_cardiovascular.py"),
]


def main():
    repo_root = "/home/eastgate/Development/ecoPrimals/healthSpring"
    errors = []

    for json_path, command in FILES:
        full_path = f"{repo_root}/{json_path}"
        script = command.replace("python3 ", "")

        try:
            with open(full_path, "r") as f:
                data = json.load(f)

            if "_provenance" not in data:
                errors.append(f"{json_path}: missing _provenance")
                continue

            data["_provenance"]["git_commit"] = GIT_COMMIT
            data["_provenance"]["command"] = command
            data["_provenance"]["script"] = script

            with open(full_path, "w") as f:
                json.dump(data, f, indent=2)
                f.write("\n")

            print(f"Updated {json_path}")
        except Exception as e:
            errors.append(f"{json_path}: {e}")

    if errors:
        for err in errors:
            print(f"ERROR: {err}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
