//use core::panicking::panic;
/** ------------------------------------------------------------
 * BFA extraction from bytestream payload
 * ------------------------------------------------------------- */
use crate::errors::BfaExtractionError;
use crate::he_mimo_ctrl::HeMimoControl;
use crate::he_mimo_ctrl::Bandwidth;

#[rustfmt::skip]

pub struct ExtractionConfig {
	pub bitfield_pattern : Vec<u8>, // Length of bitfields per subcarrier-chunk
	pub num_subcarrier   : u16,     // Number of subcarriers
}
pub struct  PhiPsiBit{
    pub phi_bit:u8,         // Phi vit value 
    pub psi_bit:u8,         // Psi bit value 
}


// Define constant patterns //TODO check values again 
const PATTERN_2_1: [&str; 2] =  ["phi", "psi"]; // same pattrn for 2x2
const PATTERN_3_1: [&str; 4] =  ["phi", "phi", "psi", "psi"];
const PATTERN_3_2: [&str; 6] =  ["phi", "phi", "psi", "psi", "phi", "psi"]; // same pattrn for 3x3
const PATTERN_4_1: [&str; 6] =  ["phi", "phi", "phi", "psi", "psi","psi"];
const PATTERN_4_2: [&str; 10] = ["phi","phi","phi","psi","psi","psi","phi","phi","psi","psi",]; 
const PATTERN_4_3: [&str; 12] = ["phi","phi","phi","psi","psi","psi","phi","phi","psi","psi","phi","psi",]; // same pattrn for 4x4

