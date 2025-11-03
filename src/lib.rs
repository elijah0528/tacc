mod codec;
mod error;
mod mapping;

use codec::{decode_i32_list_with_gzip, encode_i32_list_with_gzip};
use error::TaccError;
use mapping::TokenMapping;
use pyo3::prelude::*;
use std::sync::Arc;

// Python-facing codec for compression/decompression
#[pyclass]
pub struct Codec {
    #[pyo3(get)]
    tokenizer_name: String,
    mapping: Arc<TokenMapping>,
}

#[pymethods]
impl Codec {
    #[new]
    fn new(tokenizer: &str) -> PyResult<Self> {
        let mapping = TokenMapping::load(tokenizer)?;
        Ok(Self {
            tokenizer_name: tokenizer.to_string(),
            mapping,
        })
    }

    /// Encode tokens to compressed bytes
    #[pyo3(signature = (tokens, *, gzip=false))]
    fn encode_tokens(&self, tokens: Vec<String>, gzip: bool) -> PyResult<Vec<u8>> {
        let mapped_ids: Result<Vec<i32>, TaccError> = tokens
            .iter()
            .map(|t| self.mapping.token_to_mapped_id(t))
            .collect();

        Ok(encode_i32_list_with_gzip(&mapped_ids?, gzip))
    }

    /// Encode token IDs to compressed bytes
    #[pyo3(signature = (token_ids, *, gzip=false))]
    fn encode_token_ids(&self, token_ids: Vec<u32>, gzip: bool) -> PyResult<Vec<u8>> {
        let mapped_ids: Result<Vec<i32>, TaccError> = token_ids
            .iter()
            .map(|&tid| self.mapping.token_id_to_mapped_id(tid))
            .collect();

        Ok(encode_i32_list_with_gzip(&mapped_ids?, gzip))
    }

    /// Encode mapped IDs directly (no mapping)
    #[pyo3(signature = (mapped_ids, *, gzip=false))]
    fn encode_mapped_ids(&self, mapped_ids: Vec<i32>, gzip: bool) -> Vec<u8> {
        encode_i32_list_with_gzip(&mapped_ids, gzip)
    }

    /// Benchmark: encode raw token IDs as i32
    #[pyo3(signature = (token_ids, *, gzip=false))]
    fn encode_raw_token_ids(&self, token_ids: Vec<u32>, gzip: bool) -> Vec<u8> {
        let as_i32: Vec<i32> = token_ids.iter().map(|&id| id as i32).collect();
        encode_i32_list_with_gzip(&as_i32, gzip)
    }

    /// Decode to mapped IDs
    #[pyo3(signature = (payload, *, gzip=false))]
    fn decode_mapped_ids(&self, payload: Vec<u8>, gzip: bool) -> PyResult<Vec<i32>> {
        Ok(decode_i32_list_with_gzip(&payload, gzip)?)
    }

    /// Decode to tokens
    #[pyo3(signature = (payload, *, gzip=false))]
    fn decode_tokens(&self, payload: Vec<u8>, gzip: bool) -> PyResult<Vec<String>> {
        let mapped_ids = decode_i32_list_with_gzip(&payload, gzip)?;
        let tokens: Result<Vec<String>, TaccError> = mapped_ids
            .iter()
            .map(|&mid| self.mapping.mapped_id_to_token(mid).map(|s| s.to_string()))
            .collect();
        Ok(tokens?)
    }

    /// Decode to token IDs
    #[pyo3(signature = (payload, *, gzip=false))]
    fn decode_token_ids(&self, payload: Vec<u8>, gzip: bool) -> PyResult<Vec<u32>> {
        let mapped_ids = decode_i32_list_with_gzip(&payload, gzip)?;
        let token_ids: Result<Vec<u32>, TaccError> = mapped_ids
            .iter()
            .map(|&mid| self.mapping.mapped_id_to_token_id(mid))
            .collect();
        Ok(token_ids?)
    }

    // Map token IDs to mapped IDs (no compression)
    fn token_id_to_mapped_id(&self, token_ids: Vec<u32>) -> PyResult<Vec<i32>> {
        let mapped_ids: Result<Vec<i32>, TaccError> = token_ids
            .iter()
            .map(|&tid| self.mapping.token_id_to_mapped_id(tid))
            .collect();
        Ok(mapped_ids?)
    }

    // Map mapped IDs back to token IDs (no decompression)
    fn mapped_id_to_token_id(&self, mapped_ids: Vec<i32>) -> PyResult<Vec<u32>> {
        let token_ids: Result<Vec<u32>, TaccError> = mapped_ids
            .iter()
            .map(|&mid| self.mapping.mapped_id_to_token_id(mid))
            .collect();
        Ok(token_ids?)
    }
}

// List available tokenizers
#[pyfunction]
fn list_available_tokenizers() -> Vec<String> {
    mapping::list_available_tokenizers()
}

/// Compress token IDs with selected tokenizer
#[pyfunction]
#[pyo3(signature = (token_ids, tokenizer, *, gzip=false))]
fn compress_token_ids(token_ids: Vec<u32>, tokenizer: &str, gzip: bool) -> PyResult<Vec<u8>> {
    let codec = Codec::new(tokenizer)?;
    codec.encode_token_ids(token_ids, gzip)
}

/// Decompress bytes to token IDs
#[pyfunction]
#[pyo3(signature = (payload, tokenizer, *, gzip=false))]
fn decompress_token_ids(payload: Vec<u8>, tokenizer: &str, gzip: bool) -> PyResult<Vec<u32>> {
    let codec = Codec::new(tokenizer)?;
    codec.decode_token_ids(payload, gzip)
}

// Python module entry point
#[pymodule]
fn _internal(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Codec>()?;
    m.add_function(wrap_pyfunction!(list_available_tokenizers, m)?)?;
    m.add_function(wrap_pyfunction!(compress_token_ids, m)?)?;
    m.add_function(wrap_pyfunction!(decompress_token_ids, m)?)?;
    Ok(())
}
