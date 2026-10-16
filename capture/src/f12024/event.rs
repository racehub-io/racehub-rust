use crate::f12024::TelemetryError;
use common::f12024::event::PacketEventData;
use common::f12024::event::parse_event_packet_internal;

pub fn parse_event(buf: &[u8]) -> Result<PacketEventData, TelemetryError> {
    match parse_event_packet_internal(buf) {
        Ok((_, event)) => Ok(event),
        Err(_) => Err(TelemetryError::ParseError("Failed to parse packet event"))
    }
} 