use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_f32, le_i16};
use super::header::PacketHeader;

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct CarMotionData {
    #[nom(Parse = "le_f32")]
    pub m_worldPositionX: f32,
    #[nom(Parse = "le_f32")]
    pub m_worldPositionY: f32,
    #[nom(Parse = "le_f32")]
    pub m_worldPositionZ: f32,
    #[nom(Parse = "le_f32")]
    pub m_worldVelocityX: f32,
    #[nom(Parse = "le_f32")]
    pub m_worldVelocityY: f32,
    #[nom(Parse = "le_f32")]
    pub m_worldVelocityZ: f32,
    #[nom(Parse = "le_i16")]
    pub m_worldForwardDirX: i16,
    #[nom(Parse = "le_i16")]
    pub m_worldForwardDirY: i16,
    #[nom(Parse = "le_i16")]
    pub m_worldForwardDirZ: i16,
    #[nom(Parse = "le_i16")]
    pub m_worldRightDirX: i16,
    #[nom(Parse = "le_i16")]
    pub m_worldRightDirY: i16,
    #[nom(Parse = "le_i16")]
    pub m_worldRightDirZ: i16,
    #[nom(Parse = "le_f32")]
    pub m_gForceLateral: f32,
    #[nom(Parse = "le_f32")]
    pub m_gForceLongitudinal: f32,
    #[nom(Parse = "le_f32")]
    pub m_gForceVertical: f32,
    #[nom(Parse = "le_f32")]
    pub m_yaw: f32,
    #[nom(Parse = "le_f32")]
    pub m_pitch: f32,
    #[nom(Parse = "le_f32")]
    pub m_roll: f32,
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketCarMotionData {
    pub m_header: PacketHeader,
    #[nom(Count = "22")]
    pub m_carMotionData: Vec<CarMotionData>,
}

pub fn parse_car_motion_packet_internal(input: &[u8]) -> IResult<&[u8], PacketCarMotionData> {
    PacketCarMotionData::parse(input)
}