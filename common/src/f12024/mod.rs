#![allow(non_snake_case)]

pub mod header;
pub mod event;
pub mod car_telemetry;
pub mod car_motion;
pub mod session;
pub mod lap;
pub mod participants;
pub mod car_setup;
pub mod car_status;
pub mod final_classification;
pub mod lobby_info;
pub mod car_damage;
pub mod session_history;
pub mod tyre_sets;
pub mod car_motion_ex;
pub mod time_trial;
pub mod constants;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum F1TelemetryPacket {
    CarMotion(car_motion::PacketCarMotionData),
    Session(session::PacketSessionData),
    Lap(lap::PacketLapData),
    Event(event::PacketEventData),
    Participants(participants::PacketParticipantsData),
    CarSetup(car_setup::PacketCarSetupData),
    CarTelemetry(car_telemetry::PacketCarTelemetryData),
    CarStatus(car_status::PacketCarStatusData),
    CarDamage(car_damage::PacketCarDamageData),
    FinalClassification(final_classification::PacketFinalClassificationData),
    LobbyInfo(lobby_info::PacketLobbyInfoData),
    SessionHistory(session_history::PacketSessionHistoryData),
    TyreSets(tyre_sets::PacketTyreSetsData),
    CarMotionEx(car_motion_ex::PacketMotionExData),
    TimeTrial(time_trial::PacketTimeTrialData),
    Unknown(header::PacketHeader),
}