use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_f32, le_u16, le_i8};
use super::header::PacketHeader;

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct CarSetupData {
    #[nom(Parse = "le_u8")]
    pub m_frontWing: u8,                // Front wing aero
    #[nom(Parse = "le_u8")]
    pub m_rearWing: u8,                 // Rear wing aero
    #[nom(Parse = "le_u8")]
    pub m_onThrottle: u8,               // Differential adjustment on throttle (percentage)
    #[nom(Parse = "le_u8")]
    pub m_offThrottle: u8,              // Differential adjustment off throttle (percentage)
    #[nom(Parse = "le_f32")]
    pub m_frontCamber: f32,             // Front camber angle (suspension geometry)
    #[nom(Parse = "le_f32")]
    pub m_rearCamber: f32,              // Rear camber angle (suspension geometry)
    #[nom(Parse = "le_f32")]
    pub m_frontToe: f32,                // Front toe angle (suspension geometry)
    #[nom(Parse = "le_f32")]
    pub m_rearToe: f32,                 // Rear toe angle (suspension geometry)
    #[nom(Parse = "le_u8")]
    pub m_frontSuspension: u8,          // Front suspension
    #[nom(Parse = "le_u8")]
    pub m_rearSuspension: u8,           // Rear suspension
    #[nom(Parse = "le_u8")]
    pub m_frontAntiRollBar: u8,         // Front anti-roll bar
    #[nom(Parse = "le_u8")]
    pub m_rearAntiRollBar: u8,          // Front anti-roll bar
    #[nom(Parse = "le_u8")]
    pub m_frontSuspensionHeight: u8,    // Front ride height
    #[nom(Parse = "le_u8")]
    pub m_rearSuspensionHeight: u8,     // Rear ride height
    #[nom(Parse = "le_u8")]
    pub m_brakePressure: u8,            // Brake pressure (percentage)
    #[nom(Parse = "le_u8")]
    pub m_brakeBias: u8,                // Brake bias (percentage)
    #[nom(Parse = "le_u8")]
    pub m_engineBraking: u8,            // Engine braking (percentage)
    #[nom(Parse = "le_f32")]
    pub m_rearLeftTyrePressure: f32,    // Rear left tyre pressure (PSI)
    #[nom(Parse = "le_f32")]
    pub m_rearRightTyrePressure: f32,   // Rear right tyre pressure (PSI)
    #[nom(Parse = "le_f32")]
    pub m_frontLeftTyrePressure: f32,   // Front left tyre pressure (PSI)
    #[nom(Parse = "le_f32")]
    pub m_frontRightTyrePressure: f32,  // Front right tyre pressure (PSI)
    #[nom(Parse = "le_u8")]
    pub m_ballast: u8,                  // Ballast
    #[nom(Parse = "le_f32")]
    pub m_fuelLoad: f32,                // Fuel load
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketCarSetupData {
    pub m_header: PacketHeader,
    #[nom(Count = "22")]
    pub m_carSetups: Vec<CarSetupData>,
    #[nom(Parse = "le_f32")]
    pub m_nextFrontWingValue: f32,      // Value of front wing after next pit stop - player only
}

pub fn parse_car_setup_packet_internal(input: &[u8]) -> IResult<&[u8], PacketCarSetupData> {
    PacketCarSetupData::parse(input)
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct CarStatusData {
    #[nom(Parse = "le_u8")]
    pub m_tractionControl: u8,          // Traction control - 0 = off, 1 = medium, 2 = full
    #[nom(Parse = "le_u8")]
    pub m_antiLockBrakes: u8,           // 0 (off) - 1 (on)
    #[nom(Parse = "le_u8")]
    pub m_fuelMix: u8,                  // Fuel mix - 0 = lean, 1 = standard, 2 = rich, 3 = max
    #[nom(Parse = "le_u8")]
    pub m_frontBrakeBias: u8,           // Front brake bias (percentage)
    #[nom(Parse = "le_u8")]
    pub m_pitLimiterStatus: u8,         // Pit limiter status - 0 = off, 1 = on
    #[nom(Parse = "le_f32")]
    pub m_fuelInTank: f32,              // Current fuel mass
    #[nom(Parse = "le_f32")]
    pub m_fuelCapacity: f32,            // Fuel capacity
    #[nom(Parse = "le_f32")]
    pub m_fuelRemainingLaps: f32,       // Fuel remaining in terms of laps (value on MFD)
    #[nom(Parse = "le_u16")]
    pub m_maxRPM: u16,                  // Cars max RPM, point of rev limiter
    #[nom(Parse = "le_u16")]
    pub m_idleRPM: u16,                 // Cars idle RPM
    #[nom(Parse = "le_u8")]
    pub m_maxGears: u8,                 // Maximum number of gears
    #[nom(Parse = "le_u8")]
    pub m_drsAllowed: u8,               // 0 = not allowed, 1 = allowed
    #[nom(Parse = "le_u16")]
    pub m_drsActivationDistance: u16,   // 0 = DRS not available, non-zero - DRS will be available in [X] metres
    #[nom(Parse = "le_u8")]
    pub m_actualTyreCompound: u8,       // F1 Modern - 16 = C5, 17 = C4, 18 = C3, 19 = C2, 20 = C1
                                        // 21 = C0, 7 = inter, 8 = wet
                                        // F1 Classic - 9 = dry, 10 = wet
                                        // F2 – 11 = super soft, 12 = soft, 13 = medium, 14 = hard
                                        // 15 = wet
    #[nom(Parse = "le_u8")]
    pub m_visualTyreCompound: u8,       // F1 visual (can be different from actual compound)
                                        // 16 = soft, 17 = medium, 18 = hard, 7 = inter, 8 = wet
                                        // F1 Classic – same as above
                                        // F2 '19, 15 = wet, 19 – super soft, 20 = soft
                                        // 21 = medium , 22 = hard
    #[nom(Parse = "le_u8")]
    pub m_tyresAgeLaps: u8,             // Age in laps of the current set of tyres
    #[nom(Parse = "le_i8")]
    pub m_vehicleFiaFlags: i8,          // -1 = invalid/unknown, 0 = none, 1 = green
                                        // 2 = blue, 3 = yellow
    #[nom(Parse = "le_f32")]
    pub m_enginePowerICE: f32,          // Engine power output of ICE (W)
    #[nom(Parse = "le_f32")]
    pub m_enginePowerMGUK: f32,         // Engine power output of MGU-K (W)
    #[nom(Parse = "le_f32")]
    pub m_ersStoreEnergy: f32,          // ERS energy store in Joules
    #[nom(Parse = "le_u8")]
    pub m_ersDeployMode: u8,            // ERS deployment mode, 0 = none, 1 = medium
                                        // 2 = hotlap, 3 = overtake
    #[nom(Parse = "le_f32")]
    pub m_ersHarvestedThisLapMGUK: f32, // ERS energy harvested this lap by MGU-K
    #[nom(Parse = "le_f32")]
    pub m_ersHarvestedThisLapMGUH: f32, // ERS energy harvested this lap by MGU-H
    #[nom(Parse = "le_f32")]
    pub m_ersDeployedThisLap: f32,      // ERS energy deployed this lap
    #[nom(Parse = "le_u8")]
    pub m_networkPaused: u8,            // Whether the car is paused in a network game
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketCarStatusData {
    pub m_header: PacketHeader,
    #[nom(Count = "22")]
    pub m_carStatusData: Vec<CarStatusData>,
}

pub fn parse_car_status_packet_internal(input: &[u8]) -> IResult<&[u8], PacketCarStatusData> {
    PacketCarStatusData::parse(input)
} 