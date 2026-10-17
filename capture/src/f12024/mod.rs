pub mod header;
pub mod session;
pub mod lap;
pub mod event;
pub mod car_telemetry;
pub mod car_status;
pub mod car_motion;
pub mod participants;
pub mod car_setup;
pub mod final_classification;
pub mod lobby_info;
pub mod car_damage;
pub mod session_history;
pub mod tyre_sets;
pub mod car_motion_ex;
pub mod time_trial;
pub mod telemetry;

use std::fmt::Debug;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use common::f12024::F1TelemetryPacket;
use crate::telemetry::TelemetryProvider;
use crate::f12024::telemetry::{F12024Provider, F12024Metadata};

#[derive(Debug)]
pub enum TelemetryError {
    ParseError(&'static str),
    Other(String),
}

pub trait TelemetryPacket: Debug + Send + 'static {
    /// Parse the telemetry packet from a byte slice.
    fn from_bytes(buf: &[u8]) -> Result<Self, TelemetryError>
    where
        Self: Sized;
}

impl TelemetryPacket for F1TelemetryPacket {
    fn from_bytes(buf: &[u8]) -> Result<Self, TelemetryError> {
        let header = header::parse_header(buf)?;
        
        match header.m_packetId {
            0 => {
                match car_motion::parse_car_motion(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::CarMotion(packet)),
                    Err(e) => Err(e),
                }
            }
            1 => {
                match session::parse_session(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::Session(packet)),
                    Err(e) => Err(e),
                }
            }
            2 => {
                match lap::parse_lap_data(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::Lap(packet)),
                    Err(e) => Err(e),
                }
            }
            3 => {
                match event::parse_event(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::Event(packet)),
                    Err(e) => Err(e),
                }
            }
            4 => {
                match participants::parse_participants(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::Participants(packet)),
                    Err(e) => Err(e),
                }
            }
            5 => {
                match car_setup::parse_car_setup(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::CarSetup(packet)),
                    Err(e) => Err(e),
                }
            }
            6 => {
                match car_telemetry::parse_car_telemetry(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::CarTelemetry(packet)),
                    Err(e) => Err(e),
                }
            }
            7 => {
                match car_status::parse_car_status(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::CarStatus(packet)),
                    Err(e) => Err(e),
                }
            }
            8 => {
                match final_classification::parse_final_classification(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::FinalClassification(packet)),
                    Err(e) => Err(e),
                }
            }
            9 => {
                match lobby_info::parse_lobby_info(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::LobbyInfo(packet)),
                    Err(e) => Err(e),
                }
            }
            10 => {
                match car_damage::parse_car_damage(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::CarDamage(packet)),
                    Err(e) => Err(e),
                }
            }
            11 => {
                match session_history::parse_session_history(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::SessionHistory(packet)),
                    Err(e) => Err(e),
                }
            }
            12 => {
                match tyre_sets::parse_tyre_sets(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::TyreSets(packet)),
                    Err(e) => Err(e),
                }
            }
            13 => {
                match car_motion_ex::parse_car_motion_ex(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::CarMotionEx(packet)),
                    Err(e) => Err(e),
                }
            }
            14 => {
                match time_trial::parse_time_trial(buf) {
                    Ok(packet) => Ok(F1TelemetryPacket::TimeTrial(packet)),
                    Err(e) => Err(e),
                }
            }
            _ => Ok(F1TelemetryPacket::Unknown(header)),
        }
    }
}

pub struct TelemetryCapture {
    port: u16,
    callback: Arc<dyn Fn(F1TelemetryPacket) + Send + Sync>,
}

impl TelemetryCapture {
    pub fn new(port: u16, callback: impl Fn(F1TelemetryPacket) + Send + Sync + 'static) -> Self {
        Self {
            port,
            callback: Arc::new(callback),
        }
    }

