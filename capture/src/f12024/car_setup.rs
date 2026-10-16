use crate::f12024::TelemetryError;
use common::f12024::car_setup::{PacketCarSetupData, parse_car_setup_packet_internal};

pub fn parse_car_setup(buf: &[u8]) -> Result<PacketCarSetupData, TelemetryError> {
    match parse_car_setup_packet_internal(buf) {
        Ok((_, packet)) => Ok(packet),
        Err(_) => Err(TelemetryError::ParseError("Failed to parse car setup packet")),
    }
} 