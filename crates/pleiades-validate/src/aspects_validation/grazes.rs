//! Graze windows of the lunar-point aspect corpus (issue #168 (c)).
//!
//! `tools/se-aspects-reference --lunar-points` writes a window wherever a
//! pair's separation turns within its graze margin of a level. There an event
//! pair may exist in one ephemeris only, so the gate compares events inside a
//! near-tangent window (a turn under half the margin from the level) by count
//! parity, requires equal counts in the other windows, checks that the engine
//! puts each window's edges on SE's side of the level, and compares everything
//! outside strictly.

use super::{
    fnv1a64, max_nan, parse_manifest, residuals_of, wrap180, AspectsError, Expected, Found, Group,
    Pair, Row, ANGLES_DEG,
};

/// One graze window: indices into the corpus plan and [`ANGLES_DEG`], and the
/// window's bounds (Julian days, TT, both inclusive).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GrazeWindow {
    pub(super) pair: usize,
    pub(super) angle: usize,
    pub(super) lo: f64,
    pub(super) hi: f64,
    /// The corpus rows of the pair and angle inside the window.
    pub(super) se_events: usize,
    /// The signed level grazed: `+angle` or `-angle` of
    /// `wrap180(lon(first) − lon(second))`.
    pub(super) level_deg: f64,
    /// SE's side of the level at `lo` and at `hi` (`+1` above, `-1` below).
    pub(super) lo_side: i8,
    pub(super) hi_side: i8,
    /// How far the closest turn misses the level, and the pair's graze margin
    /// (arcseconds).
    pub(super) miss_arcsec: f64,
    pub(super) margin_arcsec: f64,
}

impl GrazeWindow {
    fn contains(&self, jd: f64) -> bool {
        self.lo <= jd && jd <= self.hi
    }

    /// Whether the window's counts must agree exactly: its turn misses the
    /// level by at least half the margin, about twice the engine–SE
    /// separation at turns, so an event pair there cannot depend on the
    /// ephemeris. Closer windows only need the parity.
    pub(super) fn strict(&self) -> bool {
        self.miss_arcsec >= 0.5 * self.margin_arcsec
    }
}

/// What the window comparison saw for one series or more.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct WindowStats {
    pub(super) windows: usize,
    /// Windows whose counts must agree exactly ([`GrazeWindow::strict`]).
    pub(super) strict: usize,
    /// Windows whose engine and corpus counts differ (allowed in a
    /// near-tangent window when both have the window's parity).
    pub(super) differing: usize,
    /// Events paired inside windows whose counts agree (not ceiling-checked).
    pub(super) compared: usize,
    pub(super) max_sep_arcsec: f64,
    pub(super) max_time_s: f64,
    pub(super) max_lon_arcsec: f64,
}

impl WindowStats {
    pub(super) fn absorb(&mut self, other: WindowStats) {
        self.windows += other.windows;
        self.strict += other.strict;
        self.differing += other.differing;
        self.compared += other.compared;
        self.max_sep_arcsec = max_nan(self.max_sep_arcsec, other.max_sep_arcsec);
        self.max_time_s = max_nan(self.max_time_s, other.max_time_s);
        self.max_lon_arcsec = max_nan(self.max_lon_arcsec, other.max_lon_arcsec);
    }
}

