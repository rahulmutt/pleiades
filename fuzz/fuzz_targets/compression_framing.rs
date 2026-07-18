#![no_main]

use libfuzzer_sys::fuzz_target;
use pleiades_compression::CompressedArtifact;

// Fuzzes the framing exactly as shipped: magic, version, checksum verification
// and trailing-byte rejection. Most inputs are rejected at the checksum — that
// is the point of this target, and why compression_payload exists separately
// rather than this one being tuned to get past the hash.
fuzz_target!(|data: &[u8]| {
    let _ = CompressedArtifact::decode(data);
});
