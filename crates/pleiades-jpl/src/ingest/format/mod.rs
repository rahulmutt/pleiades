//! Per-format tokenizing front-ends. Each converts bytes to the neutral IR.

pub mod generic_csv;
pub mod horizons_json;
pub mod vector_table;

use crate::ingest::detect::InputFormat;
use crate::ingest::error::IngestError;
use crate::ingest::ir::RawCorpus;

/// Routes raw text to the front-end for `format`.
pub fn parse_to_ir(format: InputFormat, source: &str) -> Result<RawCorpus, IngestError> {
    match format {
        InputFormat::HorizonsVectorTable => vector_table::parse(source),
        InputFormat::HorizonsApiJson => horizons_json::parse(source),
        InputFormat::GenericCsv => generic_csv::parse(source),
    }
}
