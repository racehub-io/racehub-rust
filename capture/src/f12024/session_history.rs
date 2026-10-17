use crate::f12024::TelemetryError;
use common::f12024::session_history::{PacketSessionHistoryData, parse_session_history_packet_internal};

pub fn parse_session_history(buf: &[u8]) -> Result<PacketSessionHistoryData, TelemetryError> {
    match parse_session_history_packet_internal(buf) {
        Ok(([], packet)) => Ok(packet),
        _ => Err(TelemetryError::ParseError("Failed to parse session history packet")),
    }
} 