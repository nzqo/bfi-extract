use crate::bandwidth::Bandwidth;
/** ------------------------------------------------------------
 * Base feature implementation
 * Used in "deployment", where we don't want to do anything not
 * strictly necessary that could impact performance
 * ------------------------------------------------------------- */
use crate::util::extract_bitfield;

/**
 * Minimal He Mimo Control extraction parameters
 *
 * Here we are extracting only the parameters required to decode BFA
 * angles. A full extraction is not needed, so we save that memory &
 * processing time.
 */
#[derive(Debug, Copy, Clone)]
pub struct HeMimoControl {
    pub num_streams: u8,
    pub num_antennae: u8,
    pub bandwidth: Bandwidth,
    pub codebook_info: u8,
    pub feedback_type: u8,
    pub dialog_token_number: u8,
}

/**
 * Conversion from payload byte array
 */
impl HeMimoControl {
    pub fn from_bytes(data: &[u8]) -> Self {
        let buffer = u64::from_le_bytes(data[0..8].try_into().unwrap());

        Self {
            num_streams: extract_bitfield(buffer, 0, 3) + 1,
            num_antennae: extract_bitfield(buffer, 3, 3) + 1,
            bandwidth: Bandwidth::from_code(extract_bitfield(buffer, 6, 2)),
            codebook_info: extract_bitfield(buffer, 9, 1),
            feedback_type: extract_bitfield(buffer, 10, 2),
            dialog_token_number: extract_bitfield(buffer, 30, 6),
        }
    }
}
