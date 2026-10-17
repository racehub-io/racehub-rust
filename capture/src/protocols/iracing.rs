use super::generated::iracing::{Car, TelemetryPacket};
use super::{DecodedPacket, TelemetryRow, control, number, text};
use prost::Message;
use std::collections::{HashMap, HashSet};

fn player_channels(car: &Car) -> bool {
    car.rpm.is_some()
        || car.speed.is_some()
        || car.gear.is_some()
        || car.brake.is_some()
        || car.throttle.is_some()
        || car.clutch.is_some()
        || car.lap_time.is_some()
}

pub(super) fn decode(bytes: &[u8]) -> Result<DecodedPacket, String> {
    let packet = TelemetryPacket::decode(bytes)
        .map_err(|_| "The iRacing datagram is not valid Protobuf.".to_string())?;
    let session = packet
        .session_info
        .ok_or("The iRacing datagram has no session information.")?;
    let track = session
        .track_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or("The iRacing datagram has no track name.")?;
    let time = session.session_time.unwrap_or_default();
    if !time.is_finite()
        || time < 0.0
        || packet.cars.is_empty()
        || packet.cars.len() > 128
        || packet.participants.is_empty()
        || packet.participants.len() > 128
    {
        return Err("The iRacing session time or driver count is invalid.".into());
    }
    let mut participants = HashMap::new();
    for participant in &packet.participants {
        let index = participant.participant_idx.unwrap_or_default();
        if !(0..128).contains(&index) || participants.insert(index, participant).is_some() {
            return Err("The iRacing participant slots are invalid or repeated.".into());
        }
    }
    let mut indexes = HashSet::new();
    for car in &packet.cars {
        let index = car.car_idx.unwrap_or_default();
        if !(0..128).contains(&index)
            || !participants.contains_key(&index)
            || !indexes.insert(index)
        {
            return Err("The iRacing car slots do not match the participants.".into());
        }
    }
    let candidates: Vec<_> = packet
        .cars
        .iter()
        .filter(|car| player_channels(car))
        .collect();
    if candidates.len() > 1 {
        return Err("The iRacing packet assigns player-only channels to multiple cars.".into());
    }
    let player = candidates
        .first()
        .map(|car| car.car_idx.unwrap_or_default());
    let session_id = session.session_id.unwrap_or_default().to_string();
    let key = format!("{session_id}:{track}");
    let length = session
        .track_length
        .filter(|length| *length > 0)
        .map(f64::from);
    let mut rows = Vec::with_capacity(packet.cars.len());
    for car in &packet.cars {
        let index = car.car_idx.unwrap_or_default();
        let participant = participants[&index];
        let mut row = TelemetryRow::new();
        row.insert(
            "participantId".into(),
            format!("iracing:{session_id}:{index}").into(),
        );
        row.insert("sessionId".into(), session_id.clone().into());
        row.insert("simulator".into(), "iracing".into());
        row.insert("carIndex".into(), index.into());
        if let Some(player) = player {
            row.insert("playerCarIndex".into(), player.into());
        }
        number(&mut row, "sessionTime", Some(time));
        text(&mut row, "track_name", Some(track));
        number(&mut row, "track_length", length);
        text(&mut row, "session_type", session.event_type.as_deref());
        text(
            &mut row,
            "participant_name",
            participant.driver_name.as_deref(),
        );
        text(
            &mut row,
            "participant_team_name",
            participant.team_name.as_deref(),
        );
        text(
            &mut row,
            "participant_abbreviation",
            participant.abbreviation.as_deref(),
        );
        number(
            &mut row,
            "participant_race_number",
            participant.car_number.map(f64::from),
        );
        number(
            &mut row,
            "lap_number",
            Some(f64::from(car.lap.unwrap_or_default())),
        );
        number(
            &mut row,
            "lap_distance",
            length.map(|length| f64::from(car.lap_distance_pct.unwrap_or_default()) * length),
        );
        number(
            &mut row,
            "steer_angle",
            Some(f64::from(car.steer.unwrap_or_default())),
        );
        number(
            &mut row,
            "air_temperature",
            session.air_temperature.map(f64::from),
        );
        number(
            &mut row,
            "track_temperature",
            session.track_temperature.map(f64::from),
        );
        if player == Some(index) {
            number(
                &mut row,
                "lap_time",
                Some(f64::from(car.lap_time.unwrap_or_default()) * 1000.0),
            );
            number(
                &mut row,
                "speed",
                Some(f64::from(car.speed.unwrap_or_default()) * 3.6),
            );
            number(
                &mut row,
                "throttle",
                control(f64::from(car.throttle.unwrap_or_default()) / 100.0, 0.0),
            );
            number(
                &mut row,
                "brake",
                control(f64::from(car.brake.unwrap_or_default()) / 100.0, 0.0),
            );
            number(
                &mut row,
                "clutch",
                control(f64::from(car.clutch.unwrap_or_default()) / 100.0, 0.0),
            );
            row.insert("gear".into(), car.gear.unwrap_or_default().into());
            number(
                &mut row,
                "engine_rpm",
                Some(f64::from(car.rpm.unwrap_or_default())),
            );
        }
        rows.push(row);
    }
    Ok(DecodedPacket {
        rows,
        player_participant_id: player.map(|index| format!("iracing:{session_id}:{index}")),
        session_key: Some(key),
    })
}
