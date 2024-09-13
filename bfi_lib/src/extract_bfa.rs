/** ------------------------------------------------------------
 * BFA extraction from bytestream payload
 * ------------------------------------------------------------- */
use crate::errors::BfaExtractionError;
use crate::he_mimo_ctrl::HeMimoControl;

#[rustfmt::skip]
pub struct ExtractionConfig {
	pub bitfield_pattern : Vec<u8>, // Length of bitfields per subcarrier-chunk
	pub num_subcarrier   : u16,     // Number of subcarriers
}

impl ExtractionConfig {
    pub fn from_he_mimo_ctrl(_mimo_ctrl: &HeMimoControl) -> Self {
        ExtractionConfig {
            bitfield_pattern: vec![6, 6, 6, 4, 4, 4, 6, 6, 4, 4],
            num_subcarrier: 62,
        }
    }
}

/**
 * Some sanity checks for the BFA bitfield extraction
 */
fn sanity_check_extraction(
    bitfield_pattern: &[u8],
    num_chunks: u16,
    byte_stream_len: usize,
) -> Result<(), BfaExtractionError> {
    // Find the number of bits per chunk
    let total_bits_per_chunk: usize = bitfield_pattern
        .iter()
        .map(|&bitsize| bitsize as usize)
        .sum();

    // Find the number of bits we expect present in the byte stream
    let total_bits_needed = total_bits_per_chunk * num_chunks as usize;

    // Ensure there are enough bits in the byte stream
    if byte_stream_len * 8 < total_bits_needed {
        return Err(BfaExtractionError::InsufficientBitsize {
            required: total_bits_needed,
            available: byte_stream_len * 8,
        });
    }

    // See below in extract_bitfields for an explanation.
    let max_allowed_bitsize = 9;
    if bitfield_pattern.iter().any(|&x| x > max_allowed_bitsize) {
        return Err(BfaExtractionError::InvalidBitfieldSize {
            given: *bitfield_pattern.iter().max().unwrap(),
            allowed: 9,
        });
    }

    Ok(())
}

/**
 * Extract bitfields from a pattern description
 *
 * ## Warning
 *
 * This function assumes that bfa_payload is at least of size 2.
 * This requirement is not tested, so it will panic if violated.
 *
 * ## Description
 *
 * This function runs through a stream of bytes and extracts bitfields.
 * To extract bits from LSB, we pre-shift new bytes' bitpattern to the
 * front and simply mask out the correct bits to extract.
 *
 * Also assumes that bitfield_pattern never contains a value greater
 * than 16.
 *
 */
fn extract_bitfields(
    byte_stream: &[u8],
    bitfield_pattern: Vec<u8>,
    num_chunks: u16,
) -> Result<Vec<Vec<u16>>, BfaExtractionError> {
    // Start with some sanity checks in debug mode. In release mode, we
    // leave them out for performance reasons. This will cause a crash in
    // API violations, but that's on you  ¯\_(ツ)_/¯
    #[cfg(debug_assertions)]
    sanity_check_extraction(bitfield_pattern.as_slice(), num_chunks, byte_stream.len())?;

    // --------------------------------------------------------------------------
    // Bit window processing:
    // We use a multi-byte integer as a sliding window over the byte stream to
    // extract bitfields. An index tracks the last processed bit. Since we shift
    // by 8 bits (1 byte) after processing, at most 7 bits can remain unprocessed
    // in the buffer. Therefore, to extract a bitfield of size N, the window must
    // be at least N+7 bits to handle the worst case. For BFI, the WiFi standard
    // specifies at most a bitsize of 9 for an angle, so a 16bit buffer suffices.
    let mut bit_window = u16::from_le_bytes([byte_stream[0], byte_stream[1]]);
    let mut window_offset = 0; // bit-offset pointing past last processed bit
    let mut curr_byte = 2; // stream offset past current window edge

    // Preallocate result vectors and bitmasks
    let mut result = Vec::with_capacity(num_chunks as usize);
    let mut chunk = Vec::with_capacity(bitfield_pattern.len());
    let masks: Vec<u16> = bitfield_pattern.iter().map(|&l| (1 << l) - 1).collect();

    for _ in 0..num_chunks {
        chunk.clear();
        for (i, &bit_length) in bitfield_pattern.iter().enumerate() {
            // If the to-be-processed bitfield is not completely within the
            // 16 bit, we need to advance the window.
            while window_offset + bit_length > 16 {
                // Shift in new byte from the left into window and advance
                let next_byte = byte_stream[curr_byte] as u16;
                bit_window = (bit_window >> 8) | (next_byte << 8);
                window_offset -= 8;
                curr_byte += 1;
            }

            // Extract the requested number of bits from the window (MSB first)
            let mask = masks[i];
            let bitfield = (bit_window >> window_offset) & mask;

            // Add the extracted bitfield to the chunk and advance pointer to
            // next bits in window to be processed.
            chunk.push(bitfield);
            window_offset += bit_length;
        }

        // Collect the chunk
        result.push(chunk.clone());
    }

    Ok(result)
}

