/** ------------------------------------------------------------
 * Persistence (saving extracted data to parquet files)
 * ------------------------------------------------------------- */
use crate::bfi_data::ExtractedBfiData;
use crate::PathBuf;
use std::fs::File;

use polars::prelude::*;
use polars::{datatypes::ListChunked, error::PolarsError, frame::DataFrame, series::Series};

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
