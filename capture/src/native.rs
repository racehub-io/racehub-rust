use crate::{
    TelemetryPacket,
    protocols::{DecodedPacket, ProtobufAdapters, TelemetryRow},
};
use common::f12024::{self as f1, F1TelemetryPacket};
use serde::Serialize;
use serde_json::{Map, Value, json};

pub enum Decoder {
    F1(F1Decoder),
    IRacing(ProtobufAdapters),
    RFactor2(ProtobufAdapters),
}

impl Decoder {
    pub fn new(simulator: &str) -> Result<Self, String> {
        match simulator {
            "f1" => Ok(Self::F1(F1Decoder::default())),
            "iracing" => Ok(Self::IRacing(ProtobufAdapters::new())),
            "rfactor2" => Ok(Self::RFactor2(ProtobufAdapters::new())),
            _ => Err("Select f1, iracing, or rfactor2.".into()),
        }
    }

    pub fn decode(&mut self, bytes: &[u8]) -> Result<DecodedPacket, String> {
        match self {
            Self::F1(decoder) => decoder.decode(bytes),
            Self::IRacing(decoder) => decoder.iracing(bytes),
            Self::RFactor2(decoder) => decoder.rfactor2(bytes),
        }
    }
}

#[derive(Default)]
pub struct F1Decoder {
    uid: Option<u64>,
    participants: Vec<TelemetryRow>,
    visibility: Vec<bool>,
    laps: Vec<TelemetryRow>,
    session: TelemetryRow,
}

fn snake(source: &str) -> String {
    let chars: Vec<char> = source.trim_start_matches("m_").chars().collect();
    let mut result = String::new();
    for (index, character) in chars.iter().enumerate() {
        if character.is_uppercase()
            && index > 0
            && (chars[index - 1].is_lowercase()
                || chars[index - 1].is_ascii_digit()
                || (chars[index - 1].is_uppercase()
                    && chars.get(index + 1).is_some_and(|c| c.is_lowercase())))
        {
            result.push('_');
        }
        result.extend(character.to_lowercase());
    }
    result
}

fn channel(source: &str) -> String {
    match source {
        "m_worldPositionX" => "world_x",
        "m_worldPositionY" => "world_y",
        "m_worldPositionZ" => "world_z",
        "m_gForceLateral" => "g_lateral",
        "m_gForceLongitudinal" => "g_longitudinal",
        "m_gForceVertical" => "g_vertical",
        "m_currentLapNum" => "lap_number",
        "m_currentLapTimeInMS" => "lap_time",
        "m_lastLapTimeInMS" => "last_lap_time",
        "m_currentLapInvalid" => "lap_invalid",
        "m_tyresAgeLaps" => "tyre_age",
        "m_actualTyreCompound" => "tyre_compound",
        "m_visualTyreCompound" => "tyre_visual_compound",
        "m_brakesTemperature" => "brake_temperature",
        "m_tyresSurfaceTemperature" => "tyre_surface_temperature",
        "m_tyresInnerTemperature" => "tyre_inner_temperature",
        "m_tyresPressure" => "tyre_pressure",
        "m_tyresWear" => "tyre_wear",
        "m_tyresDamage" => "tyre_damage",
        "m_brakesDamage" => "brake_damage",
        "m_rearLeftTyrePressure" => "tyre_pressure_RL",
        "m_rearRightTyrePressure" => "tyre_pressure_RR",
        "m_frontLeftTyrePressure" => "tyre_pressure_FL",
        "m_frontRightTyrePressure" => "tyre_pressure_FR",
        _ => return snake(source),
    }
    .into()
}

