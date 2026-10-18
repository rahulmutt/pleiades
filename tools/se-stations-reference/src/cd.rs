//! Stations as zeros of the central difference of a longitude series
//! (issue #167 (d)), with rejection of the artefact pairs an isolated jump in
//! the series creates, and graze windows around near-zero speed extrema.
//!
//! Two limits of window detection. An extremum is recorded only when the grid
//! samples on either side of it are on its side of zero, so a dip across zero
//! lasting two grid samples or fewer makes stations, not a window (the gate
//! then compares them strictly and fails loudly if the engine differs). And
//! the recorded miss is the sampled value, not a refined minimum: it is biased
//! away from zero by at most about curvature x grid^2 / 8, which the 3x margin
//! factor covers.
//!
//! Pure: every function takes the longitude as a closure, so the tests drive
//! it with synthetic series and `main.rs` with `swe_calc`.

/// Stations bisected to this width (days).
pub const BISECT_TOLERANCE_DAYS: f64 = 1e-7;
/// The scan grid starts this far after the span start, off the regular
/// instants where SE's apogee jumps were seen (spike 2026-10-18).
pub const GRID_OFFSET_DAYS: f64 = 0.001_37;
/// The second scan's step, relative to the first.
pub const CHECK_STEP_RATIO: f64 = 1.3;
/// Two scans agree on a station when they find it this close (days).
pub const MATCH_TOLERANCE_DAYS: f64 = 1.0 / 86_400.0;
/// Only extrema below this |speed| are written to the extrema file (deg/day),
/// the margins' basis. SE's osculating-apogee speed has step discontinuities
/// of 5e-3 to 0.2 deg/day (e.g. at JD 2426487.583 and 2433734.433) that make
/// sampled extrema of their own; between 0.02 and 0.1 deg/day three of them
/// differ from the engine by 0.08-0.21 deg/day. 0.02 is 25 times the apogee
/// margin, so no extremum above it can become a window (spike 2026-10-18).
pub const EXTREMUM_RECORD_DEG_PER_DAY: f64 = 0.02;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Turns retrograde (speed goes from positive to negative).
    R,
    /// Turns direct.
    D,
}

impl Kind {
    pub fn letter(self) -> &'static str {
        match self {
            Self::R => "R",
            Self::D => "D",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Station {
    pub jd: f64,
    pub kind: Kind,
}

/// One scan: its stations and the sampled extrema of the speed that stay on
/// one side of zero, as `(jd, speed)`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scan {
    pub stations: Vec<Station>,
    pub extrema: Vec<(f64, f64)>,
}

/// One graze window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub turn_jd: f64,
    pub miss_deg_per_day: f64,
    pub lo: f64,
    pub hi: f64,
    pub se_stations: usize,
    pub lo_side: i8,
    pub hi_side: i8,
}

pub fn wrap180(d: f64) -> f64 {
    (d + 180.0).rem_euclid(360.0) - 180.0
}

/// `wrap180(lon(jd + h) − lon(jd − h)) / 2h`, degrees per day.
pub fn central_difference(lon: &dyn Fn(f64) -> f64, jd: f64, h: f64) -> f64 {
    wrap180(lon(jd + h) - lon(jd - h)) / (2.0 * h)
}

pub fn scan(lon: &dyn Fn(f64) -> f64, span: (f64, f64), grid: f64, h: f64) -> Scan {
    let speed = |jd: f64| central_difference(lon, jd, h);
    let lo = span.0 + GRID_OFFSET_DAYS;
    let steps = ((span.1 - lo) / grid).floor() as u64;
    let mut out = Scan::default();
    let (mut p2, mut p1, mut prev_jd) = (f64::NAN, speed(lo), lo);
    for k in 1..=steps {
        let jd = lo + k as f64 * grid;
        let cur = speed(jd);
        if (p1 <= 0.0) != (cur <= 0.0) {
            let (mut a, mut b, mut f_a) = (prev_jd, jd, p1);
            while b - a > BISECT_TOLERANCE_DAYS {
                let mid = 0.5 * (a + b);
                let f_mid = speed(mid);
                if (f_a <= 0.0) == (f_mid <= 0.0) {
                    a = mid;
                    f_a = f_mid;
                } else {
                    b = mid;
                }
            }
            let kind = if cur > 0.0 { Kind::D } else { Kind::R };
            out.stations.push(Station {
                jd: 0.5 * (a + b),
                kind,
            });
        } else if p2.is_finite() && (p1 - p2) * (cur - p1) < 0.0 && (p2 <= 0.0) == (p1 <= 0.0) {
            out.extrema.push((prev_jd, p1));
        }
        p2 = p1;
        p1 = cur;
        prev_jd = jd;
    }
    out
}

