use super::generated::rfactor2::{DataPacket, ScoringInfo, TelemInfo};
use super::{DecodedPacket, TelemetryRow, control, number, text};
use prost::Message;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct Adapter {
    scoring: Option<ScoringInfo>,
    generation: u64,
    last_track: Option<String>,
    last_time: Option<f64>,
    last_session: Option<u32>,
    lap_numbers: HashMap<u32, (u32, f64)>,
}

impl Adapter {
    pub(super) fn decode(&mut self, bytes: &[u8]) -> Result<DecodedPacket, String> {
        let packet = DataPacket::decode(bytes)
            .map_err(|_| "The rFactor 2 datagram is not valid Protobuf.".to_string())?;
        let header = packet
            .packet_header
            .ok_or("The rFactor 2 datagram has no header.")?;
        let kind = header.packet_type.unwrap_or_default();
        let bodies = [
            packet.telemetry.is_some(),
            packet.scoring.is_some(),
            packet.graphics.is_some(),
            packet.physics.is_some(),
        ];
        if header.contract_version != Some(1)
            || !(0..4).contains(&kind)
            || bodies.iter().filter(|value| **value).count() != 1
            || !bodies[kind as usize]
        {
            return Err("The rFactor 2 body does not match contract 1 and its packet type.".into());
        }
        if let Some(scoring) = packet.scoring {
            return self.update_scoring(scoring, header.epoch_time);
        }
        match packet.telemetry {
            Some(car) => self.telemetry(car, header.epoch_time),
            None => Ok(DecodedPacket::default()),
        }
    }

    fn update_session(&mut self, track: &str, time: f64, session: Option<u32>) {
        let changed_track = self
            .last_track
            .as_deref()
            .is_some_and(|previous| previous != track);
        let reversed_clock = self.last_time.is_some_and(|previous| time < previous - 1.0);
        let changed_session = session
            .zip(self.last_session)
            .is_some_and(|(current, previous)| current != previous);
        if changed_track || reversed_clock || changed_session {
            self.generation += 1;
            self.scoring = None;
            self.last_time = None;
            self.last_session = None;
            self.lap_numbers.clear();
        }
        self.last_track = Some(track.to_string());
        self.last_time = Some(self.last_time.map_or(time, |previous| previous.max(time)));
        if let Some(session) = session {
            self.last_session = Some(session);
        }
    }

