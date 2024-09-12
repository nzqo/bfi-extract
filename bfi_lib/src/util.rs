/** ------------------------------------------------------------
 * Library utility functions
 * ------------------------------------------------------------- */
pub fn get_u32_from_bytes(byte_stream: &[u8]) -> u32 {
    let mut buffer = [0u8; 4];
    let len = byte_stream.len().min(4);
    buffer[..len].copy_from_slice(&byte_stream[..len]);
    u32::from_ne_bytes(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32_byte_creation_from_small_array() {
        // Example payload 11001010 11110000
        // Reverse:        01010011 00001111
        // Chunk:          0101001100 0011 11
        // Reverse:        0011001010 1100 11
        let byte_stream: &[u8] = &[0b11001010, 0b11110000];

        let expected = u32::from_le_bytes([0b11001010, 0b11110000, 0, 0]);
        let result = get_u32_from_bytes(byte_stream);
        assert_eq!(result, expected);
    }

    #[test]
    fn u32_byte_creation_from_large_array() {
        // Example payload 11001010 11110000
        // Reverse:        01010011 00001111
        // Chunk:          0101001100 0011 11
        // Reverse:        0011001010 1100 11
        let byte_stream: &[u8] = &[1, 3, 4, 7, 8];

        let expected = u32::from_le_bytes([1, 3, 4, 7]);
        let result = get_u32_from_bytes(byte_stream);
        assert_eq!(result, expected);
    }
}
