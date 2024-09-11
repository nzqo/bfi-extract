use crate::errors::BfaExtractionError;
use crate::he_mimo_ctrl::HeMimoControl;
use crate::util::get_u32_from_bytes;

#[rustfmt::skip]
pub struct ExtractionConfig {
	pub bitfield_pattern : Vec<u8>, // Length of bitfields per subcarrier-chunk
	pub num_subcarrier   : u16,     // Number of subcarriers
}

impl ExtractionConfig {
    pub fn from_he_mimo_ctrl(_mimo_ctrl: HeMimoControl) -> Self {
        ExtractionConfig {
            bitfield_pattern: vec![6, 6, 6, 4, 4, 4, 6, 6, 4, 4],
            num_subcarrier: 62,
        }
    }
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
    // Find the number of bits per chunk
    let total_bits_per_chunk: usize = bitfield_pattern
        .iter()
        .map(|&bitsize| bitsize as usize)
        .sum();

    // Find the number of bits we expect present in the byte stream
    let total_bits_needed = total_bits_per_chunk * num_chunks as usize;

    // Ensure there are enough bits in the byte stream
    if byte_stream.len() * 8 < total_bits_needed {
        return Err(BfaExtractionError::InsufficientBitsize {
            required: total_bits_needed,
            available: byte_stream.len() * 8,
        });
    }

    // Bit window:
    // To extract bit-fields of at most size 8, we keep a window to slide
    // through 2 bytes of the byte stream at a time. Within the window, we
    // need a bit offset to look at. After reading N bits, we advance the
    // offset by N. After processing a full byte, we advance the window.
    let mut bit_window = get_u32_from_bytes(byte_stream);
    let mut window_offset = 0; // Number of valid bits in the bit window
    let mut curr_byte = 2; // Tracks the current bit position in the byte stream

    // Vector to store results in
    let mut result = Vec::with_capacity(num_chunks as usize);

    for _ in 0..num_chunks {
        let mut chunk = Vec::with_capacity(bitfield_pattern.len());

        for &bit_length in &bitfield_pattern {
            // If the to-be-processed bitfield is not completely within the
            // 16 bit, we need to advance the window.
            while window_offset + bit_length > 32 {
                // Shift in new byte from the left into window and advance
                let next_byte = byte_stream[curr_byte] as u32;
                bit_window = (bit_window >> 8) | (next_byte << (32 - 8));
                window_offset -= 8;
                curr_byte += 1;
            }

            // Extract the requested number of bits from the window (MSB first)
            let mask = (1 << bit_length) - 1 as u32;
            let bitfield = (bit_window >> window_offset) as u32 & mask;

            // Add the extracted bitfield to the chunk
            chunk.push(bitfield as u16);

            // Clear the extracted bits from the window
            window_offset += bit_length;
        }

        // Push the chunk (representing the bitfields) to the result
        result.push(chunk);
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
        // Chunk:          0101001100 0011 11
        // Reverse:        0011001010 1100 11
        let byte_stream: &[u8] = &[0b11001010, 0b11110000];
        let expected: Vec<Vec<u16>> = vec![vec![0b0011001010, 0b1100, 0b11]];
        let bitfield_pattern = vec![10, 4, 2]; // Example pattern (6 bits, 4 bits, 4 bits)
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
