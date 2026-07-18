#![no_main]

use libfuzzer_sys::fuzz_target;
use pleiades_jpl::ingest::{read_public_corpus, ExpectedProfile};

// One call covers format detection plus all three front-ends (Horizons vector
// table, Horizons API JSON, generic CSV) plus the normalizer. The JSON path is
// the most interesting surface: a hand-rolled scanner extracts the `result`
// field and feeds it back into vector_table::parse, so the two parsers nest.
//
// A pre-design audit found no panic paths here, so this is a regression guard
// on threat model boundary #1 rather than a bug hunt.
fuzz_target!(|data: &[u8]| {
    let _ = read_public_corpus(data, &ExpectedProfile::default());
});
