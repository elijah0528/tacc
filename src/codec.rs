use crate::error::TaccError;
use thrift::protocol::{
    TCompactInputProtocol, TCompactOutputProtocol, TFieldIdentifier, TInputProtocol,
    TListIdentifier, TOutputProtocol, TStructIdentifier, TType,
};
use thrift::transport::TBufferChannel;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::io::{Write, Read};


/// Encode a list of i32 to Thrift CompactProtocol bytes.
pub fn encode_i32_list(values: &[i32]) -> Vec<u8> {
    encode_i32_list_with_gzip(values, false)
}

/// Encode a list of i32 to Thrift CompactProtocol bytes, with optional gzip compression.
pub fn encode_i32_list_with_gzip(values: &[i32], use_gzip: bool) -> Vec<u8> {
    let estimated_size = 10 + values.len() * 5;
    let mut transport = TBufferChannel::with_capacity(0, estimated_size);
    {
        let mut p = TCompactOutputProtocol::new(&mut transport);

        let _ = p.write_struct_begin(&TStructIdentifier::new("TokenPayload"));
        let _ = p.write_field_begin(&TFieldIdentifier::new("tokens", TType::List, 1));
        let _ = p.write_list_begin(&TListIdentifier::new(TType::I32, values.len() as i32));
        for &v in values {
            let _ = p.write_i32(v);
        }
        let _ = p.write_list_end();
        let _ = p.write_field_end();
        let _ = p.write_field_stop();
        let _ = p.write_struct_end();
        let _ = p.flush();
    }

    let bytes = transport.write_bytes().to_vec();

    if use_gzip {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes).expect("Failed to write to gzip encoder");
        encoder.finish().expect("Failed to finish gzip encoding")
    } else {
        bytes
    }
}

/// Decode a list of i32 from Thrift CompactProtocol bytes.
pub fn decode_i32_list(bytes: &[u8]) -> Result<Vec<i32>, TaccError> {
    decode_i32_list_with_gzip(bytes, false)
}

/// Decode a list of i32 from Thrift CompactProtocol bytes, with optional gzip decompression.
pub fn decode_i32_list_with_gzip(bytes: &[u8], use_gzip: bool) -> Result<Vec<i32>, TaccError> {
    let decompressed_bytes;
    let bytes_to_parse = if use_gzip {
        let mut decoder = GzDecoder::new(bytes);
        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)
            .map_err(|e| TaccError::InvalidPayload(format!("Gzip decompression failed: {}", e)))?;
        decompressed_bytes = buffer;
        &decompressed_bytes[..]
    } else {
        bytes
    };

    let mut transport = TBufferChannel::with_capacity(bytes_to_parse.len(), 0);
    transport.set_readable_bytes(bytes_to_parse);
    let mut p = TCompactInputProtocol::new(transport);

    p.read_struct_begin().map_err(|e| TaccError::InvalidPayload(format!("{}", e)))?;
    let field = p.read_field_begin().map_err(|e| TaccError::InvalidPayload(format!("{}", e)))?;

    if field.field_type == TType::Stop {
        return Err(TaccError::InvalidPayload("Payload missing tokens field".to_string()))
    }
    if field.field_type != TType::List || field.id != Some(1) {
        return Err(TaccError::InvalidPayload("Expected LIST field with id 1".to_string()))
    }

    let list = p.read_list_begin().map_err(|e| TaccError::InvalidPayload(format!("{}", e)))?;
    if list.element_type != TType::I32 {
        return Err(TaccError::InvalidPayload("Expected I32 elements".to_string()))
    }
    let mut values = Vec::with_capacity(list.size as usize);
    for _ in 0..list.size {
        values.push(p.read_i32().map_err(|e| TaccError::InvalidPayload(format!("{}", e)))?);
    }

    let _ = p.read_list_end().map_err(|e| TaccError::InvalidPayload(format!("{}", e)))?;
    let _ = p.read_field_end().map_err(|e| TaccError::InvalidPayload(format!("{}", e)))?;

    let next_field = p.read_field_begin().map_err(|e| TaccError::InvalidPayload(format!("{}", e)))?;
    if next_field.field_type != TType::Stop {
        return Err(TaccError::InvalidPayload("Unexpected fields in payload".to_string()))
    }

    let _ = p.read_struct_end().map_err(|e| TaccError::InvalidPayload(format!("{}", e)))?;
    Ok(values)
}

