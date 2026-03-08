# healthSpring Paper Review Queue

**Last Updated**: March 8, 2026
**Status**: 17 experiments complete — 192 Python checks, 179 Rust binary checks, 103 Rust unit tests

---

## Queue Summary

| Track | Papers Queued | Started | Complete |
|-------|:------------:|:-------:|:--------:|
| Track 1: PK/PD | 7 | 5 | 5 |
| Track 2: Microbiome | 7 | 3 | 3 |
| Track 3: Biosignal | 6 | 1 | 1 |
| Track 4: Endocrinology | 8 | 8 | 8 |
| **Total** | **28** | **17** | **17** |

---

## Completed

| ID | Paper | Experiment | Python | Rust Binary | Tier |
|----|-------|-----------|:------:|:-----------:|:----:|
| PK-007 | Gonzales oclacitinib → human JAK inhibitor PK | Exp001 | 19 | 18 | 0,1 |
| PK-001 | Rowland & Tozer Ch. 3 — one-compartment PK | Exp002 | 12 | 18 | 0,1 |
| PK-001 | Rowland & Tozer Ch. 19 — two-compartment PK | Exp003 | 15 | 11 | 0,1 |
| PK-006 | Gonzales lokivetmab → human mAb PK | Exp004 | 12 | 7 | 0,1 |
| PK-002 | Mould & Upton 2013 — Population PK Monte Carlo | Exp005 | 15 | 12 | 0,1 |
| MB-007 | wetSpring 16S → diversity indices | Exp010 | 14 | 12 | 0,1 |
| MB-006 | wetSpring Anderson soil → gut colonization | Exp011 | 12 | 14 | 0,1 |
| MB-001 | Jenior et al. C. diff metabolic modeling | Exp012 | 10 | 10 | 0,1 |
| BS-001 | Pan & Tompkins QRS detection | Exp020 | 12 | 12 | 0,1 |
| EN-001 | Mok Ch.11 + Shoskes 2016 — IM testosterone PK | Exp030 | 12 | 11 | 0,1 |
| EN-002 | Testopel label + Cavender 2009 — Pellet PK | Exp031 | 10 | 10 | 0,1 |
| EN-003 | Harman 2001 (BLSA) — Age testosterone decline | Exp032 | 10 | 8 | 0,1 |
| EN-004 | Saad 2013 registry — TRT weight trajectory | Exp033 | 10 | 7 | 0,1 |
| EN-005 | Saad 2016 + Sharma 2015 — TRT cardiovascular | Exp034 | 10 | 10 | 0,1 |
| EN-006 | Kapoor 2006 + Dhindsa 2016 — TRT diabetes | Exp035 | 10 | 10 | 0,1 |
| EN-007 | Composite registries — Population TRT Monte Carlo | Exp036 | 12 | 10 | 0,1 |
| EN-008 | Cross-track D1/D2 — Testosterone-gut axis | Exp037 | 12 | 10 | 0,1 |

---

## Next Papers (Remaining Queue)

1. **PK-004**: Gabrielsson & Weiner PBPK compartments
2. **MB-002**: McGill synthetic microbiota for rCDI
3. **BS-002**: MIT-BIH HRV metrics (SDNN, RMSSD, pNN50)
4. **BS-003**: Wearable PPG → SpO2 via R-value calibration
5. **BS-004**: Multi-channel biosignal fusion (ECG + PPG + EDA)

---

## Open Data Audit

All experiments use publicly accessible data. No proprietary dependencies.

| Experiment | Open Data Source | Access Method |
|-----------|-----------------|---------------|
| Exp001 | Published IC50 values (Gonzales 2014, FDA labels) | Literature |
| Exp002 | Textbook equations (Rowland & Tozer) | Published reference |
| Exp003 | Textbook equations (Rowland & Tozer Ch. 19) | Published reference |
| Exp004 | Fleck/Gonzales 2021 + Kabashima 2020 Phase III | Peer-reviewed papers |
| Exp005 | Published population PK parameters | Literature + simulation |
| Exp010 | Synthetic communities (defined abundances) | Self-generated from literature |
| Exp011 | wetSpring validated lattice code (V99) | ecoPrimals internal |
| Exp012 | Published CDI clinical parameters | Literature |
| Exp020 | MIT-BIH Arrhythmia Database | PhysioNet (open) |
| Exp030 | Shoskes 2016, Ross 2004 (IM PK) | Peer-reviewed papers |
| Exp031 | Testopel PI, Cavender 2009 | FDA label + literature |
| Exp032 | Harman 2001 (BLSA, n=890), Feldman 2002 | Peer-reviewed longitudinal |
| Exp033 | Saad 2013 registry (n=411) | Published figures |
| Exp034 | Saad 2016, Sharma 2015 (VA, n=83,010) | Published registry/cohort |
| Exp035 | Kapoor 2006 (RCT), Dhindsa 2016, Hackett 2014 | Peer-reviewed RCTs |
| Exp036 | Composite registries (simulation) | Literature parameters |
| Exp037 | Cross-track (ecoPrimals thesis) | Internal + Ridaura 2013, Tremellen 2012 |

See `specs/README.md` for full provenance table.