    fn update_scoring(
        &mut self,
        scoring: ScoringInfo,
        epoch: Option<u32>,
    ) -> Result<DecodedPacket, String> {
        let time = scoring.m_current_et.unwrap_or_default();
        if !time.is_finite()
            || time < 0.0
            || scoring.m_vehicle.len() > 256
            || scoring
                .m_track_name
                .as_deref()
                .is_none_or(|name| name.trim().is_empty())
        {
            return Err("The rFactor 2 scoring time, track, or driver count is invalid.".into());
        }
        let mut ids = HashSet::new();
        let mut players = 0;
        for vehicle in &scoring.m_vehicle {
            if !ids.insert(vehicle.m_id.unwrap_or_default()) {
                return Err("The rFactor 2 scoring data repeats a vehicle ID.".into());
            }
            if vehicle.m_is_player == Some(true) {
                players += 1;
            }
        }
        if players > 1 {
            return Err("The rFactor 2 scoring data identifies multiple players.".into());
        }
        let session = scoring.m_session.unwrap_or_default();
        let track = scoring.m_track_name.as_deref().unwrap().trim();
        self.update_session(track, time, Some(session));
        self.lap_numbers.retain(|index, _| ids.contains(index));
        let key = format!("{track}:{}", self.generation);
        let player = scoring
            .m_vehicle
            .iter()
            .find(|vehicle| vehicle.m_is_player == Some(true))
            .map(|vehicle| vehicle.m_id.unwrap_or_default());
        let mut rows = Vec::with_capacity(scoring.m_vehicle.len());
        for vehicle in &scoring.m_vehicle {
            let index = vehicle.m_id.unwrap_or_default();
            let mut row = TelemetryRow::new();
            row.insert(
                "participantId".into(),
                format!("rfactor2:{key}:{index}").into(),
            );
            row.insert("sessionId".into(), key.clone().into());
            row.insert("simulator".into(), "rfactor2".into());
            row.insert("carIndex".into(), index.into());
            row.insert("packetId".into(), 1.into());
            if let Some(player) = player {
                row.insert("playerCarIndex".into(), player.into());
            }
            if let Some(epoch) = epoch {
                row.insert("source_epoch_seconds".into(), epoch.into());
            }
            number(&mut row, "sessionTime", Some(time));
            number(&mut row, "lap_source_time", Some(time));
            if let Some((lap, source_time)) = self
                .lap_numbers
                .get(&index)
                .filter(|(_, source_time)| *source_time <= time + 0.001)
            {
                number(&mut row, "lap_number", Some(f64::from(*lap)));
                number(&mut row, "lap_number_source_time", Some(*source_time));
            }
            text(&mut row, "track_name", Some(track));
            text(&mut row, "session_type", Some(&session.to_string()));
            text(
                &mut row,
                "participant_name",
                vehicle.m_driver_name.as_deref(),
            );
            text(&mut row, "vehicle_name", vehicle.m_vehicle_name.as_deref());
            text(
                &mut row,
                "vehicle_class",
                vehicle.m_vehicle_class.as_deref(),
            );
            number(
                &mut row,
                "track_length",
                scoring.m_lap_dist.filter(|length| *length > 0.0),
            );
            number(
                &mut row,
                "laps_completed",
                Some(f64::from(vehicle.m_total_laps.unwrap_or_default())),
            );
            number(
                &mut row,
                "lap_distance",
                Some(vehicle.m_lap_dist.unwrap_or_default()),
            );
            number(&mut row, "car_position", vehicle.m_place.map(f64::from));
            number(&mut row, "air_temperature", scoring.m_ambient_temp);
            number(&mut row, "track_temperature", scoring.m_track_temp);
            number(
                &mut row,
                "last_lap_time",
                vehicle
                    .m_last_lap_time
                    .filter(|time| *time > 0.0)
                    .map(|time| time * 1000.0),
            );
            let lap_start = vehicle.m_lap_start_et.unwrap_or_default();
            if lap_start.is_finite() && lap_start >= 0.0 && lap_start <= time {
                number(&mut row, "lap_time", Some((time - lap_start) * 1000.0));
            }
            row.insert(
                "lap_invalid".into(),
                (vehicle.m_count_lap_flag.unwrap_or_default() != 2).into(),
            );
            if let Some(position) = &vehicle.m_pos {
                number(&mut row, "world_x", Some(position.x.unwrap_or_default()));
                number(&mut row, "world_y", Some(position.y.unwrap_or_default()));
                number(&mut row, "world_z", Some(position.z.unwrap_or_default()));
                number(&mut row, "world_source_time", Some(time));
            }
            rows.push(row);
        }
        self.scoring = Some(scoring);
        Ok(DecodedPacket {
            rows,
            player_participant_id: player.map(|player| format!("rfactor2:{key}:{player}")),
            session_key: Some(key),
        })
    }