/// Parses the grazes CSV against the corpus plan `pairs`. Every window has
/// `lo < hi`, holds its turn, and does not overlap
/// the previous window of its pair and angle (rows ascend by `lo`).
pub(super) fn parse_grazes(csv: &str, pairs: &[Pair]) -> Result<Vec<GrazeWindow>, AspectsError> {
    let malformed = |what: String| AspectsError::MalformedGrazes(what);
    let mut windows: Vec<GrazeWindow> = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("group,") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 13 {
            return Err(malformed(format!(
                "expected 13 fields, got {} in {line}",
                fields.len()
            )));
        }
        let group = Group::from_name(fields[0])
            .ok_or_else(|| malformed(format!("unknown group {} in {line}", fields[0])))?;
        let pair = pairs
            .iter()
            .position(|p| p.group == group && p.first == fields[1] && p.second == fields[2])
            .ok_or_else(|| {
                malformed(format!(
                    "pair {}-{} is not in the plan in {line}",
                    fields[1], fields[2]
                ))
            })?;
        let num = |i: usize| -> Result<f64, AspectsError> {
            fields[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| malformed(format!("field {i} is not a finite number in {line}")))
        };
        let angle_deg = num(3)?;
        let angle = ANGLES_DEG
            .iter()
            .position(|known| *known == angle_deg)
            .ok_or_else(|| malformed(format!("angle {angle_deg} is not in the plan in {line}")))?;
        let (turn, miss_arcsec, lo, hi) = (num(4)?, num(5)?, num(6)?, num(7)?);
        let se_events: usize = fields[8]
            .parse()
            .map_err(|_| malformed(format!("se_events is not a count in {line}")))?;
        let (level_deg, margin_arcsec) = (num(9)?, num(12)?);
        let side = |i: usize| -> Result<i8, AspectsError> {
            match fields[i] {
                "1" => Ok(1),
                "-1" => Ok(-1),
                _ => Err(malformed(format!(
                    "field {i} is not a side (1 or -1) in {line}"
                ))),
            }
        };
        let (lo_side, hi_side) = (side(10)?, side(11)?);
        if !(lo < hi && lo <= turn && turn <= hi) {
            return Err(malformed(format!(
                "window does not hold its turn, or lo >= hi, in {line}"
            )));
        }
        // 0 and 180 degrees have one level each; the others are +-angle.
        let level_ok = level_deg == angle_deg
            || (level_deg == -angle_deg && angle_deg != 0.0 && angle_deg != 180.0);
        if !level_ok {
            return Err(malformed(format!(
                "level {level_deg} is not a level of {angle_deg} degrees in {line}"
            )));
        }
        if (lo_side != hi_side) != (se_events % 2 == 1) {
            return Err(malformed(format!(
                "edge sides {lo_side},{hi_side} contradict se_events {se_events}'s parity in {line}"
            )));
        }
        if !(margin_arcsec > 0.0 && (0.0..margin_arcsec).contains(&miss_arcsec)) {
            return Err(malformed(format!(
                "miss {miss_arcsec} is not within the margin {margin_arcsec} in {line}"
            )));
        }
        if let Some(prev) = windows
            .iter()
            .rev()
            .find(|w| w.pair == pair && w.angle == angle)
        {
            if lo <= prev.hi {
                return Err(malformed(format!(
                    "window overlaps or precedes the previous one in {line}"
                )));
            }
        }
        windows.push(GrazeWindow {
            pair,
            angle,
            lo,
            hi,
            se_events,
            level_deg,
            lo_side,
            hi_side,
            miss_arcsec,
            margin_arcsec,
        });
    }
    Ok(windows)
}

/// Each window's edges must lie on SE's side of its level in the engine too:
/// `separation_at(jd)` is the engine's `wrap180(lon(first) − lon(second))`.
/// Parity alone misses an engine that crosses the level at both edges.
pub(super) fn check_window_edges(
    label: &str,
    windows: &[GrazeWindow],
    separation_at: impl Fn(f64) -> Result<f64, AspectsError>,
) -> Result<(), AspectsError> {
    for w in windows {
        for (jd, se_side) in [(w.lo, w.lo_side), (w.hi, w.hi_side)] {
            let offset = wrap180(separation_at(jd)? - w.level_deg);
            let engine_side = if offset > 0.0 { 1 } else { -1 };
            if engine_side != se_side {
                return Err(AspectsError::GrazeEdgeSide {
                    series: label.to_string(),
                    jd_tt: jd,
                    level_deg: w.level_deg,
                    se_side,
                    engine_offset_arcsec: offset * 3600.0,
                });
            }
        }
    }
    Ok(())
}

