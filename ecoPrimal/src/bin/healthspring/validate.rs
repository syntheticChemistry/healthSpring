// SPDX-License-Identifier: AGPL-3.0-or-later

//! Validate subcommand — runs scenarios from the registry.

use std::sync::Arc;

use primalspring::composition::CompositionContext;
use primalspring::validation::{NullSink, ValidationResult};

use healthspring_barracuda::validation::scenarios::{Tier, Track, build_registry};

/// CLI-friendly tier filter.
#[derive(Clone, Debug)]
pub enum TierFilter {
    Rust,
    Live,
    Both,
}

impl std::str::FromStr for TierFilter {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "rust" => Ok(Self::Rust),
            "live" => Ok(Self::Live),
            "both" => Ok(Self::Both),
            _ => Err(format!(
                "unknown tier filter: {s} (expected rust, live, or both)"
            )),
        }
    }
}

fn parse_track(s: &str) -> Option<Track> {
    match s.to_ascii_lowercase().as_str() {
        "pkpd" | "pk_pd" | "pk-pd" => Some(Track::PkPd),
        "microbiome" => Some(Track::Microbiome),
        "biosignal" => Some(Track::Biosignal),
        "endocrine" => Some(Track::Endocrine),
        "comparative" => Some(Track::Comparative),
        "discovery" => Some(Track::Discovery),
        "composition" => Some(Track::Composition),
        "toxicology" => Some(Track::Toxicology),
        _ => None,
    }
}

const fn tier_matches(scenario_tier: Tier, filter: &TierFilter) -> bool {
    match filter {
        TierFilter::Rust => matches!(scenario_tier, Tier::Rust | Tier::Both),
        TierFilter::Live => matches!(scenario_tier, Tier::Live | Tier::Both),
        TierFilter::Both => true,
    }
}

pub fn cmd_list_scenarios(
    tier_filter: Option<&TierFilter>,
    track_filter: Option<&str>,
) {
    let registry = build_registry();
    let track_parsed = track_filter.and_then(parse_track);

    let filtered: Vec<_> = registry
        .into_iter()
        .filter(|s| {
            if let Some(tf) = tier_filter {
                if !tier_matches(s.meta.tier, tf) {
                    return false;
                }
            }
            if let Some(ref track) = track_parsed {
                if s.meta.track != *track {
                    return false;
                }
            }
            true
        })
        .collect();

    println!("{} scenario(s) registered:\n", filtered.len());
    for s in &filtered {
        println!(
            "  {:30} [{:?} / {:?}]  {}",
            s.meta.id, s.meta.track, s.meta.tier, s.meta.description
        );
    }
}

pub fn cmd_validate(
    tier_filter: Option<&TierFilter>,
    track_filter: Option<&str>,
    scenario_filter: Option<&str>,
    json_mode: bool,
) {
    let registry = build_registry();
    let track_parsed = track_filter.and_then(parse_track);

    let filtered: Vec<_> = registry
        .into_iter()
        .filter(|s| {
            if let Some(tf) = tier_filter {
                if !tier_matches(s.meta.tier, tf) {
                    return false;
                }
            }
            if let Some(ref track) = track_parsed {
                if s.meta.track != *track {
                    return false;
                }
            }
            if let Some(id) = scenario_filter {
                if s.meta.id != id {
                    return false;
                }
            }
            true
        })
        .collect();

    if !json_mode {
        eprintln!(
            "healthSpring validate: {} scenario(s) selected\n",
            filtered.len()
        );
    }

    let mut v = if json_mode {
        ValidationResult::new("healthspring validation").with_sink(Arc::new(NullSink))
    } else {
        ValidationResult::new("healthspring validation")
    };
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();

    for scenario in &filtered {
        v.section(&format!(
            "{} [{:?} / {:?}]",
            scenario.meta.id, scenario.meta.track, scenario.meta.tier
        ));
        (scenario.run)(&mut v, &mut ctx);
    }

    if json_mode {
        let total = v.passed + v.failed + v.skipped;
        println!(
            "{{\"name\":\"healthspring_validate\",\"passed\":{},\"failed\":{},\"skipped\":{},\"total\":{}}}",
            v.passed, v.failed, v.skipped, total
        );
    } else {
        v.finish();
    }

    if v.failed > 0 {
        std::process::exit(1);
    }
}
