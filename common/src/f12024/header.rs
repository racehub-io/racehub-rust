use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_u16, le_u32, le_u64, le_f32};

// Define the packet type (adjust fields according to your needs)
#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct PacketHeader {
    #[nom(Parse = "le_u16")]
    pub m_packetFormat: u16,            // 2024
    #[nom(Parse = "le_u8")]
    pub m_gameYear: u8,                 // Game year - last two digits e.g. 23
    #[nom(Parse = "le_u8")]
    pub m_gameMajorVersion: u8,         // Game major version - "X.00"
    #[nom(Parse = "le_u8")]
    pub m_gameMinorVersion: u8,         // Game minor version - "1.XX"
    #[nom(Parse = "le_u8")]
    pub m_packetVersion: u8,            // Version of this packet type, all start from 1
    #[nom(Parse = "le_u8")]
    pub m_packetId: u8,                 // Identifier for the packet type
    #[nom(Parse = "le_u64")]
    pub m_sessionUID: u64,              // Unique identifier for the session
    #[nom(Parse = "le_f32")]
    pub m_sessionTime: f32,             // Session timestamp
    #[nom(Parse = "le_u32")]
    pub m_frameIdentifier: u32,         // Identifier for the frame the data was retrieved on
    #[nom(Parse = "le_u32")]
    pub m_overallFrameIdentifier: u32,  // Overall identifier for the frame the data was retrieved on
    #[nom(Parse = "le_u8")]
    pub m_playerCarIndex: u8,           // Index of player's car in the array
    #[nom(Parse = "le_u8")]
    pub m_secondaryPlayerCarIndex: u8,  // Index of secondary player's car in the array when split screen is enabled
}

pub fn parse_header_internal(input: &[u8]) -> IResult<&[u8], PacketHeader> {
    PacketHeader::parse(input)
}