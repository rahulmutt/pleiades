use super::*;

#[test]
fn j2000_equinox_is_identity() {
    let (x, y, z) = rotate_ecliptic_to_j2000(1.0, 2.0, 0.5, crate::J2000_JD);
    assert!((x - 1.0).abs() < 1.0e-12);
    assert!((y - 2.0).abs() < 1.0e-12);
    assert!((z - 0.5).abs() < 1.0e-12);
}

#[test]
fn precession_preserves_distance() {
    // Ecliptic precession is a pure rotation: |v| is invariant. J1900 = 2415020.0.
    let (x, y, z) = rotate_ecliptic_to_j2000(3.0, 0.0, 0.0, 2_415_020.0);
    let r = (x * x + y * y + z * z).sqrt();
    assert!((r - 3.0).abs() < 1.0e-9, "distance changed: r={r}");
}
