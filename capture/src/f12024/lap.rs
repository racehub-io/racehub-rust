use crate::f12024::TelemetryError;
use common::f12024::lap::{PacketLapData, parse_lap_data_packet_internal};

pub fn parse_lap_data(buf: &[u8]) -> Result<PacketLapData, TelemetryError> {
    match parse_lap_data_packet_internal(buf) {
        Ok((_, packet)) => Ok(packet),
        Err(_) => Err(TelemetryError::ParseError("Failed to parse lap data packet")),
    }
} 