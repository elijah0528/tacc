"""High-performance Rust implementation of tacc_encoder.

This module provides a drop-in replacement for the Python tacc_encoder
with 10-50x performance improvements.
"""

from tacc._internal import (
    Codec as _RustCodec,
    compress_token_ids as _rust_compress_token_ids,
    decompress_token_ids as _rust_decompress_token_ids,
    list_available_tokenizers,
)


class Codec:
    """Rust-backed codec for token compression. """

    def __init__(self, tokenizer: str, *, mapping_path=None):
        if mapping_path is not None:
            raise NotImplementedError(
                "Custom mapping paths not yet supported in Rust version"
            )
        self._inner = _RustCodec(tokenizer)
        self.tokenizer_name = tokenizer

    def encode_tokens(self, tokens, *, gzip=False):
        """ Encode string tokens to Thrift CompactProtocol bytes. """
        return bytes(self._inner.encode_tokens(list(tokens), gzip=gzip))

    def encode_token_ids(self, token_ids, *, gzip=False):
        """ Encode token IDs to Thrift CompactProtocol bytes. """
        return bytes(self._inner.encode_token_ids(list(token_ids), gzip=gzip))

    def encode_mapped_ids(self, mapped_ids, *, gzip=False):
        """ Encode mapped IDs to Thrift CompactProtocol bytes. """
        return bytes(self._inner.encode_mapped_ids(list(mapped_ids), gzip=gzip))

    def encode_raw_token_ids(self, token_ids, *, gzip=False):
        """ Encode raw token IDs without mapping. """
        return bytes(self._inner.encode_raw_token_ids(list(token_ids), gzip=gzip))

    def decode_mapped_ids(self, payload, *, gzip=False):
        """ Decode Thrift CompactProtocol bytes to mapped IDs. """
        return self._inner.decode_mapped_ids(bytes(payload), gzip=gzip)

    def decode_tokens(self, payload, *, gzip=False):
        """ Decode Thrift CompactProtocol bytes to string tokens. """
        return self._inner.decode_tokens(bytes(payload), gzip=gzip)

    def decode_token_ids(self, payload, *, gzip=False):
        """ Decode Thrift CompactProtocol bytes to token IDs. """
        return self._inner.decode_token_ids(bytes(payload), gzip=gzip)

    def token_id_to_mapped_id(self, token_ids):
        """ Convert token IDs to mapped IDs without encoding. """
        return self._inner.token_id_to_mapped_id(list(token_ids))

    def mapped_id_to_token_id(self, mapped_ids):
        """ Convert mapped IDs to token IDs without decoding. """
        return self._inner.mapped_id_to_token_id(list(mapped_ids))


def compress_token_ids(token_ids, *, tokenizer, mapping_path=None, gzip=False):
    """Compress token IDs."""
    if mapping_path is not None:
        raise NotImplementedError("Custom mapping paths not yet supported in Rust version")
    return bytes(_rust_compress_token_ids(list(token_ids), tokenizer, gzip=gzip))


def decompress_token_ids(payload, *, tokenizer, mapping_path=None, gzip=False):
    """Decompress payload to token IDs."""
    if mapping_path is not None:
        raise NotImplementedError("Custom mapping paths not yet supported in Rust version")
    return _rust_decompress_token_ids(bytes(payload), tokenizer, gzip=gzip)


__all__ = [
    "Codec",
    "compress_token_ids",
    "decompress_token_ids",
    "list_available_tokenizers",
]
