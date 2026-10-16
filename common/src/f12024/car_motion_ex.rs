use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::le_f32;
use super::header::PacketHeader;

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketMotionExData {
    pub m_header: PacketHeader,
    
    // Extra player car ONLY data
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_suspensionPosition: Vec<f32>,      // Note: All wheel arrays have order: RL, RR, FL, FR
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_suspensionVelocity: Vec<f32>,      // RL, RR, FL, FR
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_suspensionAcceleration: Vec<f32>,  // RL, RR, FL, FR
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_wheelSpeed: Vec<f32>,              // Speed of each wheel
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_wheelSlipRatio: Vec<f32>,          // Slip ratio for each wheel
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_wheelSlipAngle: Vec<f32>,          // Slip angles for each wheel
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_wheelLatForce: Vec<f32>,           // Lateral forces for each wheel
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_wheelLongForce: Vec<f32>,          // Longitudinal forces for each wheel
    #[nom(Parse = "le_f32")]
    pub m_heightOfCOGAboveGround: f32,       // Height of centre of gravity above ground
    #[nom(Parse = "le_f32")]
    pub m_localVelocityX: f32,               // Velocity in local space – metres/s
    #[nom(Parse = "le_f32")]
    pub m_localVelocityY: f32,               // Velocity in local space
    #[nom(Parse = "le_f32")]
    pub m_localVelocityZ: f32,               // Velocity in local space
    #[nom(Parse = "le_f32")]
    pub m_angularVelocityX: f32,             // Angular velocity x-component – radians/s
    #[nom(Parse = "le_f32")]
    pub m_angularVelocityY: f32,             // Angular velocity y-component
    #[nom(Parse = "le_f32")]
    pub m_angularVelocityZ: f32,             // Angular velocity z-component
    #[nom(Parse = "le_f32")]
    pub m_angularAccelerationX: f32,         // Angular acceleration x-component – radians/s/s
    #[nom(Parse = "le_f32")]
    pub m_angularAccelerationY: f32,         // Angular acceleration y-component
    #[nom(Parse = "le_f32")]
    pub m_angularAccelerationZ: f32,         // Angular acceleration z-component
    #[nom(Parse = "le_f32")]
    pub m_frontWheelsAngle: f32,             // Current front wheels angle in radians
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_wheelVertForce: Vec<f32>,          // Vertical forces for each wheel
    #[nom(Parse = "le_f32")]
    pub m_frontAeroHeight: f32,              // Front plank edge height above road surface
    #[nom(Parse = "le_f32")]
    pub m_rearAeroHeight: f32,               // Rear plank edge height above road surface
    #[nom(Parse = "le_f32")]
    pub m_frontRollAngle: f32,               // Roll angle of the front suspension
    #[nom(Parse = "le_f32")]
    pub m_rearRollAngle: f32,                // Roll angle of the rear suspension
    #[nom(Parse = "le_f32")]
    pub m_chassisYaw: f32,                   // Yaw angle of the chassis relative to direction of motion
}

pub fn parse_car_motion_ex_packet_internal(input: &[u8]) -> IResult<&[u8], PacketMotionExData> {
    PacketMotionExData::parse(input)
} 