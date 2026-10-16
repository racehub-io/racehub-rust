use crate::f12024::TelemetryError;
use common::f12024::header::PacketHeader;
use common::f12024::header::parse_header_internal;

pub fn parse_header(buf: &[u8]) -> Result<PacketHeader, TelemetryError> {
    parse_header_internal(buf)
        .map(|(_, header)| header)
        .map_err(|_| TelemetryError::ParseError("Failed to parse packet header"))
} 