fn values(value: &impl Serialize, prefix: &str) -> Result<TelemetryRow, String> {
    let Value::Object(source) = serde_json::to_value(value).map_err(|error| error.to_string())?
    else {
        return Err("Expected an F1 data object.".into());
    };
    let mut row = Map::new();
    for (key, value) in source {
        if key == "m_header" || value.is_null() {
            continue;
        }
        let name = format!("{prefix}{}", channel(&key));
        match value {
            Value::Array(items) => {
                for (index, item) in items.into_iter().enumerate() {
                    if item.is_number() || item.is_boolean() || item.is_string() {
                        let wheel = ["RL", "RR", "FL", "FR"]
                            .get(index)
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| index.to_string());
                        row.insert(format!("{name}_{wheel}"), item);
                    }
                }
            }
            Value::Object(_) => {}
            value => {
                row.insert(name, value);
            }
        }
    }
    if let Some(invalid) = row.get("lap_invalid").and_then(Value::as_u64) {
        row.insert("lap_invalid".into(), json!(invalid != 0));
    }
    if let (Some(ms), Some(minutes)) = (
        row.get("sector1_time_ms_part").and_then(Value::as_u64),
        row.get("sector1_time_minutes_part").and_then(Value::as_u64),
    ) {
        row.insert("sector_1_time".into(), json!(ms + minutes * 60000));
    }
    if let (Some(ms), Some(minutes)) = (
        row.get("sector2_time_ms_part").and_then(Value::as_u64),
        row.get("sector2_time_minutes_part").and_then(Value::as_u64),
    ) {
        row.insert("sector_2_time".into(), json!(ms + minutes * 60000));
    }
    Ok(row)
}

