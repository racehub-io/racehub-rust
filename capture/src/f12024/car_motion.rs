use crate::f12024::TelemetryError;
use common::f12024::car_motion::{PacketCarMotionData, parse_car_motion_packet_internal};

pub fn parse_car_motion(buf: &[u8]) -> Result<PacketCarMotionData, TelemetryError> {
    match parse_car_motion_packet_internal(buf) {
        Ok((_, packet)) => Ok(packet),
        Err(_) => Err(TelemetryError::ParseError("Failed to parse car motion packet")),
    }
} 