/// Each window's `se_events` must equal the corpus rows of its pair and angle
/// inside it.
pub(super) fn check_window_counts(
    windows: &[GrazeWindow],
    rows: &[Row],
) -> Result<(), AspectsError> {
    for w in windows {
        let corpus = rows
            .iter()
            .filter(|r| r.pair == w.pair && r.angle == w.angle && w.contains(r.expected.found.jd))
            .count();
        if corpus != w.se_events {
            return Err(AspectsError::GrazeEventCount {
                pair: w.pair,
                angle_deg: ANGLES_DEG[w.angle],
                lo: w.lo,
                hi: w.hi,
                recorded: w.se_events,
                corpus,
            });
        }
    }
    Ok(())
}

/// Ties the grazes file to its manifest and to the corpus rows: checksum, row
/// count, parse, and window counts.
pub(super) fn check_grazes(
    csv: &str,
    manifest: &str,
    pairs: &[Pair],
    rows: &[Row],
) -> Result<Vec<GrazeWindow>, AspectsError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got = fnv1a64(csv);
    if got != manifest_checksum {
        return Err(AspectsError::ChecksumMismatch {
            got,
            want: manifest_checksum,
        });
    }
    let windows = parse_grazes(csv, pairs)?;
    if windows.len() != manifest_rows {
        return Err(AspectsError::ManifestDrift {
            rows_csv: windows.len(),
            rows_manifest: manifest_rows,
        });
    }
    check_window_counts(&windows, rows)?;
    Ok(windows)
}

/// The windows of one pair and angle, in order.
pub(super) fn windows_of(windows: &[GrazeWindow], pair: usize, angle: usize) -> Vec<GrazeWindow> {
    windows
        .iter()
        .filter(|w| w.pair == pair && w.angle == angle)
        .copied()
        .collect()
}

/// Splits one series at its `windows`: returns the engine events and corpus
/// rows outside every window, for the strict comparison, and what the windows
/// held. Inside each window both counts must have the window's parity
/// (`se_events % 2`, set by whether its edges straddle the level), and a
/// [strict](GrazeWindow::strict) window's counts must be equal; equal
/// non-zero counts are paired in order and their residuals recorded, without
/// ceilings.
pub(super) fn split_windows(
    label: &str,
    windows: &[GrazeWindow],
    engine: &[Found],
    corpus: &[Expected],
) -> Result<(Vec<Found>, Vec<Expected>, WindowStats), AspectsError> {
    let mut stats = WindowStats::default();
    for w in windows {
        let got: Vec<&Found> = engine.iter().filter(|f| w.contains(f.jd)).collect();
        let want: Vec<&Expected> = corpus.iter().filter(|e| w.contains(e.found.jd)).collect();
        let parity = w.se_events % 2;
        for (side, count) in [("engine", got.len()), ("corpus", want.len())] {
            if count % 2 != parity {
                return Err(AspectsError::GrazeWindowParity {
                    series: label.to_string(),
                    lo: w.lo,
                    hi: w.hi,
                    side,
                    count,
                    se_events: w.se_events,
                });
            }
        }
        stats.windows += 1;
        if w.strict() {
            stats.strict += 1;
        }
        if got.len() != want.len() {
            if w.strict() {
                return Err(AspectsError::GrazeWindowCount {
                    series: label.to_string(),
                    lo: w.lo,
                    hi: w.hi,
                    engine: got.len(),
                    corpus: want.len(),
                    miss_arcsec: w.miss_arcsec,
                    margin_arcsec: w.margin_arcsec,
                });
            }
            stats.differing += 1;
            continue;
        }
        for (g, e) in got.iter().zip(&want) {
            let (sep, time_s, lon) = residuals_of(g, e);
            stats.compared += 1;
            stats.max_sep_arcsec = max_nan(stats.max_sep_arcsec, sep);
            stats.max_time_s = max_nan(stats.max_time_s, time_s.abs());
            stats.max_lon_arcsec = max_nan(stats.max_lon_arcsec, lon);
        }
    }
    let outside = |jd: f64| !windows.iter().any(|w| w.contains(jd));
    let engine_out = engine.iter().filter(|f| outside(f.jd)).copied().collect();
    let corpus_out = corpus
        .iter()
        .filter(|e| outside(e.found.jd))
        .copied()
        .collect();
    Ok((engine_out, corpus_out, stats))
}
