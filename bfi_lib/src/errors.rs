/** ------------------------------------------------------------
 * Error types raised by this lib.
 * ------------------------------------------------------------- */
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum BfaExtractionError {
    #[error("Received buffer of insufficient bit number: {available} (required: {required})")]
    InsufficientBitsize { required: usize, available: usize },
    #[error("Bitsize {given} exceeds maximum handled bitsize of {allowed}")]
    InvalidBitfieldSize { given: u8, allowed: u8 },
}
