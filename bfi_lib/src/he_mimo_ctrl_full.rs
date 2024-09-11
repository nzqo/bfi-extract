use crate::bandwidth::Bandwidth;
/** ------------------------------------------------------------
 * Full_extract feature implementation
 * Extracting all the fields. Mainly for debug/testing purposes.
 * ------------------------------------------------------------- */
use crate::util::extract_bitfield;

/**
 * Full HE Mimo Control fields
 */
#[derive(Debug, Copy, Clone)]
pub struct HeMimoControl {
    pub num_streams: u8,
    pub num_antennae: u8,
    pub bandwidth: Bandwidth,
    pub grouping: u8,
    pub codebook_info: u8,
    pub feedback_type: u8,
    pub remaining_feedback_segments: u8,
    pub first_feedback_segments: u8,
    pub ru_start_index: u8,
    pub ru_end_index: u8,
    pub dialog_token_number: u8,
    pub reserved: u8,
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
            grouping: extract_bitfield(buffer, 8, 1),
            codebook_info: extract_bitfield(buffer, 9, 1),
            feedback_type: extract_bitfield(buffer, 10, 2),
            remaining_feedback_segments: extract_bitfield(buffer, 12, 3),
            first_feedback_segments: extract_bitfield(buffer, 15, 1),
            ru_start_index: extract_bitfield(buffer, 16, 7),
            ru_end_index: extract_bitfield(buffer, 23, 7),
            dialog_token_number: extract_bitfield(buffer, 30, 6),
            reserved: extract_bitfield(buffer, 36, 4),
        }
    }
}
