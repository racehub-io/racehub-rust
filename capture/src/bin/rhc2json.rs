#![recursion_limit = "512"]

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

use anyhow::{Result, Context, bail};
use clap::Parser;
use common::f12024::F1TelemetryPacket;
use capture::TelemetryPacket;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(author, version, about = "Convert RHC telemetry files to JSON")]
struct Args {
    /// Input RHC file
    #[arg(short, long)]
    input: PathBuf,

    /// Output JSON file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Compression level (1-22)
    #[arg(short, long, default_value_t = 3)]
    compression: i32,
    
    /// Output uncompressed JSON (default is compressed with zstd)
    #[arg(short, long)]
    uncompressed: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Validate input file exists and has .rhc extension
    if !args.input.exists() {
        bail!("Input file does not exist: {:?}", args.input);
    }
    
    if args.input.extension().and_then(|e| e.to_str()) != Some("rhc") {
        bail!("Input file must have .rhc extension");
    }
    
    // Determine output file name if not provided
    let output_path = match args.output {
        Some(path) => path,
        None => {
            let input_stem = args.input.file_stem().unwrap_or_default();
            let mut output = args.input.with_file_name(input_stem);
            if args.uncompressed {
                output.set_extension("json");
            } else {
                output.set_extension("json.zst");
            }
            output
        },
    };
    
    println!("Converting {:?} to {:?}", args.input, output_path);
    if args.uncompressed {
        println!("Generating uncompressed JSON");
    } else {
        println!("Using zstd compression (level {})", args.compression);
    }
    
    let start_time = Instant::now();
    let packet_count = process_file(&args.input, &output_path, !args.uncompressed, args.compression)?;
    let duration = start_time.elapsed();
    
    println!("Conversion complete.");
    println!("Processed {} packets in {:.2?}", packet_count, duration);
    
    Ok(())
}

fn process_file(input_path: &Path, output_path: &Path, use_compression: bool, compression_level: i32) -> Result<usize> {
    // Open telemetry file for reading
    let mut reader = capture::TelemetryReader::open(input_path)
        .context("Failed to open telemetry file")?;
    
    println!("Session ID: {}", reader.session_id());
    
    // Open output file with appropriate compression
    let file = File::create(output_path).context("Failed to create output file")?;
    let mut writer = BufWriter::new(file);

    // Create the appropriate writer based on compression setting
    let mut total_bytes = 0;
    
    if use_compression {
        // Use zstd compression
        let mut compressed_writer = zstd::Encoder::new(writer, compression_level)
            .context("Failed to create zstd encoder")?;
        let packet_count = write_json_data(&mut compressed_writer, &mut reader, &mut total_bytes)?;
        compressed_writer.finish()?;
        println!("\rProcessed {} packets, {} bytes written.", packet_count, total_bytes);
        Ok(packet_count)
    } else {
        // No compression
        let packet_count = write_json_data(&mut writer, &mut reader, &mut total_bytes)?;
        println!("\rProcessed {} packets, {} bytes written.", packet_count, total_bytes);
        Ok(packet_count)
    }
}

fn write_json_data<W: Write>(writer: &mut W, reader: &mut capture::TelemetryReader, total_bytes: &mut usize) -> Result<usize> {
    // Write JSON file header
    writer.write_all(b"{")?;
    writer.write_all(b"\n  \"metadata\": ")?;
    
    // Write metadata
    let metadata = json!({
        "session_id": reader.session_id(),
        "timestamp": reader.file_timestamp(),
        "source_file": reader.source_file().to_string_lossy(),
    });
    let metadata_str = metadata.to_string();
    writer.write_all(metadata_str.as_bytes())?;
    *total_bytes += metadata_str.len();
    
    // Start packets array
    writer.write_all(b",\n  \"packets\": [\n")?;
    *total_bytes += 19; // Accounting for the JSON structure bytes
    
    // Process all packets
    let mut packet_count = 0;
    let mut first_packet = true;
    
    while let Some(packet_info) = reader.read_next_packet()? {
        if !first_packet {
            writer.write_all(b",\n")?;
            *total_bytes += 2;
        } else {
            first_packet = false;
        }
        
        // Parse the raw packet data
        let parsed_packet = F1TelemetryPacket::from_bytes(&packet_info.data)
            .map_err(|e| anyhow::anyhow!("Failed to parse packet: {:?}", e))?;
        
        // For event packets, ensure we use the properly parsed event details
        let packet_json = json!({
            "timestamp_us": packet_info.timestamp_us,
            "packet_type": packet_info.packet_type,
            "data": parsed_packet,
        });
        
        let json_str = packet_json.to_string();
        writer.write_all(json_str.as_bytes())?;
        *total_bytes += json_str.len();
        packet_count += 1;
        
        // Print progress every 1000 packets
        if packet_count % 1000 == 0 {
            print!("\rProcessed {} packets...", packet_count);
            std::io::stdout().flush()?;
        }
    }
    
    // Close the JSON structure
    writer.write_all(b"\n  ]\n}")?;
    *total_bytes += 7;
    
    Ok(packet_count)
} 