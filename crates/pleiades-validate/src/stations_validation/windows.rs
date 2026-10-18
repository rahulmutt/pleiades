//! Speed-graze windows of the apogee and fictitious-body station corpora
//! (issue #167 (d)).
//!
//! `tools/se-stations-reference` writes a window wherever a body's central-
//! difference speed turns within its margin of zero. There a station pair may
//! exist in one ephemeris only, so the gate compares stations inside a window
//! by count parity (or exactly, when the turn misses zero by at least half the
//! margin), and everything outside strictly.

use super::{body_from_name, fnv1a64, parse_manifest, Found, Group, Series, StationsError};

/// One speed-graze window of a series (Julian days, TT, bounds inclusive).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SpeedWindow {
    pub(super) group: Group,
    pub(super) body_name: &'static str,
    pub(super) turn_jd: f64,
    /// How far the speed's extremum stays from zero, deg/day.
    pub(super) miss: f64,
    pub(super) lo: f64,
    pub(super) hi: f64,
    /// The corpus stations inside the window.
    pub(super) se_stations: usize,
    /// Signs of the SE speed at the window's edges.
    pub(super) lo_side: i8,
    pub(super) hi_side: i8,
    pub(super) margin: f64,
}

impl SpeedWindow {
    pub(super) fn contains(&self, jd: f64) -> bool {
        self.lo <= jd && jd <= self.hi
    }

    /// A turn that clears zero by half the margin or more: the engine must
    /// agree on its station count, not only the parity.
    pub(super) fn strict(&self) -> bool {
        self.miss >= 0.5 * self.margin
    }
}

/// What the window comparison saw for one series or more.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct WindowStats {
    pub(super) windows: usize,
    /// Windows compared strictly (equal counts required).
    pub(super) strict: usize,
    /// Windows whose engine and corpus counts differ (allowed when both have
    /// the window's parity).
    pub(super) differing: usize,
}

/// Parses a windows CSV. Every window holds its turn, has `lo < hi`, a
/// positive margin that its miss stays under, edge sides whose agreement
/// matches the parity of its station count, and does not overlap the previous
/// window of its series.
pub(super) fn parse_windows(csv: &str) -> Result<Vec<SpeedWindow>, StationsError> {
    let malformed = |what: String| StationsError::MalformedWindows(what);
    let mut windows: Vec<SpeedWindow> = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("group,") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 10 {
            return Err(malformed(format!(
                "expected 10 fields, got {} in {line}",
                fields.len()
            )));
        }
        let group = Group::from_name(fields[0])
            .ok_or_else(|| malformed(format!("unknown group {} in {line}", fields[0])))?;
        let (_, body_name) = body_from_name(fields[1])
            .ok_or_else(|| malformed(format!("unknown body {} in {line}", fields[1])))?;
        let num = |i: usize| -> Result<f64, StationsError> {
            fields[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| malformed(format!("field {i} is not a finite number in {line}")))
        };
        let side = |i: usize| -> Result<i8, StationsError> {
            match fields[i] {
                "1" => Ok(1),
                "-1" => Ok(-1),
                _ => Err(malformed(format!("field {i} is not 1 or -1 in {line}"))),
            }
        };
        let (turn_jd, miss, lo, hi, margin) = (num(2)?, num(3)?, num(4)?, num(5)?, num(9)?);
        let se_stations: usize = fields[6]
            .parse()
            .map_err(|_| malformed(format!("se_stations is not a count in {line}")))?;
        let (lo_side, hi_side) = (side(7)?, side(8)?);
        if !(lo < hi && lo <= turn_jd && turn_jd <= hi) {
            return Err(malformed(format!(
                "window does not hold its turn, or lo >= hi, in {line}"
            )));
        }
        if !(margin > 0.0 && 0.0 <= miss && miss < margin) {
            return Err(malformed(format!(
                "margin is not positive or miss is not within it in {line}"
            )));
        }
        if (lo_side != hi_side) != (se_stations % 2 == 1) {
            return Err(malformed(format!(
                "edge sides contradict the parity of se_stations in {line}"
            )));
        }
        if let Some(prev) = windows
            .iter()
            .rev()
            .find(|w| w.group == group && w.body_name == body_name)
        {
            if lo <= prev.hi {
                return Err(malformed(format!(
                    "window overlaps or precedes the previous one in {line}"
                )));
            }
        }
        windows.push(SpeedWindow {
            group,
            body_name,
            turn_jd,
            miss,
            lo,
            hi,
            se_stations,
            lo_side,
            hi_side,
            margin,
        });
    }
    Ok(windows)
}

