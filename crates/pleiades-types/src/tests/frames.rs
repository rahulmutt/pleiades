use crate::*;

#[test]
fn coordinate_frames_have_stable_display_names() {
    assert_eq!(CoordinateFrame::Ecliptic.to_string(), "Ecliptic");
    assert_eq!(CoordinateFrame::Equatorial.to_string(), "Equatorial");
}
