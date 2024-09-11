#![allow(dead_code)]
mod bandwidth;
mod errors;
pub mod extract_bfa;
mod util;

#[cfg(not(feature = "full_extract"))]
mod he_mimo_ctrl;

#[cfg(feature = "full_extract")]
#[path = "he_mimo_ctrl_full.rs"]
mod he_mimo_ctrl;

use extract_bfa::{extract_bfa, ExtractionConfig};
use he_mimo_ctrl::HeMimoControl;
use pcap::{Capture, Packet};
use polars::datatypes::ListChunked;
use polars::error::PolarsError;
use polars::frame::DataFrame;
use polars::prelude::*;
use polars::series::Series;
use std::fs::File;
use std::path::PathBuf;

/**
 * Accumulated data from the packets from the pcap file
 */
#[derive(Debug)]
pub struct ExtractedBfiData {
    pub timestamps: Vec<f64>,
    pub token_nums: Vec<u8>,
    pub bfa_angles: Vec<Vec<Vec<u16>>>,
}

/**
 * Constructor
 */
impl ExtractedBfiData {
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
            token_nums: Vec::new(),
            bfa_angles: Vec::new(),
        }
    }
}

/**
 * Data extracted from a single packet in the pcap
 */
struct SinglePacketBfiData {
    timestamp: f64,
    token_number: u8,
    bfa_angles: Vec<Vec<u16>>,
}

/**
 * Parquet conversion of extracted BFI data
 */
impl ExtractedBfiData {
    pub fn to_parquet(&self, file_path: PathBuf) -> Result<(), PolarsError> {
        // Convert timestamps and token_nums to Polars Series
        let timestamps_series = Series::new("timestamps", &self.timestamps);

        // Convert Vec<u8> to Vec<u32> for token_nums
        // Required because polars doesnt support u8
        let token_nums_series = Series::new(
            "token_nums",
            &self
                .token_nums
                .iter()
                .map(|&num| num as u32)
                .collect::<Vec<u32>>(),
        );

        // Convert Vec<Vec<u8>> to a List Series of Vec<i32>
        let bfa_angles_series = ListChunked::from_iter(self.bfa_angles.iter().map(|outer| {
            ListChunked::from_iter(outer.iter().map(|inner| {
                UInt32Chunked::from_vec(
                    "bfa_angles_inner",
                    inner.iter().map(|&e| e as u32).collect::<Vec<u32>>(),
                )
                .into_series()
            }))
            .into_series()
        }))
        .into_series();

        // Construct DataFrame from the series
        let mut df = DataFrame::new(vec![
            timestamps_series,
            token_nums_series,
            bfa_angles_series,
        ])?;

        // Write DataFrame to a Parquet file
        let file = File::create(file_path)?;
        ParquetWriter::new(file).finish(&mut df)?;

        Ok(())
    }
}

/**
 * Extract data from a single packet
 */
fn extract_from_packet(packet: &Packet) -> SinglePacketBfiData {
    // Extract the timestamp from the pcap packet
    let timestamp = packet.header.ts;
    let timestamp_secs = timestamp.tv_sec as f64 + timestamp.tv_usec as f64 * 1e-6;

    let header_length = u16::from_le_bytes([packet.data[2], packet.data[3]]) as usize;
    let mimo_ctrl_start = header_length + 26;

    let mimo_control = HeMimoControl::from_bytes(&packet[mimo_ctrl_start..]);

    // full extract debug print
    #[cfg(feature = "full_extract")]
    print!("Test: {:#?}\n", mimo_control);

    // NOTE: BFA data starts after mimo_control (5 bytes) and SNR (2 bytes)
    // They last until before the last four bytes (Frame Check Sequence)
    let bfa_start = mimo_ctrl_start + 7;
    let bfa_end = packet.len() - 4;

    // Extract the binary data of the BFA angles
    let bfa_data = &packet[bfa_start..bfa_end];
    let bfa_angles = extract_bfa(bfa_data, ExtractionConfig::from_he_mimo_ctrl(mimo_control))
        .expect("BFA extraction failed");

    SinglePacketBfiData {
        timestamp: timestamp_secs,
        token_number: mimo_control.dialog_token_number,
        bfa_angles: bfa_angles,
    }
}

/**
 * Extract data from a pcap file
 *
 * \param capture_path Path to pcap capture file
 *
 */
pub fn extract_from_capture(capture_path: PathBuf) -> ExtractedBfiData {
    let mut capture = Capture::from_file(capture_path).expect("Couldn't open pcap file");
    let mut extracted_data = ExtractedBfiData::new();

    while let Ok(packet) = capture.next_packet() {
        let SinglePacketBfiData {
            timestamp,
            token_number,
            bfa_angles,
        } = extract_from_packet(&packet);

        extracted_data.timestamps.push(timestamp);
        extracted_data.token_nums.push(token_number);
        extracted_data.bfa_angles.push(bfa_angles);
    }

    extracted_data
}
