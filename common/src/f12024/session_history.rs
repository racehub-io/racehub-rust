use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_u16, le_u32};
use super::header::PacketHeader;

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct LapHistoryData {
    #[nom(Parse = "le_u32")]
    pub m_lapTimeInMS: u32,              // Lap time in milliseconds
    #[nom(Parse = "le_u16")]
    pub m_sector1TimeMSPart: u16,        // Sector 1 milliseconds part
    #[nom(Parse = "le_u8")]
    pub m_sector1TimeMinutesPart: u8,    // Sector 1 whole minute part
    #[nom(Parse = "le_u16")]
    pub m_sector2TimeMSPart: u16,        // Sector 2 time milliseconds part
    #[nom(Parse = "le_u8")]
    pub m_sector2TimeMinutesPart: u8,    // Sector 2 whole minute part
    #[nom(Parse = "le_u16")]
    pub m_sector3TimeMSPart: u16,        // Sector 3 time milliseconds part
    #[nom(Parse = "le_u8")]
    pub m_sector3TimeMinutesPart: u8,    // Sector 3 whole minute part
    #[nom(Parse = "le_u8")]
    pub m_lapValidBitFlags: u8,          // 0x01 bit set-lap valid, 0x02 bit set-sector 1 valid
                                         // 0x04 bit set-sector 2 valid, 0x08 bit set-sector 3 valid
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct TyreStintHistoryData {
    #[nom(Parse = "le_u8")]
    pub m_endLap: u8,                    // Lap the tyre usage ends on (255 of current tyre)
    #[nom(Parse = "le_u8")]
    pub m_tyreActualCompound: u8,        // Actual tyres used by this driver
    #[nom(Parse = "le_u8")]
    pub m_tyreVisualCompound: u8,        // Visual tyres used by this driver
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketSessionHistoryData {
    pub m_header: PacketHeader,
    #[nom(Parse = "le_u8")]
    pub m_carIdx: u8,                    // Index of the car this lap data relates to
    #[nom(Parse = "le_u8")]
    pub m_numLaps: u8,                   // Num laps in the data (including current partial lap)
    #[nom(Parse = "le_u8")]
    pub m_numTyreStints: u8,             // Number of tyre stints in the data
    #[nom(Parse = "le_u8")]
    pub m_bestLapTimeLapNum: u8,         // Lap the best lap time was achieved on
    #[nom(Parse = "le_u8")]
    pub m_bestSector1LapNum: u8,         // Lap the best Sector 1 time was achieved on
    #[nom(Parse = "le_u8")]
    pub m_bestSector2LapNum: u8,         // Lap the best Sector 2 time was achieved on
    #[nom(Parse = "le_u8")]
    pub m_bestSector3LapNum: u8,         // Lap the best Sector 3 time was achieved on
    #[nom(Count = "100")]
    pub m_lapHistoryData: Vec<LapHistoryData>,  // 100 laps of data max
    #[nom(Count = "8")]
    pub m_tyreStintsHistoryData: Vec<TyreStintHistoryData>,
}

pub fn parse_session_history_packet_internal(input: &[u8]) -> IResult<&[u8], PacketSessionHistoryData> {
    PacketSessionHistoryData::parse(input)
} 