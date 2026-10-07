//! `refine_itp` keeps `bisect`'s contract (the settled later end of a
//! bracket no wider than the tolerance) in fewer evaluations on smooth
//! residuals, and never more than one beyond bisection's on any residual.

use super::*;

const TOL: f64 = REFINE_TOLERANCE_DAYS;

/// An hour, the rise/set scanner's grid step and so its widest bracket.
const HOUR: f64 = 1.0 / 24.0;

/// A refiner over a residual and a bracket, as `itp` and `bisection` adapt.
type Refiner =
    dyn Fn(&mut dyn FnMut(f64) -> Result<f64, EventError>, f64, f64) -> Result<f64, EventError>;

/// Runs a refiner on `g` over `[lo, hi]` and returns its answer and how many
/// times it evaluated `g`.
fn counted(refiner: &Refiner, g: &dyn Fn(f64) -> f64, lo: f64, hi: f64) -> (f64, usize) {
    let mut calls = 0;
    let mut f = |t: f64| {
        calls += 1;
        Ok(g(t))
    };
    let root = refiner(&mut f, lo, hi).unwrap();
    (root, calls)
}

// `refine_itp` and `bisect` take a sized `F`; `&mut dyn FnMut` is one.
fn itp(
    mut f: &mut dyn FnMut(f64) -> Result<f64, EventError>,
    lo: f64,
    hi: f64,
) -> Result<f64, EventError> {
    let (f_lo, f_hi) = (f(lo)?, f(hi)?);
    refine_itp(&mut f, lo, f_lo, hi, f_hi)
}

fn bisection(
    mut f: &mut dyn FnMut(f64) -> Result<f64, EventError>,
    lo: f64,
    hi: f64,
) -> Result<f64, EventError> {
    let f_lo = f(lo)?;
    f(hi)?;
    bisect(&mut f, lo, f_lo, hi)
}

/// The answer is settled: `g` carries the post-crossing sign there and the
/// pre-crossing sign one tolerance earlier.
fn assert_settled(g: &dyn Fn(f64) -> f64, ascending: bool, got: f64) {
    let (before, at) = (g(got - TOL), g(got));
    if ascending {
        assert!(
            before <= 0.0 && at > 0.0,
            "not settled at {got}: {before}, {at}"
        );
    } else {
        assert!(
            before > 0.0 && at <= 0.0,
            "not settled at {got}: {before}, {at}"
        );
    }
}

const T0: f64 = 2_460_827.0;

#[test]
fn a_linear_residual_settles_in_a_few_evaluations() {
    let root = T0 + 0.37 * HOUR;
    let g = move |t: f64| t - root;
    let (got, calls) = counted(&itp, &g, T0, T0 + HOUR);
    let (_, bisect_calls) = counted(&bisection, &g, T0, T0 + HOUR);
    assert_settled(&g, true, got);
    // With kappa1 = 0.2 / width, the first truncated step lands a fifth of
    // the bracket past the regula-falsi point, and the bracket then shrinks
    // quadratically. Measured: 6 evaluations past the ends against
    // bisection's 13.
    assert!(calls - 2 <= 6, "{} evaluations past the ends", calls - 2);
    assert!(
        calls < bisect_calls,
        "{calls} against bisection's {bisect_calls}"
    );
}

#[test]
fn a_sinusoidal_residual_settles_well_under_bisection() {
    // An altitude-like residual: a day-period sine with its zero inside an
    // hour bracket, rising.
    let root = T0 + 0.61 * HOUR;
    let g = move |t: f64| (std::f64::consts::TAU * (t - root)).sin();
    let (got, calls) = counted(&itp, &g, T0, T0 + HOUR);
    let (_, bisect_calls) = counted(&bisection, &g, T0, T0 + HOUR);
    assert_settled(&g, true, got);
    assert!(calls - 2 <= 6, "{} evaluations past the ends", calls - 2);
    assert!(
        calls < bisect_calls,
        "{calls} against bisection's {bisect_calls}"
    );
}

#[test]
fn a_descending_residual_settles_on_its_post_crossing_side() {
    let root = T0 + 0.2 * HOUR;
    let g = move |t: f64| root - t;
    let (got, _) = counted(&itp, &g, T0, T0 + HOUR);
    assert_settled(&g, false, got);
}

#[test]
fn step_and_cubic_residuals_cost_at_most_one_more_than_bisection() {
    let root = T0 + 0.731 * HOUR;
    let step = move |t: f64| if t < root { -1.0 } else { 1.0 };
    let cubic = move |t: f64| ((t - root) * 24.0).powi(3) * 1e6;
    let flat_then_steep = move |t: f64| {
        if t < root {
            -1e-12
        } else {
            1e3 * (t - root) + 1e-12
        }
    };
    for (label, g) in [
        ("step", &step as &dyn Fn(f64) -> f64),
        ("cubic", &cubic),
        ("flat then steep", &flat_then_steep),
    ] {
        let (got, calls) = counted(&itp, g, T0, T0 + HOUR);
        let (_, bisect_calls) = counted(&bisection, g, T0, T0 + HOUR);
        assert_settled(g, true, got);
        assert!(
            calls <= bisect_calls + 1,
            "{label}: {calls} against bisection's {bisect_calls}"
        );
    }
}

#[test]
fn a_root_at_the_earlier_end_settles_within_a_tolerance_of_it() {
    // `f(lo) == 0` counts as the pre-crossing sign, as in `bisect`.
    let g = |t: f64| t - T0;
    let (got, _) = counted(&itp, &g, T0, T0 + HOUR);
    assert!(got > T0 && got - T0 <= TOL, "{}", got - T0);
}

#[test]
fn a_root_just_before_the_later_end_settles_within_a_tolerance_of_it() {
    let root = T0 + HOUR - 1e-9;
    let g = move |t: f64| t - root;
    let (got, _) = counted(&itp, &g, T0, T0 + HOUR);
    assert_settled(&g, true, got);
}

#[test]
fn a_bracket_already_within_tolerance_is_returned_without_evaluating() {
    let mut calls = 0;
    let mut f = |_t: f64| {
        calls += 1;
        Ok(0.0)
    };
    let got = refine_itp(&mut f, T0, -1.0, T0 + 0.4 * TOL, 1.0).unwrap();
    assert_eq!(got, T0 + 0.4 * TOL);
    assert_eq!(calls, 0);
}

#[test]
fn itp_and_bisection_agree_to_the_tolerance() {
    for k in 0..200_u32 {
        let root = T0 + (f64::from(k) + 0.5) / 200.0 * HOUR;
        let g = move |t: f64| (std::f64::consts::TAU * (t - root)).sin();
        let (a, _) = counted(&itp, &g, T0, T0 + HOUR);
        let (b, _) = counted(&bisection, &g, T0, T0 + HOUR);
        assert!(
            (a - b).abs() <= TOL,
            "root {k}: {} s apart",
            (a - b).abs() * 86_400.0
        );
    }
}

#[test]
fn an_error_from_the_residual_propagates() {
    let mut f = |_t: f64| Err(EventError::OutOfWindow { julian_day: T0 });
    assert!(matches!(
        refine_itp(&mut f, T0, -1.0, T0 + HOUR, 1.0),
        Err(EventError::OutOfWindow { .. })
    ));
}