    fn telemetry(&mut self, car: TelemInfo, epoch: Option<u32>) -> Result<DecodedPacket, String> {
        let time = car.m_elapsed_time.unwrap_or_default();
        let track = car
            .m_track_name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .ok_or("The rFactor 2 telemetry has no track name.")?;
        let velocity = car
            .m_local_vel
            .ok_or("The rFactor 2 telemetry has no velocity vector.")?;
        let speed = velocity
            .x
            .unwrap_or_default()
            .hypot(velocity.y.unwrap_or_default())
            .hypot(velocity.z.unwrap_or_default())
            * 3.6;
        if !time.is_finite() || time < 0.0 || !speed.is_finite() || car.m_wheel.len() > 4 {
            return Err(
                "The rFactor 2 telemetry time, velocity, or wheel count is invalid.".into(),
            );
        }
        self.update_session(track, time, None);
        let key = format!("{track}:{}", self.generation);
        let index = car.m_id.unwrap_or_default();
        if self.lap_numbers.len() >= 256 && !self.lap_numbers.contains_key(&index) {
            return Err("The rFactor 2 telemetry exceeds 256 vehicle slots.".into());
        }
        self.lap_numbers
            .insert(index, (car.m_lap_number.unwrap_or_default(), time));
        let scoring = self.scoring.as_ref().filter(|scoring| {
            scoring
                .m_track_name
                .as_deref()
                .is_some_and(|name| name.trim() == track)
                && scoring.m_current_et.unwrap_or_default() <= time + 0.001
        });
        let score = scoring.and_then(|scoring| {
            scoring
                .m_vehicle
                .iter()
                .find(|vehicle| vehicle.m_id.unwrap_or_default() == index)
        });
        let player = scoring
            .and_then(|scoring| {
                scoring
                    .m_vehicle
                    .iter()
                    .find(|vehicle| vehicle.m_is_player == Some(true))
            })
            .map(|vehicle| vehicle.m_id.unwrap_or_default());
        let mut row = TelemetryRow::new();
        row.insert(
            "participantId".into(),
            format!("rfactor2:{key}:{index}").into(),
        );
        row.insert("sessionId".into(), key.clone().into());
        row.insert("simulator".into(), "rfactor2".into());
        row.insert("carIndex".into(), index.into());
        row.insert("packetId".into(), 0.into());
        if let Some(player) = player {
            row.insert("playerCarIndex".into(), player.into());
        }
        if let Some(epoch) = epoch {
            row.insert("source_epoch_seconds".into(), epoch.into());
        }
        number(&mut row, "sessionTime", Some(time));
        text(&mut row, "track_name", Some(track));
        text(&mut row, "vehicle_name", car.m_vehicle_name.as_deref());
        number(&mut row, "speed", Some(speed));
        number(
            &mut row,
            "lap_number",
            Some(f64::from(car.m_lap_number.unwrap_or_default())),
        );
        let lap_start = car.m_lap_start_et.unwrap_or_default();
        if lap_start.is_finite() && lap_start >= 0.0 && lap_start <= time {
            number(&mut row, "lap_time", Some((time - lap_start) * 1000.0));
        }
        if let Some(scoring) = scoring {
            number(
                &mut row,
                "track_length",
                scoring.m_lap_dist.filter(|length| *length > 0.0),
            );
            text(
                &mut row,
                "session_type",
                Some(&scoring.m_session.unwrap_or_default().to_string()),
            );
        }
        if let Some(score) = score {
            text(&mut row, "participant_name", score.m_driver_name.as_deref());
            number(
                &mut row,
                "lap_distance",
                Some(score.m_lap_dist.unwrap_or_default()),
            );
            number(
                &mut row,
                "last_lap_time",
                score
                    .m_last_lap_time
                    .filter(|time| *time > 0.0)
                    .map(|time| time * 1000.0),
            );
            row.insert(
                "lap_invalid".into(),
                (score.m_count_lap_flag.unwrap_or_default() != 2).into(),
            );
            number(
                &mut row,
                "lap_source_time",
                scoring.map(|scoring| scoring.m_current_et.unwrap_or_default()),
            );
        }
        number(
            &mut row,
            "throttle",
            control(car.m_unfiltered_throttle.unwrap_or_default(), 0.0),
        );
        number(
            &mut row,
            "brake",
            control(car.m_unfiltered_brake.unwrap_or_default(), 0.0),
        );
        number(
            &mut row,
            "clutch",
            control(car.m_unfiltered_clutch.unwrap_or_default(), 0.0),
        );
        number(
            &mut row,
            "steer",
            control(car.m_unfiltered_steering.unwrap_or_default(), -1.0),
        );
        row.insert(
            "gear".into(),
            (car.m_gear.unwrap_or_default() as i32).into(),
        );
        number(&mut row, "engine_rpm", car.m_engine_rpm);
        number(&mut row, "fuel_in_tank", car.m_fuel);
        number(&mut row, "fuel_capacity", car.m_fuel_capacity);
        number(&mut row, "engine_temperature", car.m_engine_water_temp);
        if let Some(position) = car.m_pos {
            number(&mut row, "world_x", Some(position.x.unwrap_or_default()));
            number(&mut row, "world_y", Some(position.y.unwrap_or_default()));
            number(&mut row, "world_z", Some(position.z.unwrap_or_default()));
            number(&mut row, "world_source_time", Some(time));
        }
        for (wheel, suffix) in car.m_wheel.iter().zip(["FL", "FR", "RL", "RR"]) {
            number(
                &mut row,
                &format!("brake_temperature_{suffix}"),
                wheel.m_brake_temp,
            );
            number(
                &mut row,
                &format!("tyre_pressure_{suffix}"),
                wheel.m_pressure.map(|value| value * 0.145_037_737_7),
            );
            number(
                &mut row,
                &format!("tyre_inner_temperature_{suffix}"),
                wheel
                    .m_tire_carcass_temperature
                    .filter(|value| *value > 0.0)
                    .map(|value| value - 273.15),
            );
            number(
                &mut row,
                &format!("tyre_surface_temperature_{suffix}"),
                wheel
                    .m_temperature
                    .get(1)
                    .copied()
                    .filter(|value| *value > 0.0)
                    .map(|value| value - 273.15),
            );
        }
        Ok(DecodedPacket {
            rows: vec![row],
            player_participant_id: player.map(|player| format!("rfactor2:{key}:{player}")),
            session_key: Some(key),
        })
    }
}
