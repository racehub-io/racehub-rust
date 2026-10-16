use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_f32, le_i8, le_u16, le_u8};
use super::header::PacketHeader;

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct CarTelemetryData {
    #[nom(Parse = "le_u16")]
    pub m_speed: u16,                      // Speed of car in kilometres per hour
    #[nom(Parse = "le_f32")]
    pub m_throttle: f32,                   // Amount of throttle applied (0.0 to 1.0)
    #[nom(Parse = "le_f32")]
    pub m_steer: f32,                      // Steering (-1.0 (full lock left) to 1.0 (full lock right))
    #[nom(Parse = "le_f32")]
    pub m_brake: f32,                      // Amount of brake applied (0.0 to 1.0)
    #[nom(Parse = "le_u8")]
    pub m_clutch: u8,                      // Amount of clutch applied (0 to 100)
    #[nom(Parse = "le_i8")]
    pub m_gear: i8,                        // Gear selected (1-8, N=0, R=-1)
    #[nom(Parse = "le_u16")]
    pub m_engineRPM: u16,                  // Engine RPM
    #[nom(Parse = "le_u8")]
    pub m_drs: u8,                         // 0 = off, 1 = on
    #[nom(Parse = "le_u8")]
    pub m_revLightsPercent: u8,            // Rev lights indicator (percentage)
    #[nom(Parse = "le_u16")]
    pub m_revLightsBitValue: u16,          // Rev lights (bit 0 = leftmost LED, bit 14 = rightmost LED)
    #[nom(Count = "4", Parse = "le_u16")]
    pub m_brakesTemperature: Vec<u16>,     // Brakes temperature (celsius)
    #[nom(Count = "4", Parse = "le_u8")]
    pub m_tyresSurfaceTemperature: Vec<u8>,   // Tyres surface temperature (celsius)
    #[nom(Count = "4", Parse = "le_u8")]
    pub m_tyresInnerTemperature: Vec<u8>,    // Tyres inner temperature (celsius)
    #[nom(Parse = "le_u16")]
    pub m_engineTemperature: u16,           // Engine temperature (celsius)
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_tyresPressure: Vec<f32>,          // Tyres pressure (PSI)
    #[nom(Count = "4", Parse = "le_u8")]
    pub m_surfaceType: Vec<u8>,             // Driving surface, see appendices
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct PacketCarTelemetryData {
    pub m_header: PacketHeader,
    #[nom(Count = "22")]
    pub m_carTelemetryData: Vec<CarTelemetryData>,
    #[nom(Parse = "le_u8")]
    pub m_mfdPanelIndex: u8,               // Index of MFD panel open - 255 = MFD closed
    #[nom(Parse = "le_u8")]
    pub m_mfdPanelIndexSecondaryPlayer: u8, // See above
    #[nom(Parse = "le_i8")]
    pub m_suggestedGear: i8,               // Suggested gear for the player (1-8), 0 if no gear suggested
}

pub fn parse_car_telemetry_packet_internal(input: &[u8]) -> IResult<&[u8], PacketCarTelemetryData> {
    PacketCarTelemetryData::parse(input)
} 