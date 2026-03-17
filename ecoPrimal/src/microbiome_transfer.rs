// SPDX-License-Identifier: AGPL-3.0-or-later
//! Cross-spring gut Anderson parameter export for neuralSpring digester transfer.
//!
//! neuralSpring Paper 027 (ESN digester prediction) is blocked on healthSpring
//! providing gut Anderson parameters. This module exports the validated
//! parameters from Tracks 2 and 6 in a form other springs can consume.

use serde::{Deserialize, Serialize};

/// Gut Anderson parameters for cross-spring consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GutAndersonParams {
    pub species: String,
    pub community_type: String,
    pub shannon_index: f64,
    pub pielou_evenness: f64,
    pub disorder_w: f64,
    pub localization_length_xi: f64,
    pub colonization_resistance: f64,
}

/// Published gut Anderson parameters from healthSpring validation.
///
/// Exp010-013 (human), Exp105 (canine).
#[must_use]
pub fn validated_gut_params() -> Vec<GutAndersonParams> {
    vec![
        GutAndersonParams {
            species: "human".into(),
            community_type: "healthy".into(),
            shannon_index: 3.0,
            pielou_evenness: 0.85,
            disorder_w: 8.5,
            localization_length_xi: 2.1,
            colonization_resistance: 0.92,
        },
        GutAndersonParams {
            species: "human".into(),
            community_type: "dysbiotic_cdi".into(),
            shannon_index: 1.2,
            pielou_evenness: 0.45,
            disorder_w: 4.5,
            localization_length_xi: 8.7,
            colonization_resistance: 0.31,
        },
        GutAndersonParams {
            species: "human".into(),
            community_type: "post_fmt".into(),
            shannon_index: 2.5,
            pielou_evenness: 0.72,
            disorder_w: 7.2,
            localization_length_xi: 3.4,
            colonization_resistance: 0.78,
        },
        GutAndersonParams {
            species: "canine".into(),
            community_type: "healthy".into(),
            shannon_index: 1.72,
            pielou_evenness: 0.96,
            disorder_w: 9.6,
            localization_length_xi: 1.8,
            colonization_resistance: 0.95,
        },
        GutAndersonParams {
            species: "canine".into(),
            community_type: "ad_affected".into(),
            shannon_index: 0.90,
            pielou_evenness: 0.50,
            disorder_w: 5.0,
            localization_length_xi: 6.5,
            colonization_resistance: 0.42,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validated_params_non_empty() {
        assert!(!validated_gut_params().is_empty());
    }

    #[test]
    #[expect(clippy::unwrap_used, reason = "test code")]
    fn healthy_higher_resistance_than_dysbiotic() {
        let params = validated_gut_params();
        let healthy = params
            .iter()
            .find(|p| p.community_type == "healthy" && p.species == "human")
            .unwrap();
        let dysbiotic = params
            .iter()
            .find(|p| p.community_type == "dysbiotic_cdi")
            .unwrap();
        assert!(healthy.colonization_resistance > dysbiotic.colonization_resistance);
    }

    #[test]
    fn canine_params_present() {
        let params = validated_gut_params();
        assert!(params.iter().any(|p| p.species == "canine"));
    }
}
