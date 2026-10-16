use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_u32, le_f64};
use super::header::PacketHeader;

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct FinalClassificationData {
    #[nom(Parse = "le_u8")]
    pub m_position: u8,              // Finishing position
    #[nom(Parse = "le_u8")]
    pub m_numLaps: u8,               // Number of laps completed
    #[nom(Parse = "le_u8")]
    pub m_gridPosition: u8,          // Grid position of the car
    #[nom(Parse = "le_u8")]
    pub m_points: u8,                // Number of points scored
    #[nom(Parse = "le_u8")]
    pub m_numPitStops: u8,           // Number of pit stops made
    #[nom(Parse = "le_u8")]
    pub m_resultStatus: u8,          // Result status - 0 = invalid, 1 = inactive, 2 = active
                                    // 3 = finished, 4 = didnotfinish, 5 = disqualified
                                    // 6 = not classified, 7 = retired
    #[nom(Parse = "le_u32")]
    pub m_bestLapTimeInMS: u32,     // Best lap time of the session in milliseconds
    #[nom(Parse = "le_f64")]
    pub m_totalRaceTime: f64,       // Total race time in seconds without penalties
    #[nom(Parse = "le_u8")]
    pub m_penaltiesTime: u8,        // Total penalties accumulated in seconds
    #[nom(Parse = "le_u8")]
    pub m_numPenalties: u8,         // Number of penalties applied to this driver
    #[nom(Parse = "le_u8")]
    pub m_numTyreStints: u8,        // Number of tyres stints up to maximum
    #[nom(Count = "8", Parse = "le_u8")]
    pub m_tyreStintsActual: Vec<u8>,   // Actual tyres used by this driver
    #[nom(Count = "8", Parse = "le_u8")]
    pub m_tyreStintsVisual: Vec<u8>,   // Visual tyres used by this driver
    #[nom(Count = "8", Parse = "le_u8")]
    pub m_tyreStintsEndLaps: Vec<u8>,  // The lap number stints end on
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketFinalClassificationData {
    pub m_header: PacketHeader,
    #[nom(Parse = "le_u8")]
    pub m_numCars: u8,              // Number of cars in the final classification
    #[nom(Count = "22")]
    pub m_classificationData: Vec<FinalClassificationData>,
}

pub fn parse_final_classification_packet_internal(input: &[u8]) -> IResult<&[u8], PacketFinalClassificationData> {
    PacketFinalClassificationData::parse(input)
} 