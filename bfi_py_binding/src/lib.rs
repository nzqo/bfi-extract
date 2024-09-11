use bfi_lib::{extract_from_capture, ExtractedBfiData};
use numpy::{PyArray1, PyArray3};
use pyo3::prelude::*;

#[pymodule]
fn bfi_extract<'py>(_py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    /**
     * Extract data from a pcap file
     *
     * \param path: Path to pcap file
     *
     * \returns A tuple of extracted values, each a numpy array
     *          with length equal to the number of packets.
     */
    #[allow(dead_code)]
    #[pyfn(m)]
    fn extract_from_pcap<'py>(
        py: Python<'py>,
        path: &str,
    ) -> (
        Bound<'py, PyArray1<f64>>,
        Bound<'py, PyArray1<u8>>,
        Bound<'py, PyArray3<u16>>,
    ) {
        let ExtractedBfiData {
            timestamps,
            token_nums,
            bfa_angles,
        } = extract_from_capture(path.into());

        (
            PyArray1::from_vec_bound(py, timestamps),
            PyArray1::from_vec_bound(py, token_nums),
            PyArray3::from_vec3_bound(py, &bfa_angles).unwrap(),
        )
    }

    Ok(())
}
