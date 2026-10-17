# Capture developer guide

The `capture` crate contains the native decoders and the older F1 capture library.
The `aggregator` crate uses the native decoders to write source-rate Parquet and raw packet archives.
Start with the [main README](../README.md) for game setup and user commands.

Local capture does not require a RaceHub account or cloud service.
Reuse follows the [licence scope](../LICENSING.md).
Keep all separate protocol, SDK, dependency, and recording terms.

## Source layout

| Path | Responsibility |
| --- | --- |
| [`src/native.rs`](src/native.rs) | Simulator selection and stateful F1 24 to canonical row decoding. |
| [`src/protocols/`](src/protocols/) | Native iRacing and rFactor 2 Protobuf decoding and channel mapping. |
| [`../common/src/f12024/`](../common/src/f12024/) | F1 24 packet structures and protocol constants. |
| [`src/f12024/`](src/f12024/) | F1 parsers, the older UDP capture loop, and legacy frame projection. |
| [`src/recorder.rs`](src/recorder.rs) | Legacy F1TEL recorder and public `TelemetryReader`. |
| [`src/db.rs`](src/db.rs) | The legacy SQLite frame store. |
| [`src/lib.rs`](src/lib.rs) | Rust exports and the C ABI. |
| [`src/bin/rhc2json.rs`](src/bin/rhc2json.rs) | F1TEL to packet JSON inspection tool. |
| [`../aggregator/src/main.rs`](../aggregator/src/main.rs) | Native CLI, receive queue, session boundaries, status, and shutdown. |
| [`../aggregator/src/raw.rs`](../aggregator/src/raw.rs) | Native F1TEL/RHRAW archives and bounded reads. |
| [`../aggregator/src/storage.rs`](../aggregator/src/storage.rs) | Arrow builders, Zstd Parquet parts, and atomic session manifests. |

## Build targets

Run from the repository root:

```sh
cargo build --release --locked -p aggregator
cargo build --release --locked -p capture
cargo build --release --locked -p capture --bin rhc2json
cargo test --workspace --locked
```

The `capture` crate builds an `rlib` and a `cdylib`.
The dynamic library uses the platform's `.dll`, `.dylib`, or `.so` suffix.
The new recorder is the `aggregator` executable.
It does not load the dynamic library or use its FFI callback.

## Decoder interface

`capture::native::Decoder::new(simulator)` selects `f1`, `iracing`, or `rfactor2`.
`Decoder::decode(&mut self, bytes)` returns a `DecodedPacket` or a string error.

| Return field | Meaning |
| --- | --- |
| `rows` | `Vec<TelemetryRow>`. Each row is a `serde_json::Map<String, Value>` with scalar channels. |
| `player_participant_id` | Optional identity observed from the protocol. |
| `session_key` | Optional source session identity. Keep it separate from the recording UUID. |

`capture::protocols::ProtobufAdapters` also exposes `iracing` and `rfactor2` methods.
Its generated bindings are checked in.
A normal build does not run `protoc`.
Read [NOTICE](src/protocols/NOTICE) before changes to schemas or generated files.
The generation input preserves scalar presence with optional fields and keeps the original wire field numbers.

Keep one decoder per simulator stream.
The F1 and rFactor 2 decoders cache participant, track, player, or observed lap information.
The CLI applies recording scope, source ID filtering, and clock boundary checks around decoding.
A library caller must also keep separate source sessions separate.
An error from one invalid datagram does not require a process crash.

Native F1 decoding validates F1 24 format, game year, packet version, packet size, and session time.
Other game years are rejected.
Known fixed-size parsers reject unused trailing bytes.
Event payloads have their protocol-specific sizes.
Original packet bytes are saved even when row decoding fails.
Some valid packet families update metadata or remain raw-only and return no rows.

The iRacing adapter accepts the schema from `racehub-iracing-udp`.
The rFactor 2 adapter accepts the broadcaster's `DataPacket` schema.
These are different protocols. Select the matching simulator explicitly.
Graphics and extended-physics rFactor 2 packets can remain raw-only.

## Row and session contract

