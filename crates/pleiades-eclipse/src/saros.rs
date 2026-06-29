//! Saros series assignment by nearest-cadence match against canon anchors.

// Items here are pub(crate) for upcoming eclipse-engine tasks; silence
// dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::types::EclipseKind;

pub(crate) const SAROS_PERIOD_DAYS: f64 = 6_585.321_3;

/// `(kind, member_jd, series)` anchors taken from the NASA canon. Extended in
/// the validation-fixture build (Task 10) to cover every active 1900–2100 series.
pub(crate) const SAROS_ANCHORS: &[(EclipseKind, f64, u32)] = &[
    (EclipseKind::Solar, 2_451_044.5, 145), // 1999-08-11 total solar
    (EclipseKind::Solar, 2_457_987.27, 145), // 2017-08-21 total solar (same series)
    (EclipseKind::Lunar, 2_458_504.9, 134), // 2019-01-21 total lunar
];

pub(crate) fn saros_series(kind: EclipseKind, greatest_eclipse_jd: f64) -> u32 {
    let mut best: Option<(u32, f64)> = None;
    for &(anchor_kind, anchor_jd, series) in SAROS_ANCHORS {
        if anchor_kind != kind {
            continue;
        }
        // How far is `greatest_eclipse_jd` from an integer number of saros
        // periods away from this anchor?
        let periods = ((greatest_eclipse_jd - anchor_jd) / SAROS_PERIOD_DAYS).round();
        let residual = (greatest_eclipse_jd - (anchor_jd + periods * SAROS_PERIOD_DAYS)).abs();
        #[allow(clippy::unnecessary_map_or)]
        if best.map_or(true, |(_, r)| residual < r) {
            best = Some((series, residual));
        }
    }
    best.map(|(series, _)| series).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EclipseKind;

    #[test]
    fn picks_the_series_one_saros_period_away() {
        // Anchor: solar series 145 has a member at JD 2_451_044.5 (1999-08-11).
        // One saros (~6585.32 d) later is still series 145.
        let jd = 2_451_044.5 + SAROS_PERIOD_DAYS;
        assert_eq!(saros_series(EclipseKind::Solar, jd), 145);
    }
}
