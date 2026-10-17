use capture::{F1TelemetryPacket, TelemetryPacket, native::F1Decoder};
use serde_json::Value;

fn recordings() -> Vec<Value> {
    include_str!("recordings/2024.json")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
fn bytes(record: &Value) -> Vec<u8> {
    record["message"]["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_u64().unwrap() as u8)
        .collect()
}
fn compare(actual: &Value, expected: &Value, path: &str) {
    match (actual, expected) {
        (Value::Object(actual), Value::Object(expected)) => {
            for (key, value) in expected {
                let wire_key = if key.starts_with("m_sector") || key.starts_with("m_deltaTo") {
                    key.replace("TimeInMS", "TimeMSPart")
                        .replace("TimeMinutes", "TimeMinutesPart")
                        .replace("InMS", "MSPart")
                        .replace("FrontMinutes", "FrontMinutesPart")
                        .replace("LeaderMinutes", "LeaderMinutesPart")
                } else {
                    key.clone()
                };
                compare(
                    actual
                        .get(key)
                        .or_else(|| actual.get(&wire_key))
                        .unwrap_or_else(|| panic!("Missing {path}.{key}")),
                    value,
                    &format!("{path}.{key}"),
                );
            }
        }
        (Value::Array(actual), Value::Array(expected)) => {
            assert_eq!(actual.len(), expected.len(), "{path}");
            for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
                compare(actual, expected, &format!("{path}[{index}]"));
            }
        }
        (Value::Number(actual), Value::Number(expected)) => {
            let (actual, expected) = (actual.as_f64().unwrap(), expected.as_f64().unwrap());
            assert!(
                (actual - expected).abs() <= 1e-6 * expected.abs().max(1.0),
                "{path}: {actual} != {expected}"
            );
        }
        (Value::Number(actual), Value::String(expected)) if path.ends_with("m_sessionUID") => {
            assert_eq!(actual.to_string(), *expected)
        }
        _ => assert_eq!(actual, expected, "{path}"),
    }
}
#[test]
fn actual_f124_packets_match_the_published_parser() {
    for record in recordings() {
        let bytes = bytes(&record);
        let packet = F1TelemetryPacket::from_bytes(&bytes).unwrap();
        let value = serde_json::to_value(packet).unwrap();
        let actual = value.as_object().unwrap().values().next().unwrap();
        if bytes[6] == 3 {
            compare(
                &actual["m_header"],
                &record["parsed"]["m_header"],
                "event.header",
            );
            compare(
                &actual["m_eventStringCode"],
                &record["parsed"]["m_eventStringCode"],
                "event.code",
            );
        } else {
            compare(
                actual,
                &record["parsed"],
                record["packetID"].as_str().unwrap(),
            );
        }
    }
}
#[test]
fn actual_player_channels_keep_their_source_times_and_units() {
    let records = recordings();
    let telemetry = records
        .iter()
        .find(|record| record["packetID"] == "carTelemetry")
        .unwrap();
    let packet = bytes(telemetry);
    let player = packet[27] as usize;
    let mut decoder = F1Decoder::default();
    let output = decoder.decode(&packet).unwrap();
    let row = output
        .rows
        .iter()
        .find(|row| row["carIndex"] == player)
        .unwrap();
    let expected = &telemetry["parsed"]["m_carTelemetryData"][player];
    assert_eq!(row["engine_rpm"], expected["m_engineRPM"]);
    assert_eq!(row["speed"], expected["m_speed"]);
    assert_eq!(
        row["sessionId"],
        telemetry["parsed"]["m_header"]["m_sessionUID"]
    );
    assert_eq!(
        row["sessionTime"],
        telemetry["parsed"]["m_header"]["m_sessionTime"]
    );
    assert!(!row.contains_key("lap_number"));
    assert!(!row.contains_key("world_x"));
    let lap = records
        .iter()
        .find(|record| {
            record["packetID"] == "lapData"
                && record["parsed"]["m_header"]["m_sessionUID"]
                    == telemetry["parsed"]["m_header"]["m_sessionUID"]
        })
        .unwrap();
    decoder.decode(&bytes(lap)).unwrap();
    let output = decoder.decode(&packet).unwrap();
    assert!(
        !output.rows[0].contains_key("lap_number"),
        "A later lap packet must not give an earlier measurement a lap number."
    );
}
#[test]
fn invalid_lengths_formats_and_empty_packets_are_rejected() {
    let packet = bytes(&recordings()[7]);
    let mut decoder = F1Decoder::default();
    assert!(decoder.decode(&[]).is_err());
    assert!(decoder.decode(&packet[..packet.len() - 1]).is_err());
    let mut extra = packet.clone();
    extra.push(0);
    assert!(decoder.decode(&extra).is_err());
    let mut unsupported = packet;
    unsupported[2] = 25;
    assert!(
        decoder
            .decode(&unsupported)
            .unwrap_err()
            .contains("2024 only")
    );
}