impl ExtractionConfig {
    pub fn from_he_mimo_ctrl(mimo_ctrl: &HeMimoControl) -> Self {
        println!("Codebook Information: {}", mimo_ctrl.codebook_info());
        println!("Feedback Type: {}", mimo_ctrl.feedback_type());
        println!("NC : {}", mimo_ctrl.nc_index());
        println!("NR : {}", mimo_ctrl.nr_index());

        /*
        * ******************* derive PhiPsiBit for  bitfield_pattern *******************  
        */
        // Create a new instance of PhiPsiBit according to the values of codebook_information
        // and feedback type, if not matched raise error 
        // for mor information see IEEE 802.11ax Table 9-29a
        let phi_psi = match (mimo_ctrl.codebook_info().value(), mimo_ctrl.feedback_type().value()) {
            (0, 0) => PhiPsiBit { phi_bit: 4, psi_bit: 2 },     // fb 0 -> SU 
            (0, 1) => PhiPsiBit { phi_bit: 7, psi_bit: 5 },     // fb 1 -> MU
            (1, 0) => PhiPsiBit { phi_bit: 6, psi_bit: 4 },     // fb 0 -> SU
            (1, 1) => PhiPsiBit { phi_bit: 9, psi_bit: 7 },     // fb 1 -> MU
            (_, 2) => panic!(" Feedback type is set to CQI (Channel Quality Indication) coodboo is reserved"),
            _ => panic!("Invalid coodebook or feedback type values"),
        };
        
        /*
        * ******************* Set bitfield_pattern *******************  
        */
        let nr_index = mimo_ctrl.nr_index().value() as usize;
        let nc_index = mimo_ctrl.nc_index().value() as usize;

        let selected_pattern: &[&str] = if nr_index >= nc_index {
            // NOTE: nr and nc are -1 due to indexing by 0 
            if (nr_index == 1 && nc_index == 0 ) || (nr_index == 1 && nc_index == 2){ // 2x1 and 2x2
                &PATTERN_2_1
            } else if nr_index == 2 && nc_index == 0 { //3x1
                &PATTERN_3_1
            } else if (nr_index == 2 && nc_index == 1)  || (nr_index == 2 && nc_index == 2){ //3x2 and 3x3
                &PATTERN_3_2
            } else if nr_index == 3 && nc_index == 0 { //4x1
                &PATTERN_4_1
            } else if nr_index == 3 && nc_index == 1 { // 4x2
                &PATTERN_4_2
            } else if (nr_index == 3 && nc_index == 2) || (nr_index == 3 && nc_index == 3) { // 4x3 and 4x4
                &PATTERN_4_3
            } else {
                panic!("Invalid nr_index or nc_index in the HE Mimo Control field ")
            }
        } else {
                panic!("Invalid nr_index or nc_index in the HE Mimo Control field ");
        };
    
        let bitfield_pattern: Vec<u8> = selected_pattern.iter()
            .map(|&placeholder| match placeholder {
                "phi" => phi_psi.phi_bit, // Substitute 'phi' with PhiPsiBit.psi_bit
                "psi" => phi_psi.psi_bit, // Substitute 'psi' with PhiPsiBit.psi_bit
                _ => panic!("Something went wrong wit the place holder while mapping its real values"),   
            })
            .collect();
            
            let mut num_sub = 0;
            /*
            * ******************* Set BW *******************  
            */
            // NOTE: based on grouping bit the number of subcarrier change 
            // for more details see IEEE 802.11ax Table 9-91a and Table 9-91e
            if mimo_ctrl.grouping().value() == 0{   // subcarrier grouping: 4
            num_sub = match mimo_ctrl.bandwidth().try_into(){
                Ok(Bandwidth::Bw20) => 64,          // [–122, –120, –116, …, –8, –4, –2, 2, 4, 8, …, 116, 120, 122]
                Ok(Bandwidth::Bw40) => 122,         // [–244, –240, …, –8, –4, 4, 8, …, 240, 244]
                Ok(Bandwidth::Bw80) => 250,         // [–500, –496, …, –8, –4, 4, 8, …, 496, 500]
                Ok(Bandwidth::Bw160) => 500,        // [–1012, –1008, …, –520, –516, –508, –504, …, –16, –12, 12, 16, …, 504, 508, 516, 520, …, 1008, 1012]
                _ => panic!("Invalid BW")
            };          

            } else if mimo_ctrl.grouping().value() == 1 {   // subcarrier grouping: 4
            num_sub = match mimo_ctrl.bandwidth().try_into(){
                    Ok(Bandwidth::Bw20) => 50,      // [–122, –116, –100, …, –20, –4, –2, 2, 4, 20, …, 100, 116, 122]
                    Ok(Bandwidth::Bw40) => 32,      // [–244, –228, …, –20, –4, 4, 20, …, 228, 244]
                    Ok(Bandwidth::Bw80) => 64,      // [–500, –484, …, –20, –4, 4, 20, …, 484, 500]
                    Ok(Bandwidth::Bw160) => 160,    //  [–1012, –996, …, –532, –516, –508, –492, …, –28, –12, 12, 28, …, 492, 508, 516, 532, …, 996, 1012]
                    _ => panic!("Invalid BW")
            };  
            } else {
                panic!("Invalid grouping of subcarrier");
            }


            // Initialize the ExtractionConfig
            let config = ExtractionConfig {
                bitfield_pattern: bitfield_pattern,
                num_subcarrier: num_sub,
            };

        // println!("bitfield_pattern : {}", bitfield_pattern);
        // println!("selected_pattern : {}", selected_pattern);
        println!("NC : {}", mimo_ctrl.nc_index());
        println!("Phi: {}", phi_psi.phi_bit);
        println!("Psi: {}", phi_psi.psi_bit);
        println!("Bitfield Pattern: {:?}", config.bitfield_pattern);
        println!("num_sub {}", num_sub);
        config // Return the config instance1
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
    println!("total_bits_per_chunk {}", total_bits_per_chunk);
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
    println!("byte_stream.len() {}", byte_stream.len());
    println!("num_chunks {}", num_chunks);
    println!("bitfield_pattern.as_slice() {:?}", bitfield_pattern.as_slice());
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
    println!("bfa_payload.len() {}", bfa_payload.len());
    // println!("num_chunks {}", num_chunks);
    // println!("bitfield_pattern.as_slice() {:?}", bitfield_pattern.as_slice());
    extract_bitfields(
        bfa_payload,
        extraction_config.bitfield_pattern,
        extraction_config.num_subcarrier,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::he_mimo_ctrl::HeMimoControl;
    #[test]
    // fn test_from_he_mimo_ctrl() { 
        
    //     let mimo_ctrl = he_mimo_ctrl{
    //     // Initialize with the appropriate values
    //     codebook_info_value: 1,
    //     feedback_type_value: 0,
    //     nc_index_value: 3,
    //     nr_index_value: 1,
    //     grouping_value: 0,
    //     bandwidth_value: Bandwidth::Bw20,
    // };

    // // Call the function
    // let config = ExtractionConfig::from_he_mimo_ctrl(&mimo_ctrl);

    // // Assert the expected outcomes
    // let expected_bitfield_pattern = vec![6, 4, 6, 4, 6, 4, 6, 4, 6, 4]; // 4 phi, 2 psi
    // let expected_num_subcarrier = 64; // Expected number of subcarriers for BW20 with grouping 0
    
    // assert_eq!(config.bitfield_pattern, expected_bitfield_pattern);
    // assert_eq!(config.num_subcarrier, expected_num_subcarrier);
    // }
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
