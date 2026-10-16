use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_u16, le_u32, le_f32};
use super::header::PacketHeader;

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct LapData {
    #[nom(Parse = "le_u32")]
    pub m_lastLapTimeInMS: u32,           // Last lap time in milliseconds
    #[nom(Parse = "le_u32")]
    pub m_currentLapTimeInMS: u32,        // Current time around the lap in milliseconds
    #[nom(Parse = "le_u16")]
    pub m_sector1TimeMSPart: u16,         // Sector 1 time milliseconds part
    #[nom(Parse = "le_u8")]
    pub m_sector1TimeMinutesPart: u8,     // Sector 1 whole minute part
    #[nom(Parse = "le_u16")]
    pub m_sector2TimeMSPart: u16,         // Sector 2 time milliseconds part
    #[nom(Parse = "le_u8")]
    pub m_sector2TimeMinutesPart: u8,     // Sector 2 whole minute part
    #[nom(Parse = "le_u16")]
    pub m_deltaToCarInFrontMSPart: u16,   // Time delta to car in front milliseconds part
    #[nom(Parse = "le_u8")]
    pub m_deltaToCarInFrontMinutesPart: u8, // Time delta to car in front whole minute part
    #[nom(Parse = "le_u16")]
    pub m_deltaToRaceLeaderMSPart: u16,   // Time delta to race leader milliseconds part
    #[nom(Parse = "le_u8")]
    pub m_deltaToRaceLeaderMinutesPart: u8, // Time delta to race leader whole minute part
    #[nom(Parse = "le_f32")]
    pub m_lapDistance: f32,               // Distance vehicle is around current lap in metres
    #[nom(Parse = "le_f32")]
    pub m_totalDistance: f32,             // Total distance travelled in session in metres
    #[nom(Parse = "le_f32")]
    pub m_safetyCarDelta: f32,           // Delta in seconds for safety car
    #[nom(Parse = "le_u8")]
    pub m_carPosition: u8,                // Car race position
    #[nom(Parse = "le_u8")]
    pub m_currentLapNum: u8,              // Current lap number
    #[nom(Parse = "le_u8")]
    pub m_pitStatus: u8,                  // 0 = none, 1 = pitting, 2 = in pit area
    #[nom(Parse = "le_u8")]
    pub m_numPitStops: u8,                // Number of pit stops taken in this race
    #[nom(Parse = "le_u8")]
    pub m_sector: u8,                     // 0 = sector1, 1 = sector2, 2 = sector3
    #[nom(Parse = "le_u8")]
    pub m_currentLapInvalid: u8,          // Current lap invalid - 0 = valid, 1 = invalid
    #[nom(Parse = "le_u8")]
    pub m_penalties: u8,                  // Accumulated time penalties in seconds
    #[nom(Parse = "le_u8")]
    pub m_totalWarnings: u8,              // Accumulated number of warnings issued
    #[nom(Parse = "le_u8")]
    pub m_cornerCuttingWarnings: u8,      // Accumulated number of corner cutting warnings issued
    #[nom(Parse = "le_u8")]
    pub m_numUnservedDriveThroughPens: u8, // Num drive through pens left to serve
    #[nom(Parse = "le_u8")]
    pub m_numUnservedStopGoPens: u8,      // Num stop go pens left to serve
    #[nom(Parse = "le_u8")]
    pub m_gridPosition: u8,               // Grid position the vehicle started the race in
    #[nom(Parse = "le_u8")]
    pub m_driverStatus: u8,               // Status of driver - 0 = in garage, 1 = flying lap
    #[nom(Parse = "le_u8")]
    pub m_resultStatus: u8,               // Result status - 0 = invalid, 1 = inactive, 2 = active
    #[nom(Parse = "le_u8")]
    pub m_pitLaneTimerActive: u8,         // Pit lane timing, 0 = inactive, 1 = active
    #[nom(Parse = "le_u16")]
    pub m_pitLaneTimeInLaneInMS: u16,     // If active, the current time spent in the pit lane in ms
    #[nom(Parse = "le_u16")]
    pub m_pitStopTimerInMS: u16,          // Time of the actual pit stop in ms
    #[nom(Parse = "le_u8")]
    pub m_pitStopShouldServePen: u8,      // Whether the car should serve a penalty at this stop
    #[nom(Parse = "le_f32")]
    pub m_speedTrapFastestSpeed: f32,     // Fastest speed through speed trap in kmph
    #[nom(Parse = "le_u8")]
    pub m_speedTrapFastestLap: u8,        // Lap no the fastest speed was achieved, 255 = not set
}

#[derive(Debug, Deserialize, Serialize, Nom)]
pub struct PacketLapData {
    pub m_header: PacketHeader,
    #[nom(Count = "22")]
    pub m_lapData: Vec<LapData>,          // Lap data for all cars on track
    #[nom(Parse = "le_u8")]
    pub m_timeTrialPBCarIdx: u8,          // Index of Personal Best car in time trial (255 if invalid)
    #[nom(Parse = "le_u8")]
    pub m_timeTrialRivalCarIdx: u8,       // Index of Rival car in time trial (255 if invalid)
}

pub fn parse_lap_data_packet_internal(input: &[u8]) -> IResult<&[u8], PacketLapData> {
    PacketLapData::parse(input)
}