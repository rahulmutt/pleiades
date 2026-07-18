#![no_main]

use libfuzzer_sys::fuzz_target;
use pleiades_jpl::SpkBackend;

// Oracle: no panic, no UB, no hang. Rejecting malformed kernel bytes with an
// SpkError is success; crashing or looping on them is the failure this target
// exists to catch. Threat model boundary #1 (kernel/corpus loading).
fuzz_target!(|data: &[u8]| {
    let _ = SpkBackend::builder().add_kernel_bytes(data.to_vec(), "fuzz");
});
