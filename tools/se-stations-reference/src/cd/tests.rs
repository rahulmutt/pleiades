use super::*;

/// A longitude whose speed is `cos(2π t / 10)`: stations every 5 days,
/// R at t = 2.5 + 10k, D at 7.5 + 10k (speed crosses zero falling, then rising).
fn wave(t: f64) -> f64 {
    10.0 / (2.0 * std::f64::consts::PI) * (2.0 * std::f64::consts::PI * t / 10.0).sin()
}

#[test]
fn scan_finds_alternating_stations_to_the_tolerance() {
    let s = scan(&wave, (0.0, 30.0), 0.25, 0.01);
    let jds: Vec<f64> = s.stations.iter().map(|st| st.jd).collect();
    let want = [2.5, 7.5, 12.5, 17.5, 22.5, 27.5];
    assert_eq!(jds.len(), want.len(), "{jds:?}");
    for (got, want) in jds.iter().zip(want) {
        assert!((got - want).abs() < 1e-5, "{got} vs {want}");
    }
    assert_eq!(s.stations[0].kind, Kind::R);
    assert_eq!(s.stations[1].kind, Kind::D);
    // The one-sided extrema are the wave's peaks, at speed +-1.
    assert!(!s.extrema.is_empty());
    assert!(
        s.extrema.iter().all(|(_, v)| v.abs() > 0.99),
        "{:?}",
        s.extrema
    );
}

/// The wave plus an isolated 0.7" jump in longitude at `jump` only.
fn jumpy(jump: f64) -> impl Fn(f64) -> f64 {
    move |t: f64| {
        // A constant speed of -0.001 deg/day near zero, so a jump of the
        // central difference flips its sign.
        let base = -0.001 * t;
        if (t - jump).abs() < 1e-9 {
            base + 0.7 / 3600.0
        } else {
            base
        }
    }
}

#[test]
fn an_isolated_jump_gives_a_dropped_pair() {
    let h = 0.001;
    // Put the jump where the h scan's grid steps across t ± h.
    let span = (0.0, 2.0);
    let jump = span.0 + GRID_OFFSET_DAYS + 50.0 * 0.01 + h;
    let lon = jumpy(jump);
    let a = scan(&lon, span, 0.01, h);
    let b = scan(&lon, span, 0.01, CHECK_STEP_RATIO * h);
    assert!(
        !a.stations.is_empty(),
        "the jump makes a pair in the h scan"
    );
    let (kept, dropped) = reject_artefacts(&a.stations, &b.stations, h).unwrap();
    assert!(kept.is_empty(), "{kept:?}");
    assert_eq!(dropped.len(), a.stations.len() / 2);
    for [first, second] in dropped {
        assert_ne!(first.kind, second.kind);
        assert!(second.jd - first.jd < 3.0 * h);
    }
}

#[test]
fn close_real_stations_survive_both_scans() {
    // Two real stations 0.04 day (about an hour) apart: the speed is
    // (t - 1)(t - 1.04), so it turns retrograde at 1.0 and direct at 1.04.
    let lon = |t: f64| t.powi(3) / 3.0 - 1.02 * t * t + 1.04 * t;
    let a = scan(&lon, (0.0, 2.0), 0.01, 0.001);
    let b = scan(&lon, (0.0, 2.0), 0.01, 0.0013);
    let (kept, dropped) = reject_artefacts(&a.stations, &b.stations, 0.001).unwrap();
    assert!(dropped.is_empty());
    assert_eq!(kept, a.stations);
    assert_eq!(
        kept.iter().map(|s| s.kind).collect::<Vec<_>>(),
        [Kind::R, Kind::D]
    );
    assert!((kept[1].jd - kept[0].jd - 0.04).abs() < 1e-4);
}

#[test]
fn a_lone_unmatched_station_aborts() {
    let a = [Station {
        jd: 10.0,
        kind: Kind::R,
    }];
    let b: [Station; 0] = [];
    let err = reject_artefacts(&a, &b, 0.001).unwrap_err();
    assert!(err.contains("10.0"), "{err}");
}

#[test]
fn a_mismatched_kind_is_not_a_match() {
    let a = [Station {
        jd: 10.0,
        kind: Kind::R,
    }];
    let b = [Station {
        jd: 10.0,
        kind: Kind::D,
    }];
    assert!(reject_artefacts(&a, &b, 0.001).is_err());
}

/// A speed `v(t) = (t − 5)^2 / 10 + floor`: one extremum at t = 5.
fn dip(floor: f64) -> impl Fn(f64) -> f64 {
    move |t: f64| (t - 5.0).powi(2) / 10.0 + floor
}

#[test]
fn a_shallow_dip_becomes_a_window_with_both_sides_above() {
    let speed = dip(0.001);
    let w = graze_windows(&speed, &[(5.0, 0.001)], &[], (0.0, 10.0), 0.01, 0.002).unwrap();
    assert_eq!(w.len(), 1);
    let w = w[0];
    assert_eq!((w.se_stations, w.lo_side, w.hi_side), (0, 1, 1));
    assert!(speed(w.lo) >= 0.004 && speed(w.hi) >= 0.004);
    assert!(speed(w.lo + 0.01) < 0.004 && speed(w.hi - 0.01) < 0.004);
    assert_eq!(w.miss_deg_per_day, 0.001);
}

#[test]
fn a_deep_extremum_is_not_a_window() {
    let w = graze_windows(&dip(0.01), &[(5.0, 0.01)], &[], (0.0, 10.0), 0.01, 0.002).unwrap();
    assert!(w.is_empty());
}

#[test]
fn a_window_whose_count_contradicts_its_sides_aborts() {
    // Both sides above zero, but one station recorded inside.
    let inside = [Station {
        jd: 5.0,
        kind: Kind::R,
    }];
    let err = graze_windows(
        &dip(0.001),
        &[(5.0, 0.001)],
        &inside,
        (0.0, 10.0),
        0.01,
        0.002,
    )
    .unwrap_err();
    assert!(err.contains("parity") || err.contains("count"), "{err}");
}

#[test]
fn a_window_reaching_the_span_edge_aborts() {
    assert!(graze_windows(&dip(0.001), &[(5.0, 0.001)], &[], (4.9, 5.1), 0.01, 0.002).is_err());
}

#[test]
fn overlapping_windows_merge_and_keep_the_closer_miss() {
    let speed = |t: f64| ((t - 5.0).powi(2) - 0.0001).abs() / 10.0 + 0.0005;
    let extrema = [(4.99, 0.0008), (5.01, 0.0005)];
    let w = graze_windows(&speed, &extrema, &[], (0.0, 10.0), 0.01, 0.002).unwrap();
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].miss_deg_per_day, 0.0005);
    assert_eq!(w[0].turn_jd, 5.01);
}
