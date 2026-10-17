use crate::f12024::TelemetryError;
use common::f12024::session::{PacketSessionData, parse_session_packet_internal};

pub fn parse_session(buf: &[u8]) -> Result<PacketSessionData, TelemetryError> {
    match parse_session_packet_internal(buf) {
        Ok(([], packet)) => Ok(packet),
        _ => Err(TelemetryError::ParseError("Failed to parse session packet")),
    }
} 