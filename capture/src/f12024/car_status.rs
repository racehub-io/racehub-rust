use crate::f12024::TelemetryError;
use common::f12024::car_status::{PacketCarStatusData, parse_car_status_packet_internal};

pub fn parse_car_status(buf: &[u8]) -> Result<PacketCarStatusData, TelemetryError> {
    match parse_car_status_packet_internal(buf) {
        Ok((_, packet)) => Ok(packet),
        Err(_) => Err(TelemetryError::ParseError("Failed to parse car status packet")),
    }
} 