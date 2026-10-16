use crate::f12024::TelemetryError;
use common::f12024::lobby_info::{PacketLobbyInfoData, parse_lobby_info_packet_internal};

pub fn parse_lobby_info(buf: &[u8]) -> Result<PacketLobbyInfoData, TelemetryError> {
    match parse_lobby_info_packet_internal(buf) {
        Ok((_, packet)) => Ok(packet),
        Err(_) => Err(TelemetryError::ParseError("Failed to parse lobby info packet")),
    }
} 