use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_f32};
use super::header::PacketHeader;

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct CarDamageData {
    #[nom(Count = "4", Parse = "le_f32")]
    pub m_tyresWear: Vec<f32>,                // Tyre wear (percentage)
    #[nom(Count = "4", Parse = "le_u8")]
    pub m_tyresDamage: Vec<u8>,               // Tyre damage (percentage)
    #[nom(Count = "4", Parse = "le_u8")]
    pub m_brakesDamage: Vec<u8>,              // Brakes damage (percentage)
    #[nom(Parse = "le_u8")]
    pub m_frontLeftWingDamage: u8,            // Front left wing damage (percentage)
    #[nom(Parse = "le_u8")]
    pub m_frontRightWingDamage: u8,           // Front right wing damage (percentage)
    #[nom(Parse = "le_u8")]
    pub m_rearWingDamage: u8,                 // Rear wing damage (percentage)
    #[nom(Parse = "le_u8")]
    pub m_floorDamage: u8,                    // Floor damage (percentage)
    #[nom(Parse = "le_u8")]
    pub m_diffuserDamage: u8,                 // Diffuser damage (percentage)
    #[nom(Parse = "le_u8")]
    pub m_sidepodDamage: u8,                  // Sidepod damage (percentage)
    #[nom(Parse = "le_u8")]
    pub m_drsFault: u8,                       // Indicator for DRS fault, 0 = OK, 1 = fault
    #[nom(Parse = "le_u8")]
    pub m_ersFault: u8,                       // Indicator for ERS fault, 0 = OK, 1 = fault
    #[nom(Parse = "le_u8")]
    pub m_gearBoxDamage: u8,                  // Gear box damage (percentage)
    #[nom(Parse = "le_u8")]
    pub m_engineDamage: u8,                   // Engine damage (percentage)
    #[nom(Parse = "le_u8")]
    pub m_engineMGUHWear: u8,                 // Engine wear MGU-H (percentage)
    #[nom(Parse = "le_u8")]
    pub m_engineESWear: u8,                   // Engine wear ES (percentage)
    #[nom(Parse = "le_u8")]
    pub m_engineCEWear: u8,                   // Engine wear CE (percentage)
    #[nom(Parse = "le_u8")]
    pub m_engineICEWear: u8,                  // Engine wear ICE (percentage)
    #[nom(Parse = "le_u8")]
    pub m_engineMGUKWear: u8,                 // Engine wear MGU-K (percentage)
    #[nom(Parse = "le_u8")]
    pub m_engineTCWear: u8,                   // Engine wear TC (percentage)
    #[nom(Parse = "le_u8")]
    pub m_engineBlown: u8,                    // Engine blown, 0 = OK, 1 = fault
    #[nom(Parse = "le_u8")]
    pub m_engineSeized: u8,                   // Engine seized, 0 = OK, 1 = fault
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketCarDamageData {
    pub m_header: PacketHeader,
    #[nom(Count = "22")]
    pub m_carDamageData: Vec<CarDamageData>,
}

pub fn parse_car_damage_packet_internal(input: &[u8]) -> IResult<&[u8], PacketCarDamageData> {
    PacketCarDamageData::parse(input)
} 