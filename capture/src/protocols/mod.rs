mod iracing;
mod rfactor2;

#[allow(dead_code)]
mod generated {
    pub mod iracing {
        include!("generated/iracing.rs");
    }
    pub mod rfactor2 {
        include!("generated/r_factor2_udp_broadcaster.rs");
    }
}

pub type TelemetryRow = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Default)]
pub struct DecodedPacket {
    pub rows: Vec<TelemetryRow>,
    pub player_participant_id: Option<String>,
    pub session_key: Option<String>,
}

#[derive(Default)]
pub struct ProtobufAdapters {
    rfactor2: rfactor2::Adapter,
}

impl ProtobufAdapters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iracing(&mut self, bytes: &[u8]) -> Result<DecodedPacket, String> {
        check_datagram(bytes)?;
        iracing::decode(bytes)
    }

    pub fn rfactor2(&mut self, bytes: &[u8]) -> Result<DecodedPacket, String> {
        check_datagram(bytes)?;
        self.rfactor2.decode(bytes)
    }
}

fn check_datagram(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > 65_507 {
        Err("The telemetry datagram has an invalid length.".into())
    } else {
        Ok(())
    }
}

fn number(row: &mut TelemetryRow, key: &str, value: Option<f64>) {
    if let Some(value) = value.and_then(serde_json::Number::from_f64) {
        row.insert(key.into(), value.into());
    }
}

fn text(row: &mut TelemetryRow, key: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        row.insert(
            key.into(),
            value.chars().take(160).collect::<String>().into(),
        );
    }
}

fn control(value: f64, minimum: f64) -> Option<f64> {
    value.is_finite().then(|| value.clamp(minimum, 1.0))
}

#[cfg(test)]
mod tests {
    use super::ProtobufAdapters;

    #[test]
    fn empty_oversized_and_malformed_datagrams_are_rejected() {
        let mut adapters = ProtobufAdapters::new();
        for bytes in [&[][..], &[0xff][..], &[0][..], &vec![0; 65_508][..]] {
            assert!(adapters.iracing(bytes).is_err());
            assert!(adapters.rfactor2(bytes).is_err());
        }
    }

    #[test]
    fn missing_packet_bodies_are_rejected() {
        let mut adapters = ProtobufAdapters::new();
        assert!(adapters.iracing(&[0x08, 0x01]).is_err());
        assert!(adapters.rfactor2(&[0x08, 0x01]).is_err());
    }
}
