# Tokenization-Aware Compression Codec (tacc)

Tokenization-Aware Compression Codec (tacc) encodes LLM output as mapped token IDs instead of UTF-8 bytes, using a precomputed mapping to zero-centre token frequencies so Thrift CompactProtocol compresses the integer sequence efficiently and then gzips the information dense byte stream, yielding much smaller payloads than gzip alone or brotli for chat completions—especially valuable in low-latency or bandwidth-limited scenarios. It currently supports cl100k_base, gpt2, o200k_base, o200k_harmony, p50k_base, p50k_edit, and r50k_base. This algorithm not only achieves superior compression ratios, but also provides a 20–25x speedup in end-to-end compression time due to the tacc pre-processing which vastly speeds up the subsequent gzip compression.


Try it out:
```bash
pip install tacc
```

## Benchmarks

![Compression Efficiency Comparison](assets/compression_metrics.png)

Tacc + gzip significantly compresses token output 

| Method                         | Size (bytes) | Time (ms) | vs Tacc   |
|---------------------------------|--------------|-----------|-----------|
| Tacc only                      |    43,800    |   0.827   | baseline  |
| Python gzip only (raw IDs)     |    41,615    |  13.707   |  +1557%   |
| Tacc + Python gzip             |    31,542    |   1.390   |   +68%    |
| Tacc + Rust gzip               |    31,796    |   1.097   |   +33%    |

This chart shows end-to-end compression and speed on a real token dataset (50 samples, 27,535 tokens):

- **Tacc only**: Thrift Compact encoding, no gzip applied (serves as the baseline at 1.00x for both size and speed).
- **Python gzip only**: Standard gzip applied to the raw token ID bytes—results in poor compression ratio and is dramatically slower than other approaches.
- **Tacc + Python gzip**: Thrift Compact encoding followed by Python gzip—much faster than gzipping raw token IDs directly, and achieves greater compression.
- **Tacc + Rust gzip**: Thrift Compact encoding followed by Rust's native gzip (used by the internal Rust library). This option is both faster and comparable or slightly better in size than Python gzip.

### Why does Tacc pre-processing improve gzip?

Tacc’s compact encoding preprocesses token IDs using mapping and Thrift CompactProtocol, which removes entropy, eliminates metadata overhead, and ensures a much denser byte stream. As a result, gzip becomes *both* more effective (smaller files) and faster—since the data is already smaller, more regular, and simpler to compress. Compared to gzipping raw token IDs, Tacc’s method cuts file size by up to 2-3x and provides a 20–25x speedup in end-to-end compression time. For both storage and network transfer, the combination of Tacc encoding and (optionally) gzip offers the best efficiency.


## Installation
### From PyPI

```bash
pip install tacc
```

### From Source (Development)

```bash
# Install maturin (Rust-Python build tool)
pip install maturin

# Build and install in development mode
maturin develop --release

```

### Build Wheel for Distribution

```bash
maturin build --release
# Wheel will be in target/wheels/
```

## Usage

```python
from tacc import Codec

# Instantiate the Codec for a specific tokenizer (e.g., "cl100k_base")
codec = Codec("cl100k_base")

# Example: encode a list of token IDs
tokens = [" the", " quick", " brown", " fox"]
payload = codec.encode_tokens(tokens)

# Decode the payload back to the original tokens
decoded_tokens = codec.decode_tokens(payload)
assert decoded_tokens == tokens
```

## API Reference

All encoding/decoding methods accept an optional `gzip` keyword argument for additional compression. Anything compressed is expected to be `mapped_ids` and is always decoded as if it was. The `gzip` parameter is an optional parameter for further compression, that should be set based on how it was encoded

**Encoding methods:**
- `Codec.encode_tokens(tokens, *, gzip=False)` — Encode an iterable of string tokens to Thrift CompactProtocol bytes (with mapping).
- `Codec.encode_token_ids(token_ids, *, gzip=False)` — Encode an iterable of token IDs to Thrift CompactProtocol bytes (with mapping).
- `Codec.encode_mapped_ids(mapped_ids, *, gzip=False)` — Encode an iterable of mapped IDs to Thrift CompactProtocol bytes (no further mapping).
- `Codec.encode_raw_token_ids(token_ids, *, gzip=False)` — Encode an iterable of raw token IDs to bytes without mapping.

**Decoding methods:**
- `Codec.decode_tokens(payload, *, gzip=False)` — Decode payload bytes to a list of string tokens.
- `Codec.decode_token_ids(payload, *, gzip=False)` — Decode payload bytes to a list of token IDs.
- `Codec.decode_mapped_ids(payload, *, gzip=False)` — Decode payload bytes to a list of mapped IDs.

**Utility methods:**
- `Codec.token_id_to_mapped_id(token_ids)` — Convert an iterable of token IDs to mapped IDs without encoding.
- `Codec.mapped_id_to_token_id(mapped_ids)` — Convert an iterable of mapped IDs to token IDs without decoding.

**Non-codec methods:**
- `compress_token_ids(token_ids, *, tokenizer, mapping_path=None, gzip=False)` — Compress a list of token IDs for a specified tokenizer.
- `decompress_token_ids(payload, *, tokenizer, mapping_path=None, gzip=False)` — Decompress a payload back to token IDs for a specified tokenizer.

### When to use gzip?

- **Storage**: Always use `gzip=True` for persistent storage (2-3x smaller)
- **Network**: Use `gzip=True` if not using HTTP compression
- **Low-latency**: Skip gzip for minimal overhead (~0.05ms per 1k tokens)



