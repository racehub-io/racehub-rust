use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_i16};
use super::header::PacketHeader;

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct TyreSetData {
    #[nom(Parse = "le_u8")]
    pub m_actualTyreCompound: u8,     // Actual tyre compound used
    #[nom(Parse = "le_u8")]
    pub m_visualTyreCompound: u8,     // Visual tyre compound used
    #[nom(Parse = "le_u8")]
    pub m_wear: u8,                   // Tyre wear (percentage)
    #[nom(Parse = "le_u8")]
    pub m_available: u8,              // Whether this set is currently available
    #[nom(Parse = "le_u8")]
    pub m_recommendedSession: u8,      // Recommended session for tyre set, see appendix
    #[nom(Parse = "le_u8")]
    pub m_lifeSpan: u8,               // Laps left in this tyre set
    #[nom(Parse = "le_u8")]
    pub m_usableLife: u8,             // Max number of laps recommended for this compound
    #[nom(Parse = "le_i16")]
    pub m_lapDeltaTime: i16,          // Lap delta time in milliseconds compared to fitted set
    #[nom(Parse = "le_u8")]
    pub m_fitted: u8,                 // Whether the set is fitted or not
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketTyreSetsData {
    pub m_header: PacketHeader,
    #[nom(Parse = "le_u8")]
    pub m_carIdx: u8,                 // Index of the car this data relates to
    #[nom(Count = "20")]
    pub m_tyreSetData: Vec<TyreSetData>, // 13 (dry) + 7 (wet)
    #[nom(Parse = "le_u8")]
    pub m_fittedIdx: u8,              // Index into array of fitted tyre
}

pub fn parse_tyre_sets_packet_internal(input: &[u8]) -> IResult<&[u8], PacketTyreSetsData> {
    PacketTyreSetsData::parse(input)
} 