/// Ties the windows file to its manifest and to the corpus: checksum, parse,
/// row count, and each window's `se_stations` against its series.
pub(super) fn check_windows(
    csv: &str,
    manifest: &str,
    series: &[Series],
) -> Result<Vec<SpeedWindow>, StationsError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got = fnv1a64(csv);
    if got != manifest_checksum {
        return Err(StationsError::ChecksumMismatch {
            got,
            want: manifest_checksum,
        });
    }
    let windows = parse_windows(csv)?;
    if windows.len() != manifest_rows {
        return Err(StationsError::ManifestDrift {
            rows_csv: windows.len(),
            rows_manifest: manifest_rows,
        });
    }
    for w in &windows {
        let owner = series
            .iter()
            .find(|s| s.group == w.group && s.body_name == w.body_name);
        let corpus = owner.map_or(0, |s| {
            s.stations.iter().filter(|f| w.contains(f.jd)).count()
        });
        if corpus != w.se_stations {
            return Err(StationsError::WindowStationCount {
                series: format!("{} {}", w.group.name(), w.body_name),
                lo: w.lo,
                hi: w.hi,
                recorded: w.se_stations,
                corpus,
            });
        }
    }
    Ok(windows)
}

/// Splits one series at its `windows`: returns the engine and corpus stations
/// outside every window, for the strict comparison, and what the windows
/// held. Inside each window both counts must have the window's parity
/// (`se_stations % 2`); a strict window must also have equal counts.
pub(super) fn split_windows(
    label: &str,
    windows: &[SpeedWindow],
    engine: &[Found],
    corpus: &[Found],
) -> Result<(Vec<Found>, Vec<Found>, WindowStats), StationsError> {
    let mut stats = WindowStats::default();
    for w in windows {
        let got = engine.iter().filter(|f| w.contains(f.jd)).count();
        let want = corpus.iter().filter(|f| w.contains(f.jd)).count();
        for (side, count) in [("engine", got), ("corpus", want)] {
            if count % 2 != w.se_stations % 2 {
                return Err(StationsError::WindowParity {
                    series: label.to_string(),
                    lo: w.lo,
                    hi: w.hi,
                    side,
                    count,
                    se_stations: w.se_stations,
                });
            }
        }
        stats.windows += 1;
        if w.strict() {
            stats.strict += 1;
            if got != want {
                return Err(StationsError::WindowCount {
                    series: label.to_string(),
                    lo: w.lo,
                    hi: w.hi,
                    engine: got,
                    corpus: want,
                });
            }
        } else if got != want {
            stats.differing += 1;
        }
    }
    let outside = |jd: f64| !windows.iter().any(|w| w.contains(jd));
    let engine_out = engine.iter().filter(|f| outside(f.jd)).copied().collect();
    let corpus_out = corpus.iter().filter(|f| outside(f.jd)).copied().collect();
    Ok((engine_out, corpus_out, stats))
}

/// At each window edge the engine's speed must be on the side the SE speed
/// was (zero counts as negative, as in the reference tool), so the parity
/// argument holds for the engine too.
pub(super) fn check_window_edges(
    label: &str,
    windows: &[SpeedWindow],
    speed_at: impl Fn(f64) -> Result<f64, StationsError>,
) -> Result<(), StationsError> {
    for w in windows {
        for (jd, se_side) in [(w.lo, w.lo_side), (w.hi, w.hi_side)] {
            let engine_speed = speed_at(jd)?;
            let engine_side = if engine_speed > 0.0 { 1 } else { -1 };
            if engine_side != se_side {
                return Err(StationsError::WindowEdgeSide {
                    series: label.to_string(),
                    jd_tt: jd,
                    se_side,
                    engine_speed,
                });
            }
        }
    }
    Ok(())
}
