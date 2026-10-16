use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_f32};
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