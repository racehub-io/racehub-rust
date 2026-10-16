# Telemetry Capture Library

This crate provides functionality for capturing, recording, and processing telemetry data from racing games, with an initial focus on F1 2024.

## Features

- Capture UDP telemetry packets from racing games
- Record telemetry data to `.rhc` (RaceHub Capture) files
- Store telemetry data in SQLite databases
- Convert telemetry recordings to JSON for analysis

## RHC File Format

The `.rhc` file format is a custom binary format designed for efficiently storing telemetry data:

- File header: Magic bytes "F1TEL", version, session ID, timestamp
- Packet stream: Each packet consists of timestamp, packet type, compressed data length, and zstd-compressed data

## Tools

### rhc2json

The `rhc2json` tool converts `.rhc` telemetry recordings to JSON files for easier analysis, with support for various compression formats including those compatible with pandas.

#### Usage

```bash
rhc2json --input <INPUT_FILE> [--output <OUTPUT_FILE>] [--format <FORMAT>] [--compression <LEVEL>]
```

Arguments:
- `--input`, `-i`: Path to the input .rhc file
- `--output`, `-o`: Path to the output JSON file (default: same name with extension based on format)
- `--format`, `-f`: Compression format to use (default: "zst")
  - Available formats: "none", "zst", "gz", "xz", "bz2"
  - For pandas compatibility, use "gz", "bz2", or "none"
- `--compression`, `-c`: Compression level (default: 3)
  - For zstd: 1-22 (higher = better compression but slower)
  - For others: 1-9 (higher = better compression but slower)

#### Example

```bash
# Convert to uncompressed JSON (direct pandas import)
cargo run --bin rhc2json -- --input telemetry_data/1234567890_session_123456789.rhc --format none

# Convert to gzip-compressed JSON (pandas compatible)
cargo run --bin rhc2json -- --input telemetry_data/1234567890_session_123456789.rhc --format gz

# Convert to bzip2-compressed JSON (pandas compatible)
cargo run --bin rhc2json -- --input telemetry_data/1234567890_session_123456789.rhc --format bz2 --compression 9

# Convert with zstd (best compression, but requires extra steps for pandas)
cargo run --bin rhc2json -- --input telemetry_data/1234567890_session_123456789.rhc --format zst --compression 19
```

#### Building with All Compression Formats

By default, only zstd compression is available. To enable all compression formats:

```bash
cargo build --release --bin rhc2json --features all-compression
```

## JSON Format

The JSON output from `rhc2json` has the following structure:

```json
{
  "metadata": {
    "session_id": "123456789",
    "timestamp": 1621234567,
    "source_file": "telemetry_data/1234567890_session_123456789.rhc"
  },
  "packets": [
    {
      "timestamp_us": 1621234567890123,
      "packet_type": 6,
      "data": {
        "type": "CarTelemetry",
        "header": { ... },
        "car_telemetry_data": [ ... ],
        ...
      }
    },
    ...
  ]
}
```

## Loading JSON Output in Pandas

### For uncompressed, gzip or bzip2 files (directly supported by pandas):

```python
import pandas as pd

# Reading the JSON file (supports .json, .json.gz, .json.bz2)
with open('yourfile.json.gz', 'r') as f:
    data = pd.read_json(f)

# Or more directly:
data = pd.read_json('yourfile.json.gz', compression='gzip')

# Extract just the packets
packets_df = pd.json_normalize(data['packets'])
```

### For zstd compressed files:

```python
import zstandard as zstd
import json
import pandas as pd

# Read and decompress the file
with open('yourfile.json.zst', 'rb') as f:
    dctx = zstd.ZstdDecompressor()
    decompressed = dctx.decompress(f.read())
    
# Parse the JSON
data = json.loads(decompressed)

# Convert to DataFrame
packets_df = pd.json_normalize(data['packets'])
```

## Library Usage

The `capture` crate can also be used as a library for custom telemetry processing:

```rust
use capture::{TelemetryReader, TelemetryPacket, F1TelemetryPacket};

fn process_telemetry_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = TelemetryReader::open(path)?;
    
    println!("Session ID: {}", reader.session_id());
    
    while let Some(packet_info) = reader.read_next_packet()? {
        let parsed_packet = F1TelemetryPacket::from_bytes(&packet_info.data)?;
        
        // Process the packet based on its type
        match parsed_packet {
            F1TelemetryPacket::CarTelemetry(data) => {
                // Process telemetry data
            },
            F1TelemetryPacket::Session(data) => {
                // Process session data
            },
            // ... handle other packet types
            _ => {}
        }
    }
    
    Ok(())
}
``` 