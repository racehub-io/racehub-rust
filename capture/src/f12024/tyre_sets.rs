use crate::f12024::TelemetryError;
use common::f12024::tyre_sets::{PacketTyreSetsData, parse_tyre_sets_packet_internal};

pub fn parse_tyre_sets(buf: &[u8]) -> Result<PacketTyreSetsData, TelemetryError> {
    match parse_tyre_sets_packet_internal(buf) {
        Ok((_, packet)) => Ok(packet),
        Err(_) => Err(TelemetryError::ParseError("Failed to parse tyre sets packet")),
    }
} 