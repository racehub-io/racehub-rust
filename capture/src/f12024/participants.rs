use crate::f12024::TelemetryError;
use common::f12024::participants::{PacketParticipantsData, parse_participants_packet_internal};

pub fn parse_participants(buf: &[u8]) -> Result<PacketParticipantsData, TelemetryError> {
    match parse_participants_packet_internal(buf) {
        Ok(([], packet)) => Ok(packet),
        _ => Err(TelemetryError::ParseError("Failed to parse participants packet")),
    }
} 