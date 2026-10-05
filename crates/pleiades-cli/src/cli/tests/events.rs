//! Tests for the `stations` and `aspects` commands (issues #167 (c), #168 (e)
//! and the CLI half of #165): ranges, next/previous, civil and Julian-day
//! instants, ordering, and argument errors.

use crate::cli::render_cli;

const J2000: &str = "2451545.0";
/// 366 days after J2000.
const END_OF_2000: &str = "2451911.0";

fn run(args: &[&str]) -> String {
    render_cli(args).unwrap_or_else(|error| panic!("{args:?}: {error}"))
}

fn error(args: &[&str]) -> String {
    render_cli(args).expect_err("the command should be rejected")
}

/// The event lines of a rendering: everything after the header line.
fn events(rendered: &str) -> Vec<&str> {
    rendered.lines().skip(1).collect()
}

/// The TDB Julian day printed on an event line.
fn julian_day(line: &str) -> f64 {
    let (_, tail) = line.split_once("JD ").expect("a JD field");
    let (jd, _) = tail.split_once(' ').expect("a scale after the JD");
    jd.parse().expect("a numeric JD")
}

#[test]
fn stations_lists_a_year_of_mercury_stations_with_civil_times() {
    let rendered = run(&[
        "stations",
        "--body",
        "Mercury",
        "--from",
        J2000,
        "--to",
        END_OF_2000,
    ]);
    assert!(rendered.starts_with("Stations"), "{rendered}");
    let lines = events(&rendered);
    assert_eq!(lines.len(), 6, "{rendered}");
    // Mercury turned retrograde on 21 February 2000.
    assert!(lines[0].starts_with("2000-02-21T"), "{rendered}");
    assert!(lines[0].contains(" UTC  JD 24515"), "{rendered}");
    assert!(
        lines[0].contains(" TDB  Mercury turns retrograde at "),
        "{rendered}"
    );
    assert!(lines[1].contains("Mercury turns direct at "), "{rendered}");
}

#[test]
fn a_civil_instant_is_read_as_utc() {
    // 2000-01-01T12:00:00 UTC is 64.184 s before JD 2451545.0 TDB: the same
    // six stations either way.
    let by_jd = run(&[
        "stations",
        "--body",
        "Mercury",
        "--from",
        J2000,
        "--to",
        END_OF_2000,
    ]);
    let by_civil = run(&[
        "stations",
        "--body",
        "Mercury",
        "--from",
        "2000-01-01T12:00:00",
        "--to",
        "2001-01-01T12:00:00",
    ]);
    // The two scans start 64 s apart, so each station is located on a
    // different grid: the same event, within the 0.5 s search tolerance.
    let (by_jd, by_civil) = (events(&by_jd), events(&by_civil));
    assert_eq!(by_jd.len(), 6);
    assert_eq!(by_civil.len(), 6);
    for (a, b) in by_jd.iter().zip(&by_civil) {
        assert!((julian_day(a) - julian_day(b)).abs() < 1e-5, "{a} vs {b}");
        let what = |line: &str| line.split_once(" TDB  ").expect("an event").1.to_string();
        assert_eq!(what(a), what(b));
    }
}

#[test]
fn next_and_previous_give_one_station_each_side_of_the_instant() {
    let next = run(&["stations", "--body", "Mercury", "--next", "--at", J2000]);
    let lines = events(&next);
    assert_eq!(lines.len(), 1, "{next}");
    assert!(lines[0].starts_with("2000-02-21T"), "{next}");

    let previous = run(&["stations", "--body", "Mercury", "--previous", "--at", J2000]);
    let lines = events(&previous);
    assert_eq!(lines.len(), 1, "{previous}");
    // Mercury turned direct on 25 November 1999.
    assert!(lines[0].starts_with("1999-11-25T"), "{previous}");
    assert!(lines[0].contains("turns direct"), "{previous}");
}

#[test]
fn several_bodies_are_merged_in_time_order() {
    let rendered = run(&[
        "stations",
        "--body",
        "Mercury",
        "--body",
        "Jupiter",
        "--body",
        "Saturn",
        "--from",
        J2000,
        "--to",
        END_OF_2000,
    ]);
    let lines = events(&rendered);
    assert!(
        lines.iter().any(|line| line.contains("Jupiter turns")),
        "{rendered}"
    );
    assert!(
        lines.iter().any(|line| line.contains("Saturn turns")),
        "{rendered}"
    );
    let days: Vec<f64> = lines.iter().map(|line| julian_day(line)).collect();
    assert!(days.windows(2).all(|pair| pair[0] <= pair[1]), "{rendered}");
}

#[test]
fn a_body_that_never_stations_says_so() {
    let rendered = run(&[
        "stations",
        "--body",
        "Sun",
        "--from",
        J2000,
        "--to",
        END_OF_2000,
    ]);
    assert_eq!(events(&rendered), ["no stations found"]);
}

#[test]
fn an_event_before_1972_is_printed_in_ut1() {
    // JD 2430000.0 is in January 1941.
    let rendered = run(&[
        "stations",
        "--body",
        "Mercury",
        "--next",
        "--at",
        "2430000.0",
    ]);
    let lines = events(&rendered);
    assert!(lines[0].starts_with("1941-"), "{rendered}");
    assert!(lines[0].contains(" UT1  JD 2430"), "{rendered}");
}

