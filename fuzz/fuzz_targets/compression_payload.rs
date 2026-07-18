#![no_main]

use libfuzzer_sys::fuzz_target;
use pleiades_compression::{CompressedArtifact, ARTIFACT_VERSION};

// ARTIFACT_MAGIC and fnv1a64 are pub(crate) in pleiades-compression. Rather
// than widen production visibility or add a #[cfg(fuzzing)] hook, the harness
// carries its own copies: the magic is a fixed 8-byte literal and FNV-1a is a
// five-line standard algorithm. If either ever changes in the library, this
// target stops reaching the codec and the framing target still covers the
// public path — a silent-coverage-loss risk accepted deliberately, and pinned
// by the assertion in the smoke-test step of this task.
const ARTIFACT_MAGIC: [u8; 8] = *b"PLDEPHEM";

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

// The fuzzer's bytes are treated as the artifact PAYLOAD. The harness wraps
// them in valid framing with a correct checksum, so the mutator's work lands
// inside the codec instead of dying at the hash. decode() itself is unmodified
// and still the real public entry point.
fuzz_target!(|payload: &[u8]| {
    let mut framed = Vec::with_capacity(payload.len() + 18);
    framed.extend_from_slice(&ARTIFACT_MAGIC);
    framed.extend_from_slice(&ARTIFACT_VERSION.to_le_bytes());
    framed.extend_from_slice(&fnv1a64(payload).to_le_bytes());
    framed.extend_from_slice(payload);

    let _ = CompressedArtifact::decode(&framed);
});
