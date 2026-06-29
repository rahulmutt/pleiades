//! Saros series assignment by nearest-cadence match against canon anchors.

// Items here are pub(crate) for upcoming eclipse-engine tasks; silence
// dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::types::EclipseKind;

pub(crate) const SAROS_PERIOD_DAYS: f64 = 6_585.321_3;

/// `(kind, member_jd, series)` anchors taken from the NASA canon. One per active
/// series in the 1900–2100 window, extracted from the eclipse corpus fixture.
pub(crate) const SAROS_ANCHORS: &[(EclipseKind, f64, u32)] = &[
    // Solar series
    (EclipseKind::Solar, 2415848.08688_f64, 108),
    (EclipseKind::Solar, 2421222.36553_f64, 111),
    (EclipseKind::Solar, 2420011.36958_f64, 114),
    (EclipseKind::Solar, 2417413.05161_f64, 115),
    (EclipseKind::Solar, 2421399.05302_f64, 116),
    (EclipseKind::Solar, 2418800.73765_f64, 117),
    (EclipseKind::Solar, 2416202.56624_f64, 118),
    (EclipseKind::Solar, 2420188.50904_f64, 119),
    (EclipseKind::Solar, 2417589.75397_f64, 120),
    (EclipseKind::Solar, 2421576.89398_f64, 121),
    (EclipseKind::Solar, 2418977.58926_f64, 122),
    (EclipseKind::Solar, 2416378.69435_f64, 123),
    (EclipseKind::Solar, 2420366.02392_f64, 124),
    (EclipseKind::Solar, 2417767.14204_f64, 125),
    (EclipseKind::Solar, 2415168.12079_f64, 126),
    (EclipseKind::Solar, 2419155.43567_f64, 127),
    (EclipseKind::Solar, 2416556.73662_f64, 128),
    (EclipseKind::Solar, 2420542.68981_f64, 129),
    (EclipseKind::Solar, 2417944.40650_f64, 130),
    (EclipseKind::Solar, 2415345.80536_f64, 131),
    (EclipseKind::Solar, 2419331.67572_f64, 132),
    (EclipseKind::Solar, 2416733.36413_f64, 133),
    (EclipseKind::Solar, 2420720.45307_f64, 134),
    (EclipseKind::Solar, 2418121.18740_f64, 135),
    (EclipseKind::Solar, 2415522.73181_f64, 136),
    (EclipseKind::Solar, 2419509.98220_f64, 137),
    (EclipseKind::Solar, 2416910.71697_f64, 138),
    (EclipseKind::Solar, 2420897.16691_f64, 139),
    (EclipseKind::Solar, 2418298.98921_f64, 140),
    (EclipseKind::Solar, 2415699.81135_f64, 141),
    (EclipseKind::Solar, 2419686.06683_f64, 142),
    (EclipseKind::Solar, 2417088.04683_f64, 143),
    (EclipseKind::Solar, 2421074.58762_f64, 144),
    (EclipseKind::Solar, 2418475.47127_f64, 145),
    (EclipseKind::Solar, 2415877.44046_f64, 146),
    (EclipseKind::Solar, 2419864.23133_f64, 147),
    (EclipseKind::Solar, 2417264.82176_f64, 148),
    (EclipseKind::Solar, 2421251.81147_f64, 149),
    (EclipseKind::Solar, 2418653.32278_f64, 150),
    (EclipseKind::Solar, 2416053.83354_f64, 151),
    (EclipseKind::Solar, 2420040.69848_f64, 152),
    (EclipseKind::Solar, 2417442.55058_f64, 153),
    (EclipseKind::Solar, 2421428.61299_f64, 154),
    (EclipseKind::Solar, 2425415.35241_f64, 155),
    (EclipseKind::Solar, 2455743.86076_f64, 156),
    (EclipseKind::Solar, 2472900.51360_f64, 157),
    (EclipseKind::Solar, 2476887.24535_f64, 158),
    (EclipseKind::Solar, 2487635.94179_f64, 164),
    // Lunar series
    (EclipseKind::Lunar, 2416541.62678_f64, 102),
    (EclipseKind::Lunar, 2420528.70674_f64, 103),
    (EclipseKind::Lunar, 2420705.01712_f64, 108),
    (EclipseKind::Lunar, 2418107.08787_f64, 109),
    (EclipseKind::Lunar, 2415508.27127_f64, 110),
    (EclipseKind::Lunar, 2419494.42657_f64, 111),
    (EclipseKind::Lunar, 2416896.29169_f64, 112),
    (EclipseKind::Lunar, 2420882.86089_f64, 113),
    (EclipseKind::Lunar, 2418283.41330_f64, 114),
    (EclipseKind::Lunar, 2415685.13563_f64, 115),
    (EclipseKind::Lunar, 2419671.98947_f64, 116),
    (EclipseKind::Lunar, 2417072.65346_f64, 117),
    (EclipseKind::Lunar, 2421059.69869_f64, 118),
    (EclipseKind::Lunar, 2418461.56170_f64, 119),
    (EclipseKind::Lunar, 2415862.28657_f64, 120),
    (EclipseKind::Lunar, 2419848.99848_f64, 121),
    (EclipseKind::Lunar, 2417250.82428_f64, 122),
    (EclipseKind::Lunar, 2421236.82278_f64, 123),
    (EclipseKind::Lunar, 2418637.87131_f64, 124),
    (EclipseKind::Lunar, 2416039.75238_f64, 125),
    (EclipseKind::Lunar, 2420026.03355_f64, 126),
    (EclipseKind::Lunar, 2417427.04178_f64, 127),
    (EclipseKind::Lunar, 2421414.40213_f64, 128),
    (EclipseKind::Lunar, 2418815.73213_f64, 129),
    (EclipseKind::Lunar, 2416216.50902_f64, 130),
    (EclipseKind::Lunar, 2420203.67579_f64, 131),
    (EclipseKind::Lunar, 2417605.06806_f64, 132),
    (EclipseKind::Lunar, 2421590.90731_f64, 133),
    (EclipseKind::Lunar, 2418992.51449_f64, 134),
    (EclipseKind::Lunar, 2416394.13719_f64, 135),
    (EclipseKind::Lunar, 2420380.07983_f64, 136),
    (EclipseKind::Lunar, 2417781.68226_f64, 137),
    (EclipseKind::Lunar, 2415183.64418_f64, 138),
    (EclipseKind::Lunar, 2419169.74750_f64, 139),
    (EclipseKind::Lunar, 2416571.02255_f64, 140),
    (EclipseKind::Lunar, 2420558.26356_f64, 141),
    (EclipseKind::Lunar, 2417959.05667_f64, 142),
    (EclipseKind::Lunar, 2415359.93506_f64, 143),
    (EclipseKind::Lunar, 2419347.15052_f64, 144),
    (EclipseKind::Lunar, 2416748.23245_f64, 145),
    (EclipseKind::Lunar, 2420734.39395_f64, 146),
    (EclipseKind::Lunar, 2418136.39855_f64, 147),
    (EclipseKind::Lunar, 2441878.98564_f64, 148),
    (EclipseKind::Lunar, 2445865.10184_f64, 149),
    (EclipseKind::Lunar, 2456437.67437_f64, 150),
    (EclipseKind::Lunar, 2486765.61367_f64, 151),
    (EclipseKind::Lunar, 2473771.66962_f64, 156),
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
        // Anchor: solar series 145 has a member at JD 2418475.47127 (one of its
        // early eclipses). One saros (~6585.32 d) later is still series 145.
        let jd = 2418475.47127 + SAROS_PERIOD_DAYS;
        assert_eq!(saros_series(EclipseKind::Solar, jd), 145);
    }
}