#[test]
fn a_sidereal_zodiac_and_a_frame_are_accepted() {
    let rendered = run(&[
        "stations",
        "--body",
        "Mercury",
        "--ayanamsa",
        "Lahiri",
        "--frame",
        "mean",
        "--from",
        J2000,
        "--to",
        END_OF_2000,
    ]);
    assert!(
        rendered.lines().next().unwrap().contains("Lahiri"),
        "{rendered}"
    );
    assert_eq!(events(&rendered).len(), 6, "{rendered}");
    // Nothing stations seen from the Sun.
    let helio = run(&[
        "stations",
        "--body",
        "Mercury",
        "--frame",
        "helio",
        "--from",
        J2000,
        "--to",
        END_OF_2000,
    ]);
    assert_eq!(events(&helio), ["no stations found"]);
}

#[test]
fn aspects_finds_the_great_conjunction_of_2000() {
    let rendered = run(&[
        "aspects",
        "--pair",
        "Jupiter,Saturn",
        "--angle",
        "0",
        "--from",
        J2000,
        "--to",
        END_OF_2000,
    ]);
    assert!(rendered.starts_with("Aspects"), "{rendered}");
    let lines = events(&rendered);
    assert_eq!(lines.len(), 1, "{rendered}");
    assert!(lines[0].starts_with("2000-05-28T"), "{rendered}");
    assert!(lines[0].contains(" TDB  Jupiter–Saturn 0° ("), "{rendered}");
}

#[test]
fn aspects_loops_over_pairs_and_angles_in_time_order() {
    // Thirty days hold one new Moon and one full Moon, and the Moon passes
    // Mars once.
    let rendered = run(&[
        "aspects",
        "--pair",
        "Sun,Moon",
        "--pair",
        "Moon,Mars",
        "--angle",
        "0",
        "--angle",
        "180",
        "--from",
        J2000,
        "--to",
        "2451575.0",
    ]);
    let lines = events(&rendered);
    assert!(
        lines.iter().any(|line| line.contains("Sun–Moon 0° (")),
        "{rendered}"
    );
    assert!(
        lines.iter().any(|line| line.contains("Sun–Moon 180° (")),
        "{rendered}"
    );
    assert!(
        lines.iter().any(|line| line.contains("Moon–Mars 0° (")),
        "{rendered}"
    );
    let days: Vec<f64> = lines.iter().map(|line| julian_day(line)).collect();
    assert!(days.windows(2).all(|pair| pair[0] <= pair[1]), "{rendered}");
}

#[test]
fn aspects_next_and_previous_give_one_event_per_pair_and_angle() {
    let next = run(&[
        "aspects", "--pair", "Sun,Moon", "--angle", "180", "--next", "--at", J2000,
    ]);
    assert_eq!(events(&next).len(), 1, "{next}");
    let previous = run(&[
        "aspects",
        "--pair",
        "Sun,Moon",
        "--angle",
        "180",
        "--previous",
        "--at",
        J2000,
    ]);
    // The last full Moon before J2000 was on 22 December 1999.
    assert!(
        events(&previous)[0].starts_with("1999-12-22T"),
        "{previous}"
    );
}

#[test]
fn the_search_window_must_be_one_range_or_one_direction() {
    let stations = |tail: &[&str]| {
        let mut args = vec!["stations", "--body", "Mercury"];
        args.extend_from_slice(tail);
        error(&args)
    };
    assert!(
        stations(&[]).contains("--from and --to"),
        "{}",
        stations(&[])
    );
    assert!(stations(&["--from", J2000]).contains("--to"));
    assert!(stations(&["--next"]).contains("--at"));
    assert!(stations(&["--at", J2000]).contains("--next or --previous"));
    assert!(stations(&["--next", "--previous", "--at", J2000]).contains("only one of"));
    assert!(stations(&[
        "--next",
        "--at",
        J2000,
        "--from",
        J2000,
        "--to",
        END_OF_2000
    ])
    .contains("cannot be combined"));
}

#[test]
fn bad_arguments_are_rejected_with_the_flag_named() {
    assert!(error(&["stations", "--from", J2000, "--to", END_OF_2000]).contains("--body"));
    assert!(error(&[
        "aspects",
        "--angle",
        "0",
        "--from",
        J2000,
        "--to",
        END_OF_2000
    ])
    .contains("--pair"));
    assert!(error(&[
        "aspects",
        "--pair",
        "Sun,Moon",
        "--from",
        J2000,
        "--to",
        END_OF_2000
    ])
    .contains("--angle"));
    assert!(
        error(&["aspects", "--pair", "Sun", "--angle", "0", "--from", J2000, "--to", J2000])
            .contains("--pair")
    );
    assert!(error(&[
        "stations", "--body", "Mercury", "--frame", "sideways", "--next", "--at", J2000
    ])
    .contains("--frame"));
    assert!(error(&[
        "stations",
        "--body",
        "Mercury",
        "--from",
        "yesterday",
        "--to",
        J2000
    ])
    .contains("--from"));
    assert_eq!(
        error(&["stations", "--body", "Mercury", "--loudly"]),
        "unknown argument: --loudly"
    );
    // An engine error is reported, not swallowed: 200 degrees is no separation.
    assert!(
        error(&["aspects", "--pair", "Sun,Moon", "--angle", "200", "--next", "--at", J2000])
            .contains("between 0 and 180")
    );
}

#[test]
fn help_lists_the_event_commands() {
    let help = run(&["help"]);
    assert!(help.contains("  stations "), "{help}");
    assert!(help.contains("  aspects "), "{help}");
    assert!(run(&["stations", "--help"]).contains("Usage:\n  stations "));
    assert!(run(&["aspects", "--help"]).contains("Usage:\n  aspects "));
}