    async fn store_packet(packet: &F1TelemetryPacket, raw_data: &[u8]) {
        // Extract packet type
        let packet_id = match packet {
            F1TelemetryPacket::CarTelemetry(_) => 6,
            F1TelemetryPacket::CarStatus(_) => 7,
            F1TelemetryPacket::CarMotion(_) => 0,
            F1TelemetryPacket::Lap(_) => 2,
            F1TelemetryPacket::Event(_) => 3,
            F1TelemetryPacket::Participants(_) => 4,
            F1TelemetryPacket::CarSetup(_) => 5,
            F1TelemetryPacket::Session(_) => 1,
            F1TelemetryPacket::FinalClassification(_) => 8,
            F1TelemetryPacket::LobbyInfo(_) => 9,
            F1TelemetryPacket::CarDamage(_) => 10,
            F1TelemetryPacket::SessionHistory(_) => 11,
            F1TelemetryPacket::TyreSets(_) => 12,
            F1TelemetryPacket::CarMotionEx(_) => 13,
            F1TelemetryPacket::TimeTrial(_) => 14,
            F1TelemetryPacket::Unknown(_) => 255,
        };
        
        // Extract session ID and other metadata
        let Some((frame_id, session_time, session_id)) = F12024Metadata::get_packet_metadata(packet) else {
            return; // Skip if we can't get packet metadata
        };
        
        // Skip packets with session_id == 0
        if session_id == 0 {
            return;
        }
        
        // Check if we need to initialize a new recording session
        let mut new_session = false;
        let mut db_filename = None;
        
        // Handle recorder initialization first
        if let Ok(mut recorder) = crate::recorder::RECORDER.lock() {
            if recorder.session_id.is_none() || recorder.session_id != Some(session_id) {
                // New session detected - initialize recorder
                if let Err(e) = recorder.initialize(session_id) {
                    eprintln!("Failed to initialize telemetry recorder: {:?}", e);
                } else {
                    new_session = true;
                    if let Some(path) = recorder.get_file_path() {
                        let rhc_path = path.to_string_lossy().to_string();
                        db_filename = Some(rhc_path.replace(".rhc", ".db"));
                    }
                }
            }
            
            // Record the packet
            let timestamp = crate::recorder::timestamp_us();
            if let Err(e) = recorder.write_packet(packet_id, timestamp, raw_data) {
                eprintln!("Failed to record telemetry packet: {:?}", e);
            }
        }
        
        // Handle database operations
        if let Ok(mut db) = crate::db::DB.lock() {
            // Initialize database with same filename if this is a new session
            if new_session && db_filename.is_some() {
                if let Err(e) = db.initialize_db(&db_filename.unwrap()) {
                    eprintln!("Failed to initialize database: {}", e);
                }
            }
            
            // Process the frame data
            if let Err(e) = db.handle_new_frame(session_id, session_time, frame_id) {
                eprintln!("Failed to handle new frame: {}", e);
            }
            
            // Process telemetry data
            let provider = F12024Provider;
            if let Ok(car_data) = provider.get_car_telemetry(packet) {
                if !car_data.is_empty() {
                    if let Err(e) = db.update_frame_with_telemetry(frame_id, car_data) {
                        eprintln!("Failed to update frame with telemetry: {}", e);
                    }
                }
            }
        }
    }

    pub async fn run(&self) -> Result<(), TelemetryError> {
        let addr = format!("0.0.0.0:{}", self.port);
        let socket = UdpSocket::bind(&addr)
            .await
            .map_err(|e| TelemetryError::Other(e.to_string()))?;

        let (tx, mut rx) = mpsc::channel::<(F1TelemetryPacket, Vec<u8>)>(100);
        let callback = self.callback.clone();
        
        // Spawn a worker task to handle processing packets
        let processor = tokio::spawn(async move {
            while let Some((packet, raw_data)) = rx.recv().await {
                Self::store_packet(&packet, &raw_data).await;
                (callback)(packet);
            }
        });

        // Setup Ctrl+C handler
        let mut buf = [0u8; 2048];
        println!("Listening on {}", addr);
        
        loop {
            tokio::select! {
                // Process incoming UDP packet
                result = socket.recv_from(&mut buf) => {
                    match result {
                        Ok((n, _peer)) => {
                            let raw_data = buf[..n].to_vec();
                            
                            match F1TelemetryPacket::from_bytes(&raw_data) {
                                Ok(packet) => {
                                    // Check session_id and skip processing if it's 0
                                    if let Some(session_id) = F12024Metadata::get_session_id(&packet) {
                                        if session_id == 0 {
                                            // Skip processing for session_id == 0
                                            continue;
                                        }
                                    }
                                    
                                    if tx.send((packet, raw_data)).await.is_err() {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse packet: {:?}", e);
                                }
                            }
                        },
                        Err(e) => {
                            return Err(TelemetryError::Other(e.to_string()));
                        }
                    }
                },
                
                // Handle shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    println!("Received Ctrl+C signal, shutting down gracefully...");
                    break;
                }
            }
        }
        
        // Clean up resources
        drop(tx); // This will cause the processor task to exit

        // Drain the queued packets before the files close.
        if let Err(e) = processor.await {
            eprintln!("Error waiting for processor task: {:?}", e);
        }
        
        // Make sure database and recorder are properly flushed and closed
        if let Ok(mut db) = crate::db::DB.lock() {
            if let Err(e) = db.flush_and_close() {
                eprintln!("Error flushing database: {}", e);
            }
        }
        
        if let Ok(mut recorder) = crate::recorder::RECORDER.lock() {
            if let Err(e) = recorder.finalize() {
                eprintln!("Error finalizing recorder: {}", e);
            }
        }
        
        println!("Telemetry capture shutdown complete.");
        Ok(())
    }
}