/// Keeps the stations of `primary` that `check` (the 1.3 h scan) also finds,
/// with the same kind within [`MATCH_TOLERANCE_DAYS`]. Every station found by
/// one scan only must be one of an R/D pair less than 3 h apart in that scan,
/// the signature of an isolated jump stepped over at t ± h; such pairs are
/// returned as dropped (those of `primary`). Anything else is an error.
pub fn reject_artefacts(
    primary: &[Station],
    check: &[Station],
    h: f64,
) -> Result<(Vec<Station>, Vec<[Station; 2]>), String> {
    let matched = |s: &Station, other: &[Station]| {
        other
            .iter()
            .any(|o| o.kind == s.kind && (o.jd - s.jd).abs() <= MATCH_TOLERANCE_DAYS)
    };
    let pairs_of = |list: &[Station],
                    other: &[Station],
                    width: f64|
     -> Result<Vec<[Station; 2]>, String> {
        let lone: Vec<Station> = list
            .iter()
            .filter(|s| !matched(s, other))
            .copied()
            .collect();
        let mut pairs = Vec::new();
        let mut i = 0;
        while i < lone.len() {
            let pair = lone
                .get(i + 1)
                .filter(|next| next.kind != lone[i].kind && next.jd - lone[i].jd < width);
            match pair {
                    Some(next) => {
                        pairs.push([lone[i], *next]);
                        i += 2;
                    }
                    None => {
                        return Err(format!(
                            "station at jd_tt={:.7} ({}) is found by one scan only and is not one of an artefact pair",
                            lone[i].jd,
                            lone[i].kind.letter()
                        ))
                    }
                }
        }
        Ok(pairs)
    };
    let dropped = pairs_of(primary, check, 3.0 * h)?;
    pairs_of(check, primary, 3.0 * CHECK_STEP_RATIO * h)?;
    let kept = primary
        .iter()
        .filter(|s| matched(s, check))
        .copied()
        .collect();
    Ok((kept, dropped))
}

/// One window per extremum of `extrema` within `margin` of zero: from the
/// first grid instant on each side (on the scan's grid, from the extremum's
/// sample) at which `|speed| >= 2 × margin`. Overlapping windows merge and
/// keep the closer miss. Each window counts the `stations` inside it and
/// records the sign of `speed` at both edges; a count of the wrong parity,
/// or a window reaching the span's edge, is an error.
///
/// `extrema` must be in ascending order (the scan produces them that way).
pub fn graze_windows(
    speed: &dyn Fn(f64) -> f64,
    extrema: &[(f64, f64)],
    stations: &[Station],
    span: (f64, f64),
    grid: f64,
    margin: f64,
) -> Result<Vec<Window>, String> {
    let far = |jd: f64| speed(jd).abs() >= 2.0 * margin;
    let mut windows: Vec<Window> = Vec::new();
    for &(turn_jd, value) in extrema {
        if value.abs() >= margin {
            continue;
        }
        let (mut lo, mut hi) = (turn_jd, turn_jd);
        while !far(lo) {
            lo -= grid;
            if lo < span.0 {
                return Err(format!(
                    "graze window at jd_tt={turn_jd:.7} reaches the span's start"
                ));
            }
        }
        while !far(hi) {
            hi += grid;
            if hi > span.1 {
                return Err(format!(
                    "graze window at jd_tt={turn_jd:.7} reaches the span's end"
                ));
            }
        }
        let miss = value.abs();
        match windows.last_mut() {
            Some(last) if lo <= last.hi => {
                last.hi = last.hi.max(hi);
                if miss < last.miss_deg_per_day {
                    last.miss_deg_per_day = miss;
                    last.turn_jd = turn_jd;
                }
            }
            _ => windows.push(Window {
                turn_jd,
                miss_deg_per_day: miss,
                lo,
                hi,
                se_stations: 0,
                lo_side: 0,
                hi_side: 0,
            }),
        }
    }
    let side = |jd: f64| if speed(jd) > 0.0 { 1 } else { -1 };
    for w in &mut windows {
        w.se_stations = stations
            .iter()
            .filter(|s| w.lo <= s.jd && s.jd <= w.hi)
            .count();
        w.lo_side = side(w.lo);
        w.hi_side = side(w.hi);
        if (w.lo_side != w.hi_side) != (w.se_stations % 2 == 1) {
            return Err(format!(
                "graze window [{:.7}, {:.7}] holds {} stations, but its edge sides {},{} imply the other parity (count)",
                w.lo, w.hi, w.se_stations, w.lo_side, w.hi_side
            ));
        }
    }
    Ok(windows)
}

#[cfg(test)]
mod tests;
