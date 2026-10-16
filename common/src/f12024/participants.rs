use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_u16};
use nom::bytes::complete::take;
use super::header::PacketHeader;
use std::str;

// Custom parser function to convert null-terminated bytes to a String
fn parse_null_terminated_string(input: &[u8]) -> IResult<&[u8], String> {
    // Take 48 bytes for the name
    let (remaining, name_bytes) = take(48u8)(input)?;
    
    // Find the position of the null terminator
    let null_pos = name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len());
    
    // Convert bytes to string, stopping at the null terminator
    let name_str = match str::from_utf8(&name_bytes[0..null_pos]) {
        Ok(s) => s.trim().to_string(),
        Err(_) => String::from("Invalid UTF-8")
    };
    
    Ok((remaining, name_str))
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct ParticipantData {
    #[nom(Parse = "le_u8")]
    pub m_aiControlled: u8,           // Whether the vehicle is AI (1) or Human (0) controlled
    #[nom(Parse = "le_u8")]
    pub m_driverId: u8,              // Driver id - see appendix, 255 if network human
    #[nom(Parse = "le_u8")]
    pub m_networkId: u8,             // Network id – unique identifier for network players
    #[nom(Parse = "le_u8")]
    pub m_teamId: u8,                // Team id - see appendix
    #[nom(Parse = "le_u8")]
    pub m_myTeam: u8,                // My team flag – 1 = My Team, 0 = otherwise
    #[nom(Parse = "le_u8")]
    pub m_raceNumber: u8,            // Race number of the car
    #[nom(Parse = "le_u8")]
    pub m_nationality: u8,           // Nationality of the driver
    #[nom(Parse = "parse_null_terminated_string")]
    pub m_name: String,              // Name of participant in UTF-8 format
    #[nom(Parse = "le_u8")]
    pub m_yourTelemetry: u8,         // The player's UDP setting, 0 = restricted, 1 = public
    #[nom(Parse = "le_u8")]
    pub m_showOnlineNames: u8,       // 0 = not show online names, 1 = show online names
    #[nom(Parse = "le_u16")]
    pub m_techLevel: u16,            // F1 World tech level
    #[nom(Parse = "le_u8")]
    pub m_platform: u8,              // 1 = Steam, 3 = PlayStation, 4 = Xbox, 6 = Origin, 255 = unknown
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketParticipantsData {
    pub m_header: PacketHeader,
    #[nom(Parse = "le_u8")]
    pub m_numActiveCars: u8,         // Number of active cars in the data
    #[nom(Count = "22")]
    pub m_participants: Vec<ParticipantData>,
}

pub fn parse_participants_packet_internal(input: &[u8]) -> IResult<&[u8], PacketParticipantsData> {
    PacketParticipantsData::parse(input)
} 