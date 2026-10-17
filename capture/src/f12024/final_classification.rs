use crate::f12024::TelemetryError;
use common::f12024::final_classification::{PacketFinalClassificationData, parse_final_classification_packet_internal};

pub fn parse_final_classification(buf: &[u8]) -> Result<PacketFinalClassificationData, TelemetryError> {
    match parse_final_classification_packet_internal(buf) {
        Ok(([], packet)) => Ok(packet),
        _ => Err(TelemetryError::ParseError("Failed to parse final classification packet")),
    }
} 