#![no_main]

use libfuzzer_sys::fuzz_target;
use pleiades_backend::{CelestialBody, EphemerisBackend, EphemerisRequest};
use pleiades_jpl::SpkBackend;
use pleiades_types::{Instant, JulianDay, TimeScale};

// Oracle: no panic, no UB, no hang, no allocation sized by an unchecked file
// value. Where `spk_kernel` stops at DAF container parsing, this target goes
// one call deeper: a kernel that loads is queried through the public
// `position()`, which walks the segment chain and decodes Chebyshev (Type 2/3)
// and modified-difference (Type 1/21) records from the fuzzer's bytes.
// Returning an error is success. Threat model boundary #1 (kernel loading).
const BODIES: [CelestialBody; 4] = [
    CelestialBody::Sun,
    CelestialBody::Moon,
    CelestialBody::Mars,
    CelestialBody::Ceres,
];

// J2000.0, a modern epoch, and one far outside any real kernel's coverage.
const EPOCHS_JD: [f64; 3] = [2_451_545.0, 2_460_000.5, 1.0e9];

fuzz_target!(|data: &[u8]| {
    let Ok(builder) = SpkBackend::builder().add_kernel_bytes(data.to_vec(), "fuzz") else {
        return;
    };
    let backend = builder.build();
    for body in BODIES {
        for jd in EPOCHS_JD {
            let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
            let _ = backend.position(&EphemerisRequest::new(body.clone(), instant));
        }
    }
});
