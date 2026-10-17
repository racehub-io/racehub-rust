use crate::f12024::TelemetryError;
use common::f12024::car_telemetry::{PacketCarTelemetryData, parse_car_telemetry_packet_internal};

pub fn parse_car_telemetry(buf: &[u8]) -> Result<PacketCarTelemetryData, TelemetryError> {
    match parse_car_telemetry_packet_internal(buf) {
        Ok(([], packet)) => Ok(packet),
        _ => Err(TelemetryError::ParseError("Failed to parse car telemetry packet")),
    }
} 