use crate::telemetry::{CarTelemetry, TelemetryProvider};
use common::f12024::F1TelemetryPacket;
use super::TelemetryError;

pub struct F12024Provider;

impl TelemetryProvider for F12024Provider {
    type Packet = F1TelemetryPacket;
    type Error = TelemetryError;

    fn get_car_telemetry(&self, packet: &Self::Packet) -> Result<Vec<CarTelemetry>, Self::Error> {
        match packet {
            F1TelemetryPacket::CarTelemetry(telemetry_packet) => {
                let mut cars = vec![CarTelemetry::default(); telemetry_packet.m_carTelemetryData.len()];
                
                // Update telemetry data for each car
                for (i, car_data) in telemetry_packet.m_carTelemetryData.iter().enumerate() {
                    let car = &mut cars[i];
                    car.speed = Some(car_data.m_speed as f32);
                    car.rpm = Some(car_data.m_engineRPM as f32);
                    car.throttle = Some(car_data.m_throttle);
                    car.steer = Some(car_data.m_steer);
                    car.brake = Some(car_data.m_brake);
                    car.clutch = Some(car_data.m_clutch as f32);
                    car.gear = Some(car_data.m_gear as i32);
                    car.drs = Some(car_data.m_drs);
                    car.brake_temperature_front_left = Some(car_data.m_brakesTemperature[2]);
                    car.brake_temperature_front_right = Some(car_data.m_brakesTemperature[3]);
                    car.brake_temperature_rear_left = Some(car_data.m_brakesTemperature[0]);
                    car.brake_temperature_rear_right = Some(car_data.m_brakesTemperature[1]);
                    car.tyre_surface_temperature_front_left = Some(car_data.m_tyresSurfaceTemperature[2]);
                    car.tyre_surface_temperature_front_right = Some(car_data.m_tyresSurfaceTemperature[3]);
                    car.tyre_surface_temperature_rear_left = Some(car_data.m_tyresSurfaceTemperature[0]);
                    car.tyre_surface_temperature_rear_right = Some(car_data.m_tyresSurfaceTemperature[1]);
                    car.tyre_carcass_temperature_front_left = Some(car_data.m_tyresInnerTemperature[2]);
                    car.tyre_carcass_temperature_front_right = Some(car_data.m_tyresInnerTemperature[3]);
                    car.tyre_carcass_temperature_rear_left = Some(car_data.m_tyresInnerTemperature[0]);
                    car.tyre_carcass_temperature_rear_right = Some(car_data.m_tyresInnerTemperature[1]);
                    car.tyre_pressure_front_left = Some(car_data.m_tyresPressure[2]);
                    car.tyre_pressure_front_right = Some(car_data.m_tyresPressure[3]);
                    car.tyre_pressure_rear_left = Some(car_data.m_tyresPressure[0]);
                    car.tyre_pressure_rear_right = Some(car_data.m_tyresPressure[1]);
                    car.engine_temperature = Some(car_data.m_engineTemperature);
                }
                
                Ok(cars)
            },
            F1TelemetryPacket::CarMotion(motion_packet) => {
                let mut cars = vec![CarTelemetry::default(); motion_packet.m_carMotionData.len()];
                
                // Update motion data for each car
                for (i, car_data) in motion_packet.m_carMotionData.iter().enumerate() {
                    let car = &mut cars[i];
                    car.world_position_x = Some(car_data.m_worldPositionX);
                    car.world_position_y = Some(car_data.m_worldPositionY);
                    car.world_position_z = Some(car_data.m_worldPositionZ);
                    car.g_force_lateral = Some(car_data.m_gForceLateral);
                    car.g_force_longitudinal = Some(car_data.m_gForceLongitudinal);
                    car.g_force_vertical = Some(car_data.m_gForceVertical);
                    car.yaw = Some(car_data.m_yaw);
                    car.pitch = Some(car_data.m_pitch);
                    car.roll = Some(car_data.m_roll);
                }
                
                Ok(cars)
            },
            F1TelemetryPacket::Lap(lap_packet) => {
                let mut cars = vec![CarTelemetry::default(); lap_packet.m_lapData.len()];
                
                for (i, car_data) in lap_packet.m_lapData.iter().enumerate() {
                    let car = &mut cars[i];
                    car.lap_time = Some(car_data.m_currentLapTimeInMS as f32);
                    car.lap_distance = Some(car_data.m_lapDistance);
                }
                
                Ok(cars)
            },
            // Return empty result for other packet types
            _ => Ok(vec![])
        }
    }

    fn get_car_count(&self, packet: &Self::Packet) -> usize {
        match packet {
            F1TelemetryPacket::CarTelemetry(p) => p.m_carTelemetryData.len(),
            F1TelemetryPacket::CarMotion(p) => p.m_carMotionData.len(),
            F1TelemetryPacket::Lap(p) => p.m_lapData.len(),
            F1TelemetryPacket::Participants(p) => p.m_participants.len(),
            F1TelemetryPacket::CarStatus(p) => p.m_carStatusData.len(),
            F1TelemetryPacket::CarSetup(p) => p.m_carSetups.len(),
            F1TelemetryPacket::CarDamage(p) => p.m_carDamageData.len(),
            _ => 0,
        }
    }
}

// Helper methods for extracting metadata from F1 packets
pub struct F12024Metadata;

impl F12024Metadata {
    pub fn get_packet_metadata(packet: &F1TelemetryPacket) -> Option<(u32, f32, u64)> {
        // All packet types have the same header structure, so we can access it directly
        let header = match packet {
            F1TelemetryPacket::CarTelemetry(p) => &p.m_header,
            F1TelemetryPacket::CarStatus(p) => &p.m_header,
            F1TelemetryPacket::CarMotion(p) => &p.m_header,
            F1TelemetryPacket::Lap(p) => &p.m_header,
            F1TelemetryPacket::Event(p) => &p.m_header,
            F1TelemetryPacket::Participants(p) => &p.m_header,
            F1TelemetryPacket::CarSetup(p) => &p.m_header,
            F1TelemetryPacket::Session(p) => &p.m_header,
            F1TelemetryPacket::FinalClassification(p) => &p.m_header,
            F1TelemetryPacket::LobbyInfo(p) => &p.m_header,
            F1TelemetryPacket::CarDamage(p) => &p.m_header,
            F1TelemetryPacket::SessionHistory(p) => &p.m_header,
            F1TelemetryPacket::TyreSets(p) => &p.m_header,
            F1TelemetryPacket::CarMotionEx(p) => &p.m_header,
            F1TelemetryPacket::TimeTrial(p) => &p.m_header,
            F1TelemetryPacket::Unknown(h) => h,
        };
        
        // Extract all metadata at once from the header
        Some((
            header.m_frameIdentifier,
            header.m_sessionTime,
            header.m_sessionUID
        ))
    }
    
    pub fn get_session_id(packet: &F1TelemetryPacket) -> Option<u64> {
        Self::get_packet_metadata(packet).map(|(_, _, session_id)| session_id)
    }
} 