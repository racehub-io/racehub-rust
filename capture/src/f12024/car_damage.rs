use crate::f12024::TelemetryError;
use common::f12024::car_damage::{PacketCarDamageData, parse_car_damage_packet_internal};

pub fn parse_car_damage(buf: &[u8]) -> Result<PacketCarDamageData, TelemetryError> {
    match parse_car_damage_packet_internal(buf) {
        Ok((_, packet)) => Ok(packet),
        Err(_) => Err(TelemetryError::ParseError("Failed to parse car damage packet")),
    }
} 