A projected row requires a string `participantId` and finite, non-negative `sessionTime` in seconds.
The writer accepts numeric, text, and Boolean channels.
Missing channels remain absent, then become Parquet nulls.
It does not replace unavailable measurements with invented values.
Use the [unit table](../README.md#channels-and-units) for the common channels.

Important fields include:

- `sessionId`: source identity as text, when available.
- `packetId`: numeric source packet family where supplied. F1 uses its packet IDs. rFactor 2 uses 0 for telemetry and 1 for scoring. The current iRacing adapter does not add this field.
- `frameId`: F1 frame identifier.
- `lap_number`, `lap_distance`, `lap_time`, `last_lap_time`, and `lap_invalid`: observed lap fields when available.
- `lap_source_time` and `lap_number_source_time`: the source time of carried lap data.
- `world_source_time`: the source time of world position data where supplied.

A copied lap value does not become a new measurement at another packet's time.
Charts must use the packet family and source time when they select measurements.
Keep F1 clutch percentages separate from iRacing and rFactor 2 clutch ratios.
Keep iRacing `steer_angle` in degrees.
Keep F1 fuel kilograms separate from rFactor 2 fuel litres.

The session manifest uses `format: "racehub-telemetry"` and `version: 1`.
It includes recording identity, simulator, name, creation time, track metadata,
`scope`, `captureRate: "source"`, `source_session_id`, `rowCount`, and `parts`.
Part entries contain `name`, `size` in bytes, and `rowCount`.
The final manifest also lists `rawFiles`, counters, state, and an error if present.
A Parquet footer entry named `racehub` contains a JSON metadata object for that part.
Use the final manifest for later track metadata and the complete part list.

The writer uses Float64, UTF-8, and Boolean Arrow columns.
It keeps at most 512 channels, 160 bytes per channel key, and 4,096 bytes per text value.
It flushes groups at 8,192 rows or an estimated 16 MiB decoded size.
It normally closes parts at 65,536 rows.
A schema addition can close a part early.
Each row group is sorted by participant ID and session time.
The full recording is not globally sorted.

## Receive and shutdown contract

The CLI has a 1,024-packet receive queue.
It does not sample packets or impose a total row limit.
A full queue stops capture and reports dropped packets.
That counter cannot detect earlier network or operating system loss.

The raw writer runs before row projection.
The CLI filters a selected F1 source UID before it mutates the decoder's session cache.
It stops projected capture on a different source session or a clock rewind greater than two seconds.
The rFactor 2 adapter also starts a new source generation after a rewind greater than one second.
Packet reordering must stay within both the decoder and recorder checks.
After a projection failure, accepted queued packets can still be saved as raw input.

`stop` on standard input, EOF, Ctrl+C, and Unix SIGTERM start an orderly stop.
The process closes UDP input, closes the sender, drains the accepted queue, and completes files.
The stdout protocol uses `ready`, `status`, `sample`, `complete`, and `error` JSON lines.
The full part list belongs in `session.json`, not in a large stdout message.
A completed session can contain an error. Check both events and final metadata.

## Native raw archive formats

Both native raw formats use `.rhc` files.
Read the magic bytes before you select a reader.
The Python collector's gzip archive is a separate format.

All multibyte archive integers use big-endian order.
The original game payload keeps its original byte order.

### F1TEL version 1

The native writer uses this format when the header identifies F1 24 and a nonzero session UID.
The legacy F1 recorder also writes this format.

| Header field | Bytes |
| --- | ---: |
| Magic `F1TEL` | 5 |
| Version, currently 1 | 1 |
| F1 session UID, unsigned integer | 8 |
| Start receive timestamp, Unix seconds | 8 |
| Reserved | 32 |

The header is 54 bytes long.

### RHRAW version 1

The native writer uses this format for the other simulators and unrecognized F1 headers.

| Header field | Bytes |
| --- | ---: |
| Magic `RHRAW` | 5 |
| Version, currently 1 | 1 |
| Simulator: 0 F1, 1 iRacing, 2 rFactor 2 | 1 |
| Start receive timestamp, Unix seconds | 8 |
| Reserved | 32 |

The header is 47 bytes long.

### Packet frames

Both formats then repeat this frame:

| Frame field | Bytes |
| --- | ---: |
| Receive timestamp, Unix microseconds | 8 |
| F1 packet ID, or 255 for the generic raw stream | 1 |
| Zstd-compressed payload length | 4 |
| One compressed original UDP datagram | Declared length |

The native reader rejects unsupported archive versions and compressed frames over 1 MiB.
Expanded datagrams cannot exceed 65,507 bytes.
Each archive stores one frame per received packet that reaches the writer.
The native writer permits at most 256 raw source files in one recording.

Raw buffers flush every 100 packets and sync at normal completion.
A crash can lose the last buffered writes.
A partial final frame is an error, not a clean end of file.
The converter completes the recovered prefix and reports the error.
It does not claim that missing bytes were recovered.
Use a new directory for each conversion.

### Line-delimited raw packet input

The native converter also accepts the format in
[`tests/recordings/2024.json`](tests/recordings/2024.json).
Each non-empty line is one JSON object.
`message.data` must be a non-empty array of byte values.
`time` must be an RFC 3339 receive timestamp or a Unix microsecond integer.
Each line is limited to 2 MiB.
The saved `parsed` object is not used for decoding.

The fixture interleaves source IDs and includes non-monotonic source times.
Its bytes and times must stay unchanged.
Use it for decoder tests or the benchmark command.
A native conversion still enforces the source ID and clock boundaries.

## Legacy Rust capture and C ABI

The older `TelemetryCapture` path captures F1 through its existing parser and provider.
It writes F1TEL files and SQLite frame tables under `telemetry_data` in the process working directory.
It binds UDP on `0.0.0.0` at the supplied port.
The native CLI instead uses its own recorder, raw writer, and Parquet store.
Its default bind address is `127.0.0.1`.

| Export | Contract |
| --- | --- |
| `create_telemetry_capture` | Creates a runtime and capture handle. Accepts a port, callback, and user pointer. Can return null. |
| `start_telemetry_capture` | Runs capture synchronously on that runtime. It blocks until capture stops. Returns success as a Boolean. |
| `destroy_telemetry_capture` | Destroys the handle and finalizes legacy recorder/database state. |
| `flush_and_close_db` | Flushes and closes the legacy SQLite connection. |

The callback receives bincode-serialized `F1TelemetryPacket` bytes.
They are not the original UDP packet or the canonical Parquet row.
Copy the callback buffer before the callback returns if you need to keep it.
Use matching enum and serialization versions in the caller.
Do not destroy a handle while its capture call is running.
The legacy stop path uses Ctrl+C.
The new CLI is the path integrated with RaceHub Web.

## Legacy F1TEL inspection tool

`capture::TelemetryReader::open` reads F1TEL version 1.
`read_next_packet` returns `PacketInfo` with `timestamp_us`, `packet_type`, and the original `data` bytes.
It reads one bounded packet at a time.
`read_all_packets` collects the full recording in memory.
The legacy reader does not accept RHRAW.

The `rhc2json` binary uses this reader and writes packet JSON.
Set `RACEHUB_RAW_FILE` to an actual F1TEL file path in your shell.
Then run from the repository root:

```sh
cargo run --release --locked -p capture --bin rhc2json -- --input "$RACEHUB_RAW_FILE"
cargo run --release --locked -p capture --bin rhc2json -- --input "$RACEHUB_RAW_FILE" --uncompressed
```

The first command writes Zstd-compressed `.json.zst`.
The second writes plain `.json`.
Use `--output` or `-o` for a different path.
Choose a new path because this legacy tool can overwrite its output.
Use `--compression` or `-c` for the Zstd level; the default is 3.
`--input` also has the short form `-i`.

The JSON object contains `metadata` and a `packets` array.
Metadata has the numeric source `session_id`, start `timestamp`, and `source_file` path.
Each packet has `timestamp_us`, `packet_type`, and `data`.
`data` uses Serde's external enum names.
For example, a telemetry packet's data is under `CarTelemetry`, then `m_carTelemetryData`.
These packet objects differ from the flat canonical rows written by the native CLI.
A JavaScript consumer must preserve the precision of numeric 64-bit source IDs.

The older root `converter.py` reads this packet JSON and writes per-family Parquet tables.
It needs the Python dependencies in `../pyproject.toml` and reads the input JSON into memory.
Its packet-prefixed columns differ from the native canonical columns.
Use the native `aggregator convert` command for the current Parquet recording path.
The older SQLite analysis script and notebook remain optional inspection tools.

## Validation and changes

Read [CONTRIBUTING.md](../CONTRIBUTING.md) before a change.
Run the locked workspace tests.
Use recorded, licensed input for positive decoder tests.
Use empty or malformed input for negative checks.
Keep source packet bytes and timestamps intact.

The F1 fixture and its MIT licence are in `tests/recordings`.
Current tests cover exact F1 packet fields, sparse rows and units, source boundaries,
file locks, raw bytes, truncated recovery, and multi-group Parquet parts.
The iRacing and rFactor 2 tests check invalid input and schema handling.
A successful build does not verify a live Windows game connection or a complete lap.
