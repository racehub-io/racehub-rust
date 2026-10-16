use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_u32};
use super::header::PacketHeader;

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct TimeTrialDataSet {
    #[nom(Parse = "le_u8")]
    pub m_carIdx: u8,                   // Index of the car this data relates to
    #[nom(Parse = "le_u8")]
    pub m_teamId: u8,                   // Team id - see appendix
    #[nom(Parse = "le_u32")]
    pub m_lapTimeInMS: u32,             // Lap time in milliseconds
    #[nom(Parse = "le_u32")]
    pub m_sector1TimeInMS: u32,         // Sector 1 time in milliseconds
    #[nom(Parse = "le_u32")]
    pub m_sector2TimeInMS: u32,         // Sector 2 time in milliseconds
    #[nom(Parse = "le_u32")]
    pub m_sector3TimeInMS: u32,         // Sector 3 time in milliseconds
    #[nom(Parse = "le_u8")]
    pub m_tractionControl: u8,          // 0 = off, 1 = medium, 2 = full
    #[nom(Parse = "le_u8")]
    pub m_gearboxAssist: u8,            // 1 = manual, 2 = manual & suggested gear, 3 = auto
    #[nom(Parse = "le_u8")]
    pub m_antiLockBrakes: u8,           // 0 (off) - 1 (on)
    #[nom(Parse = "le_u8")]
    pub m_equalCarPerformance: u8,      // 0 = Realistic, 1 = Equal
    #[nom(Parse = "le_u8")]
    pub m_customSetup: u8,              // 0 = No, 1 = Yes
    #[nom(Parse = "le_u8")]
    pub m_valid: u8,                    // 0 = invalid, 1 = valid
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketTimeTrialData {
    pub m_header: PacketHeader,
    pub m_playerSessionBestDataSet: TimeTrialDataSet,  // Player session best data set
    pub m_personalBestDataSet: TimeTrialDataSet,       // Personal best data set
    pub m_rivalDataSet: TimeTrialDataSet,              // Rival data set
}

pub fn parse_time_trial_packet_internal(input: &[u8]) -> IResult<&[u8], PacketTimeTrialData> {
    PacketTimeTrialData::parse(input)
} 