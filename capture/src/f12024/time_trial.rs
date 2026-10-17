use crate::f12024::TelemetryError;
use common::f12024::time_trial::{PacketTimeTrialData, parse_time_trial_packet_internal};

pub fn parse_time_trial(buf: &[u8]) -> Result<PacketTimeTrialData, TelemetryError> {
    match parse_time_trial_packet_internal(buf) {
        Ok(([], packet)) => Ok(packet),
        _ => Err(TelemetryError::ParseError("Failed to parse time trial packet")),
    }
} 