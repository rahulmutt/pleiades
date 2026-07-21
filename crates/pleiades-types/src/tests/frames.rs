use crate::*;

#[test]
fn coordinate_frames_have_stable_display_names() {
    assert_eq!(CoordinateFrame::Ecliptic.to_string(), "Ecliptic");
    assert_eq!(CoordinateFrame::Equatorial.to_string(), "Equatorial");
}

#[test]
fn apparentness_displays_stable_labels() {
    assert_eq!(Apparentness::Apparent.to_string(), "Apparent");
    assert_eq!(Apparentness::Mean.to_string(), "Mean");
}
