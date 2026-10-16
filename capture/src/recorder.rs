use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chrono::{DateTime, Utc};
use thiserror::Error;


// Magic bytes that identify our telemetry file format
const MAGIC_BYTES: &[u8; 5] = b"F1TEL";
const FORMAT_VERSION: u8 = 1;

#[derive(Debug, Error)]
pub enum RecorderError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Compression error: {0}")]
    Compression(String),
    
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Recorder not initialized")]
    NotInitialized,
    
    #[error("Invalid file format")]
    InvalidFormat,
}

pub struct TelemetryRecorder {
    file: Option<BufWriter<File>>,
    file_path: Option<PathBuf>,
    pub session_id: Option<u64>,
    packet_count: u32,
    total_bytes: u64,
    start_time: DateTime<Utc>,
    compression_level: i32,
}

impl TelemetryRecorder {
    pub fn new(compression_level: i32) -> Self {
        Self {
            file: None,
            file_path: None,
            session_id: None,
            packet_count: 0,
            total_bytes: 0,
            start_time: Utc::now(),
            compression_level,
        }
    }
    
    pub fn initialize(&mut self, session_id: u64) -> Result<(), RecorderError> {
        // Create directory structure if it doesn't exist
        let data_dir = "telemetry_data";
        fs::create_dir_all(data_dir)?;
        
        // Generate a filename based on date and session ID
        let now = Utc::now();
        let timestamp = self.start_time.timestamp_millis() as u64;
        let path = format!(
            "{}/{}_session_{}.rhc",
            data_dir,
            timestamp,
            session_id
        );
        
        // Open the file for writing
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        
        let mut writer = BufWriter::new(file);
        
        // Write file header
        writer.write_all(MAGIC_BYTES)?;
        writer.write_u8(FORMAT_VERSION)?;
        writer.write_u64::<BigEndian>(session_id)?;
        
        // Write timestamp as seconds since epoch
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        writer.write_u64::<BigEndian>(timestamp)?;
        
        // Write reserved space (32 bytes) for future use
        writer.write_all(&[0u8; 32])?;
        
        // Store the file for future writes
        self.file = Some(writer);
        self.file_path = Some(PathBuf::from(path.clone()));
        self.session_id = Some(session_id);
        self.packet_count = 0;
        self.total_bytes = 0;
        self.start_time = now;
        
        Ok(())
    }
    
    pub fn write_packet(&mut self, packet_type: u8, timestamp_us: u64, data: &[u8]) -> Result<(), RecorderError> {
        // Make sure the file is initialized
        let writer = match &mut self.file {
            Some(w) => w,
            None => return Err(RecorderError::NotInitialized),
        };
        
        // Compress the packet data using zstd
        let compressed_data = zstd::encode_all(data, self.compression_level)
            .map_err(|e| RecorderError::Compression(e.to_string()))?;
        
        // Write packet header: timestamp (8 bytes), packet type (1 byte), and compressed size (4 bytes)
        writer.write_u64::<BigEndian>(timestamp_us)?;
        writer.write_u8(packet_type)?;
        
        // Write length of compressed data
        writer.write_u32::<BigEndian>(compressed_data.len() as u32)?;
        
        // Write compressed data
        writer.write_all(&compressed_data)?;
        
        // Update statistics
        self.packet_count += 1;
        self.total_bytes += (8 + 1 + 4 + compressed_data.len()) as u64; // header + data
        
        // Every 100 packets, flush to disk to prevent data loss
        if self.packet_count % 100 == 0 {
            writer.flush()?;
        }
        
        Ok(())
    }
    
    // Return the current file path
    pub fn get_file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }
    
    pub fn finalize(&mut self) -> Result<(), RecorderError> {
        if let Some(mut writer) = self.file.take() {
            // Ensure all data is written to disk
            writer.flush()?;
            
            // Reset state
            self.file_path = None;
            self.session_id = None;
        }
        
        Ok(())
    }
}

impl Drop for TelemetryRecorder {
    fn drop(&mut self) {
        if self.file.is_some() {
            // Attempt to finalize on drop, but ignore errors
            let _ = self.finalize();
        }
    }
}

// Create a globally accessible recorder instance
lazy_static::lazy_static! {
    pub static ref RECORDER: Arc<Mutex<TelemetryRecorder>> = 
        Arc::new(Mutex::new(TelemetryRecorder::new(3))); // Default compression level of 3
}

// Utility function to get microseconds since Unix epoch
pub fn timestamp_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

pub struct TelemetryReader {
    file: BufReader<File>,
    session_id: u64,
    file_timestamp: u64,
    source_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PacketInfo {
    pub timestamp_us: u64,
    pub packet_type: u8,
    pub data: Vec<u8>,
}

impl TelemetryReader {
    pub fn open(file_path: impl AsRef<Path>) -> Result<Self, RecorderError> {
        let file = File::open(&file_path)?;
        let mut reader = BufReader::new(file);
        
        // Read and verify magic bytes
        let mut magic = [0u8; 5];
        reader.read_exact(&mut magic)?;
        if magic != *MAGIC_BYTES {
            return Err(RecorderError::InvalidFormat);
        }
        
        // Read version
        let version = reader.read_u8()?;
        if version != FORMAT_VERSION {
            eprintln!("Warning: Reading telemetry file with version {}, current version is {}", 
                     version, FORMAT_VERSION);
        }
        
        // Read session ID and timestamp
        let session_id = reader.read_u64::<BigEndian>()?;
        let file_timestamp = reader.read_u64::<BigEndian>()?;
        
        // Skip reserved space
        let mut reserved = [0u8; 32];
        reader.read_exact(&mut reserved)?;
        
        Ok(Self {
            file: reader,
            session_id,
            file_timestamp,
            source_path: file_path.as_ref().to_path_buf(),
        })
    }
    
    pub fn session_id(&self) -> u64 {
        self.session_id
    }
    
    pub fn file_timestamp(&self) -> u64 {
        self.file_timestamp
    }
    
    pub fn source_file(&self) -> &Path {
        &self.source_path
    }
    
    pub fn read_next_packet(&mut self) -> Result<Option<PacketInfo>, RecorderError> {
        // Try to read packet header
        let timestamp_us = match self.file.read_u64::<BigEndian>() {
            Ok(ts) => ts,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        
        // Read packet type and data length
        let packet_type = self.file.read_u8()?;
        let data_len = self.file.read_u32::<BigEndian>()? as usize;
        
        // Read compressed data
        let mut compressed_data = vec![0u8; data_len];
        self.file.read_exact(&mut compressed_data)?;
        
        // Decompress data
        let data = zstd::decode_all(&compressed_data[..])
            .map_err(|e| RecorderError::Compression(e.to_string()))?;
        
        Ok(Some(PacketInfo {
            timestamp_us,
            packet_type,
            data,
        }))
    }
    
    pub fn read_all_packets(&mut self) -> Result<Vec<PacketInfo>, RecorderError> {
        let mut packets = Vec::new();
        
        while let Some(packet) = self.read_next_packet()? {
            packets.push(packet);
        }
        
        Ok(packets)
    }
}