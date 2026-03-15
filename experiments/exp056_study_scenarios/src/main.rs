// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp056: Generate `petalTongue` scenarios for all 4 healthSpring study tracks
//! and validate that every `DataChannel`, `ClinicalRange`, and edge is well-formed.

use healthspring_barracuda::visualization::scenarios;

#[expect(
    clippy::too_many_lines,
    reason = "validation binary — exercises all 6 study scenarios sequentially"
)]
fn main() {
    let mut checks = 0;
    let mut pass = 0;

    macro_rules! check {
        ($name:expr, $cond:expr) => {{
            checks += 1;
            if $cond {
                pass += 1;
                println!("  [PASS] {}", $name);
            } else {
                println!("  [FAIL] {}", $name);
            }
        }};
    }

    // -----------------------------------------------------------------------
    // Track 1: PK/PD
    // -----------------------------------------------------------------------
    println!("\n=== Track 1: PK/PD ===");
    let (pkpd, pkpd_edges) = scenarios::pkpd_study();
    let pkpd_json = scenarios::scenario_with_edges_json(&pkpd, &pkpd_edges);
    let pkpd_val: serde_json::Value = serde_json::from_str(&pkpd_json).expect("valid JSON");

    check!("pkpd: valid JSON", pkpd_val.is_object());
    check!("pkpd: 6 nodes", pkpd.ecosystem.primals.len() == 6);
    check!("pkpd: 5 edges", pkpd_edges.len() == 5);
    check!(
        "pkpd: has version 2.0.0",
        pkpd_json.contains("\"version\": \"2.0.0\"")
    );

    let hill_node = pkpd
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "hill")
        .unwrap();
    check!(
        "pkpd: hill has 5 channels (4 drugs + EC50 bar)",
        hill_node.data_channels.len() == 5
    );

    let onecomp = pkpd
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "one_comp")
        .unwrap();
    check!(
        "pkpd: one_comp has 4 channels",
        onecomp.data_channels.len() == 4
    );
    check!(
        "pkpd: one_comp has 2 clinical ranges",
        onecomp.clinical_ranges.len() == 2
    );

    let twocomp = pkpd
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "two_comp")
        .unwrap();
    check!(
        "pkpd: two_comp has 3 channels (curve + α + β)",
        twocomp.data_channels.len() == 3
    );

    let pop_pk = pkpd
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "pop_pk")
        .unwrap();
    check!(
        "pkpd: pop_pk has 3 channels (2 dist + scatter3d)",
        pop_pk.data_channels.len() == 3
    );

    let pbpk = pkpd
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "pbpk")
        .unwrap();
    check!(
        "pkpd: pbpk has 9 channels (venous + 5 tissue TS + bar + AUC + CO)",
        pbpk.data_channels.len() == 9
    );

    // Validate JSON has data_channels array with channel_type
    check!(
        "pkpd: JSON has timeseries",
        pkpd_json.contains("\"channel_type\": \"timeseries\"")
    );
    check!(
        "pkpd: JSON has distribution",
        pkpd_json.contains("\"channel_type\": \"distribution\"")
    );
    check!(
        "pkpd: JSON has bar",
        pkpd_json.contains("\"channel_type\": \"bar\"")
    );
    check!(
        "pkpd: JSON has gauge",
        pkpd_json.contains("\"channel_type\": \"gauge\"")
    );
    check!(
        "pkpd: JSON has scatter3d",
        pkpd_json.contains("\"channel_type\": \"scatter3d\"")
    );
    check!("pkpd: JSON has edges", pkpd_val["edges"].is_array());

    // -----------------------------------------------------------------------
    // Track 2: Microbiome
    // -----------------------------------------------------------------------
    println!("\n=== Track 2: Microbiome ===");
    let (micro, micro_edges) = scenarios::microbiome_study();
    let micro_json = scenarios::scenario_with_edges_json(&micro, &micro_edges);
    let micro_val: serde_json::Value = serde_json::from_str(&micro_json).expect("valid JSON");

    check!("micro: valid JSON", micro_val.is_object());
    check!("micro: 4 nodes", micro.ecosystem.primals.len() == 4);
    check!("micro: 3 edges", micro_edges.len() == 3);

    let diversity = micro
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "diversity")
        .unwrap();
    check!(
        "micro: diversity has 4 channels (3 bar + heatmap)",
        diversity.data_channels.len() == 4
    );
    check!(
        "micro: diversity has 2 clinical ranges",
        diversity.clinical_ranges.len() == 2
    );

    let anderson = micro
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "anderson")
        .unwrap();
    check!(
        "micro: anderson has 6 channels (2 spectrum + 3 gauge + 1 bar)",
        anderson.data_channels.len() == 6
    );

    let fmt = micro
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "fmt")
        .unwrap();
    check!("micro: fmt has 2 timeseries", fmt.data_channels.len() == 2);

    check!(
        "micro: JSON has bar",
        micro_json.contains("\"channel_type\": \"bar\"")
    );
    check!(
        "micro: JSON has gauge",
        micro_json.contains("\"channel_type\": \"gauge\"")
    );
    check!(
        "micro: JSON has heatmap",
        micro_json.contains("\"channel_type\": \"heatmap\"")
    );
    check!(
        "micro: JSON has timeseries",
        micro_json.contains("\"channel_type\": \"timeseries\"")
    );

    // -----------------------------------------------------------------------
    // Track 3: Biosignal
    // -----------------------------------------------------------------------
    println!("\n=== Track 3: Biosignal ===");
    let (bio, bio_edges) = scenarios::biosignal_study();
    let bio_json = scenarios::scenario_with_edges_json(&bio, &bio_edges);
    let bio_val: serde_json::Value = serde_json::from_str(&bio_json).expect("valid JSON");

    check!("bio: valid JSON", bio_val.is_object());
    check!("bio: 5 nodes", bio.ecosystem.primals.len() == 5);
    check!("bio: 4 edges", bio_edges.len() == 4);

    let qrs = bio
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "qrs")
        .unwrap();
    check!(
        "bio: qrs has 8 channels (5 TS + 3 gauge)",
        qrs.data_channels.len() == 8
    );

    let hrv = bio
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "hrv")
        .unwrap();
    check!(
        "bio: hrv has 5 channels (tachogram + spectrum + 3 gauge)",
        hrv.data_channels.len() == 5
    );

    let spo2 = bio
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "spo2")
        .unwrap();
    check!(
        "bio: spo2 has 2 channels (calibration + gauge)",
        spo2.data_channels.len() == 2
    );
    check!(
        "bio: spo2 has 2 clinical ranges",
        spo2.clinical_ranges.len() == 2
    );

    let fusion = bio
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "fusion")
        .unwrap();
    check!(
        "bio: fusion has 4 channels (2 EDA + stress + score)",
        fusion.data_channels.len() == 4
    );

    let wfdb = bio
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "wfdb_ecg")
        .unwrap();
    check!(
        "bio: wfdb_ecg has 5 channels (TS + bar + 3 gauge)",
        wfdb.data_channels.len() == 5
    );

    check!(
        "bio: JSON has spectrum",
        bio_json.contains("\"channel_type\": \"spectrum\"")
    );

    // -----------------------------------------------------------------------
    // Track 4: Endocrinology
    // -----------------------------------------------------------------------
    println!("\n=== Track 4: Endocrinology ===");
    let (endo, endo_edges) = scenarios::endocrine_study();
    let endo_json = scenarios::scenario_with_edges_json(&endo, &endo_edges);
    let endo_val: serde_json::Value = serde_json::from_str(&endo_json).expect("valid JSON");

    check!("endo: valid JSON", endo_val.is_object());
    check!("endo: 8 nodes", endo.ecosystem.primals.len() == 8);
    check!("endo: 7 edges", endo_edges.len() == 7);

    let t_im = endo
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "t_im")
        .unwrap();
    check!(
        "endo: t_im has 3 channels (TS + 2 gauge)",
        t_im.data_channels.len() == 3
    );

    let decline = endo
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "age_decline")
        .unwrap();
    check!(
        "endo: age_decline has 4 channels (3 TS + gauge)",
        decline.data_channels.len() == 4
    );

    let gut_axis = endo
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "gut_axis")
        .unwrap();
    check!(
        "endo: gut_axis has 1 bar chart",
        gut_axis.data_channels.len() == 1
    );

    let hrv_cardiac = endo
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "hrv_cardiac")
        .unwrap();
    check!(
        "endo: hrv_cardiac has 3 channels (TS + bar + gauge)",
        hrv_cardiac.data_channels.len() == 3
    );

    // -----------------------------------------------------------------------
    // Track 5: NLME
    // -----------------------------------------------------------------------
    println!("\n=== Track 5: NLME ===");
    let (nlme, nlme_edges) = scenarios::nlme_study();
    let nlme_json = scenarios::scenario_with_edges_json(&nlme, &nlme_edges);
    let nlme_val: serde_json::Value = serde_json::from_str(&nlme_json).expect("valid JSON");

    check!("nlme: valid JSON", nlme_val.is_object());
    check!("nlme: 5 nodes", nlme.ecosystem.primals.len() == 5);
    check!("nlme: 5 edges", nlme_edges.len() == 5);

    let nlme_pop = nlme
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "nlme_population")
        .unwrap();
    check!(
        "nlme: nlme_population has 18 channels",
        nlme_pop.data_channels.len() == 18
    );

    check!(
        "nlme: JSON has distribution",
        nlme_json.contains("\"channel_type\": \"distribution\"")
    );
    check!(
        "nlme: JSON has scatter3d",
        nlme_json.contains("\"channel_type\": \"scatter3d\"")
    );

    // -----------------------------------------------------------------------
    // Full study (all tracks combined)
    // -----------------------------------------------------------------------
    println!("\n=== Full Study (all 5 tracks) ===");
    let (full, full_edges) = scenarios::full_study();
    let full_json = scenarios::scenario_with_edges_json(&full, &full_edges);
    let full_val: serde_json::Value = serde_json::from_str(&full_json).expect("valid JSON");

    check!("full: valid JSON", full_val.is_object());
    check!(
        "full: 34 nodes (6+4+5+8+5+6 V16)",
        full.ecosystem.primals.len() == 34
    );
    check!(
        "full: 38 edges (29 intra + 5 cross + 4 V16 cross)",
        full_edges.len() == 38
    );

    let total_channels: usize = full
        .ecosystem
        .primals
        .iter()
        .map(|n| n.data_channels.len())
        .sum();
    check!("full: > 60 total data channels", total_channels > 60);

    let total_ranges: usize = full
        .ecosystem
        .primals
        .iter()
        .map(|n| n.clinical_ranges.len())
        .sum();
    check!("full: > 8 total clinical ranges", total_ranges > 8);

    // Channel type coverage
    let has_ts = full_json.contains("\"channel_type\": \"timeseries\"");
    let has_dist = full_json.contains("\"channel_type\": \"distribution\"");
    let has_bar = full_json.contains("\"channel_type\": \"bar\"");
    let has_gauge = full_json.contains("\"channel_type\": \"gauge\"");
    let has_spectrum = full_json.contains("\"channel_type\": \"spectrum\"");
    let has_heatmap = full_json.contains("\"channel_type\": \"heatmap\"");
    let has_scatter3d = full_json.contains("\"channel_type\": \"scatter3d\"");
    check!(
        "full: all 7 channel types present",
        has_ts && has_dist && has_bar && has_gauge && has_spectrum && has_heatmap && has_scatter3d
    );

    // JSON size sanity
    let json_kb = full_json.len() / 1024;
    check!(
        &format!("full: JSON size reasonable ({json_kb} KB)"),
        json_kb > 10 && json_kb < 2000
    );

    // Write scenarios to stdout summary
    println!("\n=== Scenario Sizes ===");
    println!(
        "  PK/PD:         {} nodes, {} edges, {} KB JSON",
        pkpd.ecosystem.primals.len(),
        pkpd_edges.len(),
        pkpd_json.len() / 1024
    );
    println!(
        "  Microbiome:    {} nodes, {} edges, {} KB JSON",
        micro.ecosystem.primals.len(),
        micro_edges.len(),
        micro_json.len() / 1024
    );
    println!(
        "  Biosignal:     {} nodes, {} edges, {} KB JSON",
        bio.ecosystem.primals.len(),
        bio_edges.len(),
        bio_json.len() / 1024
    );
    println!(
        "  Endocrinology: {} nodes, {} edges, {} KB JSON",
        endo.ecosystem.primals.len(),
        endo_edges.len(),
        endo_json.len() / 1024
    );
    println!(
        "  Full Study:    {} nodes, {} edges, {} KB JSON",
        full.ecosystem.primals.len(),
        full_edges.len(),
        full_json.len() / 1024
    );
    println!("  Total channels: {total_channels}  Clinical ranges: {total_ranges}");

    println!("\n====================================");
    println!("Exp056 Study Scenarios: {pass}/{checks} checks passed");
    std::process::exit(i32::from(pass != checks));
}