/**
 * Extract BFA from payload using the corresponding extraction config
 */
pub fn extract_bfa(
    bfa_payload: &[u8],
    extraction_config: ExtractionConfig,
) -> Result<Vec<Vec<u16>>, BfaExtractionError> {
    extract_bitfields(
        bfa_payload,
        extraction_config.bitfield_pattern,
        extraction_config.num_subcarrier,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extraction() {
        // Example payload 11001010 11110000 01011100 00111110
        // Reverse:        01010011 00001111 00111010 01111100
        // Chunk:          010100 1100 0011 110011 1010 0111 (1100)
        // Reverse:        001010 0011 1100 110011 0101 1110
        let byte_stream: &[u8] = &[0b11001010, 0b11110000, 0b01011100, 0b00111110];
        let expected: Vec<Vec<u16>> = vec![
            vec![0b001010, 0b0011, 0b1100],
            vec![0b110011, 0b0101, 0b1110],
        ];
        let bitfield_pattern = vec![6, 4, 4]; // Example pattern (6 bits, 4 bits, 4 bits)
        let num_chunks = 2; // Example number of chunks

        let result = extract_bitfields(byte_stream, bitfield_pattern, num_chunks);
        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(
            result == expected,
            "Expected {:?}, but got: {:?}",
            expected,
            result
        );
    }

    #[test]
    fn test_large_bitfields() {
        // Example payload 11001010 11110000
        // Reverse:        01010011 00001111
        // Chunk:          010100110 00011 11
        // Reverse:        011001010 11000 11
        let byte_stream: &[u8] = &[0b11001010, 0b11110000];
        let expected: Vec<Vec<u16>> = vec![vec![0b011001010, 0b11000, 0b11]];
        let bitfield_pattern = vec![9, 5, 2]; // Example pattern (6 bits, 4 bits, 4 bits)
        let num_chunks = 1; // Example number of chunks

        let result = extract_bitfields(byte_stream, bitfield_pattern, num_chunks);
        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(
            result == expected,
            "Expected {:?}, but got: {:?}",
            expected,
            result
        );
    }

    #[test]
    fn test_todo_more() {
        // write a test that errors when wrong bitshift is used
    }

    #[test]
    fn capacity_error() {
        // Example payload 11001010 11110000 01011100 00111110
        // Reverse:        01010011 00001111 00111010 01111100
        // Chunk:          010100 1100 0011 110011 1010 0111 (1100)
        // Reverse:        001010 0011 1100 110011 0101 1110
        let byte_stream: &[u8] = &[0b11001010, 0b11110000];
        let bitfield_pattern = vec![6, 4, 4];
        let num_chunks = 2;

        // 2 chunks, each of size 14 bit -> exceeds payload of 16 bits

        let result = extract_bitfields(byte_stream, bitfield_pattern, num_chunks);
        if let Err(BfaExtractionError::InsufficientBitsize {
            required,
            available,
        }) = result
        {
            assert_eq!(required, 28);
            assert_eq!(available, 16);
        } else {
            assert!(false, "Expected InsufficientBitsize error");
        }
    }
}
