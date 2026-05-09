// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(
    deprecated,
    reason = "legacy binary, replaced by healthspring_unibin certify"
)]

//! healthSpring guideStone — legacy binary, now delegates to `certification::certify()`.

fn main() {
    eprintln!("╔══════════════════════════════════════════════════════════════════╗");
    eprintln!("║  healthSpring guideStone — self-validating NUCLEUS node        ║");
    eprintln!("║  NOTE: Use `healthspring_unibin certify` instead               ║");
    eprintln!("╚══════════════════════════════════════════════════════════════════╝\n");

    let v = healthspring_barracuda::certification::certify(
        healthspring_barracuda::certification::MAX_TIER,
    );

    if v.failed == 0 && v.passed > 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