impl F1Decoder {
    pub fn decode(&mut self, bytes: &[u8]) -> Result<DecodedPacket, String> {
        if bytes.len() < 29 || bytes.len() > 65507 {
            return Err("Invalid F1 datagram length.".into());
        }
        let format = u16::from_le_bytes([bytes[0], bytes[1]]);
        if format != 2024 || bytes[2] != 24 || bytes[5] != 1 {
            return Err(format!(
                "Native F1 capture supports F1 2024 only. Received format {format}."
            ));
        }
        let packet_id = bytes[6];
        let expected = [
            1349, 753, 1285, 45, 1350, 1133, 1352, 1239, 1020, 1306, 953, 1460, 231, 237, 101,
        ];
        if packet_id as usize >= expected.len()
            || (packet_id != 3 && bytes.len() != expected[packet_id as usize])
            || (packet_id == 3 && !(33..=45).contains(&bytes.len()))
        {
            return Err("The F1 packet size does not match its type.".into());
        }
        let packet = F1TelemetryPacket::from_bytes(bytes)
            .map_err(|error| format!("Invalid F1 2024 packet: {error:?}"))?;
        let frame = u32::from_le_bytes(bytes[19..23].try_into().unwrap());
        let time = f32::from_le_bytes(bytes[15..19].try_into().unwrap()) as f64;
        let uid = u64::from_le_bytes(bytes[7..15].try_into().unwrap());
        let player = bytes[27] as usize;
        if !time.is_finite() || time < 0.0 || uid == 0 {
            return Err("Invalid F1 session identity or time.".into());
        }
        if self.uid != Some(uid) {
            *self = Self::default();
            self.uid = Some(uid);
        }
        let mut data = Vec::new();
        let private = matches!(packet_id, 5..=7 | 10);
        match packet {
            F1TelemetryPacket::Participants(packet) => {
                self.participants.clear();
                self.visibility.clear();
                for participant in packet
                    .m_participants
                    .iter()
                    .take(packet.m_numActiveCars.min(22) as usize)
                {
                    let mut row = Map::new();
                    row.insert("participant_name".into(), json!(participant.m_name));
                    row.insert(
                        "participant_race_number".into(),
                        json!(participant.m_raceNumber),
                    );
                    row.insert("participant_isAI".into(), json!(participant.m_aiControlled));
                    if let Some(team) = f1::constants::teams::Team::from_id(participant.m_teamId) {
                        row.insert("participant_team_name".into(), json!(team.name()));
                    }
                    self.participants.push(row);
                    self.visibility.push(participant.m_yourTelemetry != 0);
                }
            }
            F1TelemetryPacket::Session(packet) => {
                self.session.clear();
                if let Some(track) = f1::constants::tracks::Track::from_id(packet.m_trackId as u8) {
                    self.session
                        .insert("track_name".into(), json!(track.name()));
                }
                self.session
                    .insert("track_length".into(), json!(packet.m_trackLength));
                if let Some(kind) =
                    f1::constants::session_types::SessionType::from_id(packet.m_sessionType)
                {
                    self.session
                        .insert("session_type".into(), json!(kind.name()));
                }
                let mut row = Map::new();
                row.insert("air_temperature".into(), json!(packet.m_airTemperature));
                row.insert("track_temperature".into(), json!(packet.m_trackTemperature));
                row.insert("weather".into(), json!(packet.m_weather));
                row.insert("total_laps".into(), json!(packet.m_totalLaps));
                for index in self.indexes(player) {
                    data.push((index, row.clone()));
                }
            }
            F1TelemetryPacket::Lap(packet) => {
                self.laps = packet
                    .m_lapData
                    .iter()
                    .map(|lap| values(lap, ""))
                    .collect::<Result<_, _>>()?;
                for lap in &mut self.laps {
                    lap.insert("lap_source_time".into(), json!(time));
                }
                for index in self.indexes(player) {
                    data.push((index, self.laps[index].clone()));
                }
            }
            F1TelemetryPacket::CarMotion(packet) => {
                for index in self.indexes(player) {
                    let mut row = values(&packet.m_carMotionData[index], "")?;
                    row.insert("world_source_time".into(), json!(time));
                    data.push((index, row));
                }
            }
            F1TelemetryPacket::CarTelemetry(packet) => {
                for index in self.indexes(player) {
                    data.push((index, values(&packet.m_carTelemetryData[index], "")?));
                }
            }
            F1TelemetryPacket::CarStatus(packet) => {
                for index in self.indexes(player) {
                    data.push((index, values(&packet.m_carStatusData[index], "")?));
                }
            }
            F1TelemetryPacket::CarDamage(packet) => {
                for index in self.indexes(player) {
                    data.push((index, values(&packet.m_carDamageData[index], "")?));
                }
            }
            F1TelemetryPacket::CarSetup(packet) => {
                for index in self.indexes(player) {
                    data.push((index, values(&packet.m_carSetups[index], "setup_")?));
                }
            }
            F1TelemetryPacket::CarMotionEx(packet) if player < 22 => {
                data.push((player, values(&packet, "")?));
            }
            _ => {}
        }
        let mut rows = Vec::with_capacity(data.len());
        for (index, channels) in data {
            if private && index != player && !self.visibility.get(index).copied().unwrap_or(false) {
                continue;
            }
            let mut row = self.session.clone();
            if let Some(person) = self.participants.get(index) {
                row.extend(person.clone());
            }
            if let Some(lap) = self.laps.get(index).filter(|lap| {
                lap.get("lap_source_time")
                    .and_then(Value::as_f64)
                    .is_some_and(|source| source <= time)
            }) {
                for key in [
                    "lap_number",
                    "lap_distance",
                    "lap_time",
                    "lap_invalid",
                    "last_lap_time",
                    "lap_source_time",
                ] {
                    if let Some(value) = lap.get(key) {
                        row.insert(key.into(), value.clone());
                    }
                }
            }
            row.extend(channels);
            row.insert("participantId".into(), json!(format!("f1:{uid}:{index}")));
            row.insert("sessionId".into(), json!(uid.to_string()));
            row.insert("simulator".into(), json!("f1"));
            row.insert("carIndex".into(), json!(index));
            row.insert("playerCarIndex".into(), json!(player));
            row.insert("packetId".into(), json!(packet_id));
            row.insert("frameId".into(), json!(frame));
            row.insert("sessionTime".into(), json!(time));
            rows.push(row);
        }
        Ok(DecodedPacket {
            rows,
            session_key: Some(uid.to_string()),
            player_participant_id: (player < 22).then(|| format!("f1:{uid}:{player}")),
        })
    }

    fn indexes(&self, player: usize) -> Vec<usize> {
        if !self.participants.is_empty() {
            (0..self.participants.len()).collect()
        } else if player < 22 {
            vec![player]
        } else {
            Vec::new()
        }
    }
}
