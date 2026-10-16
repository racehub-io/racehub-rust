use crate::f12024::TelemetryError;
use common::f12024::car_motion_ex::{PacketMotionExData, parse_car_motion_ex_packet_internal};

pub fn parse_car_motion_ex(buf: &[u8]) -> Result<PacketMotionExData, TelemetryError> {
    match parse_car_motion_ex_packet_internal(buf) {
        Ok((_, packet)) => Ok(packet),
        Err(_) => Err(TelemetryError::ParseError("Failed to parse motion ex packet")),
    }
} 