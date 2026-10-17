use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_i8, le_u8, le_u16, le_u32, le_f32};
use super::header::PacketHeader;

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct MarshalZone {
    #[nom(Parse = "le_f32")]
    pub m_zoneStart: f32,  // Fraction (0..1) of way through the lap
    #[nom(Parse = "le_i8")]
    pub m_zoneFlag: i8,    // -1 = invalid/unknown, 0 = none, 1 = green, 2 = blue, 3 = yellow
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct WeatherForecastSample {
    #[nom(Parse = "le_u8")]
    pub m_sessionType: u8,              // 0 = unknown, see appendix
    #[nom(Parse = "le_u8")]
    pub m_timeOffset: u8,               // Time in minutes the forecast is for
    #[nom(Parse = "le_u8")]
    pub m_weather: u8,                  // Weather - 0 = clear, 1 = light cloud, 2 = overcast
    #[nom(Parse = "le_i8")]
    pub m_trackTemperature: i8,         // Track temp. in degrees Celsius
    #[nom(Parse = "le_i8")]
    pub m_trackTemperatureChange: i8,   // Track temp. change – 0 = up, 1 = down, 2 = no change
    #[nom(Parse = "le_i8")]
    pub m_airTemperature: i8,           // Air temp. in degrees celsius
    #[nom(Parse = "le_i8")]
    pub m_airTemperatureChange: i8,     // Air temp. change – 0 = up, 1 = down, 2 = no change
    #[nom(Parse = "le_u8")]
    pub m_rainPercentage: u8,           // Rain percentage (0-100)
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct PacketSessionData {
    pub m_header: PacketHeader,
    #[nom(Parse = "le_u8")]
    pub m_weather: u8,
    #[nom(Parse = "le_i8")]
    pub m_trackTemperature: i8,
    #[nom(Parse = "le_i8")]
    pub m_airTemperature: i8,
    #[nom(Parse = "le_u8")]
    pub m_totalLaps: u8,
    #[nom(Parse = "le_u16")]
    pub m_trackLength: u16,
    #[nom(Parse = "le_u8")]
    pub m_sessionType: u8,
    #[nom(Parse = "le_i8")]
    pub m_trackId: i8,
    #[nom(Parse = "le_u8")]
    pub m_formula: u8,
    #[nom(Parse = "le_u16")]
    pub m_sessionTimeLeft: u16,
    #[nom(Parse = "le_u16")]
    pub m_sessionDuration: u16,
    #[nom(Parse = "le_u8")]
    pub m_pitSpeedLimit: u8,
    #[nom(Parse = "le_u8")]
    pub m_gamePaused: u8,
    #[nom(Parse = "le_u8")]
    pub m_isSpectating: u8,
    #[nom(Parse = "le_u8")]
    pub m_spectatorCarIndex: u8,
    #[nom(Parse = "le_u8")]
    pub m_sliProNativeSupport: u8,
    #[nom(Parse = "le_u8")]
    pub m_numMarshalZones: u8,
    #[nom(Count = "21")]
    pub m_marshalZones: Vec<MarshalZone>,
    #[nom(Parse = "le_u8")]
    pub m_safetyCarStatus: u8,
    #[nom(Parse = "le_u8")]
    pub m_networkGame: u8,
    #[nom(Parse = "le_u8")]
    pub m_numWeatherForecastSamples: u8,
    #[nom(Count = "64")]
    pub m_weatherForecastSamples: Vec<WeatherForecastSample>,
    #[nom(Parse = "le_u8")]
    pub m_forecastAccuracy: u8,
    #[nom(Parse = "le_u8")]
    pub m_aiDifficulty: u8,
    #[nom(Parse = "le_u32")]
    pub m_seasonLinkIdentifier: u32,
    #[nom(Parse = "le_u32")]
    pub m_weekendLinkIdentifier: u32,
    #[nom(Parse = "le_u32")]
    pub m_sessionLinkIdentifier: u32,
    #[nom(Parse = "le_u8")]
    pub m_pitStopWindowIdealLap: u8,
    #[nom(Parse = "le_u8")]
    pub m_pitStopWindowLatestLap: u8,
    #[nom(Parse = "le_u8")]
    pub m_pitStopRejoinPosition: u8,
    #[nom(Parse = "le_u8")]
    pub m_steeringAssist: u8,
    #[nom(Parse = "le_u8")]
    pub m_brakingAssist: u8,
    #[nom(Parse = "le_u8")]
    pub m_gearboxAssist: u8,
    #[nom(Parse = "le_u8")]
    pub m_pitAssist: u8,
    #[nom(Parse = "le_u8")]
    pub m_pitReleaseAssist: u8,
    #[nom(Parse = "le_u8")]
    pub m_ERSAssist: u8,
    #[nom(Parse = "le_u8")]
    pub m_DRSAssist: u8,
    #[nom(Parse = "le_u8")]
    pub m_dynamicRacingLine: u8,
    #[nom(Parse = "le_u8")]
    pub m_dynamicRacingLineType: u8,
    #[nom(Parse = "le_u8")]
    pub m_gameMode: u8,
    #[nom(Parse = "le_u8")]
    pub m_ruleSet: u8,
    #[nom(Parse = "le_u32")]
    pub m_timeOfDay: u32,
    #[nom(Parse = "le_u8")]
    pub m_sessionLength: u8,
    #[nom(Parse = "le_u8")]
    pub m_speedUnitsLeadPlayer: u8,
    #[nom(Parse = "le_u8")]
    pub m_temperatureUnitsLeadPlayer: u8,
    #[nom(Parse = "le_u8")]
    pub m_speedUnitsSecondaryPlayer: u8,
    #[nom(Parse = "le_u8")]
    pub m_temperatureUnitsSecondaryPlayer: u8,
    #[nom(Parse = "le_u8")]
    pub m_numSafetyCarPeriods: u8,
    #[nom(Parse = "le_u8")]
    pub m_numVirtualSafetyCarPeriods: u8,
    #[nom(Parse = "le_u8")]
    pub m_numRedFlagPeriods: u8,
    #[nom(Parse = "le_u8")]
    pub m_equalCarPerformance: u8,
    #[nom(Parse = "le_u8")]
    pub m_recoveryMode: u8,
    #[nom(Parse = "le_u8")]
    pub m_flashbackLimit: u8,
    #[nom(Parse = "le_u8")]
    pub m_surfaceType: u8,
    #[nom(Parse = "le_u8")]
    pub m_lowFuelMode: u8,
    #[nom(Parse = "le_u8")]
    pub m_raceStarts: u8,
    #[nom(Parse = "le_u8")]
    pub m_tyreTemperature: u8,
    #[nom(Parse = "le_u8")]
    pub m_pitLaneTyreSim: u8,
    #[nom(Parse = "le_u8")]
    pub m_carDamage: u8,
    #[nom(Parse = "le_u8")]
    pub m_carDamageRate: u8,
    #[nom(Parse = "le_u8")]
    pub m_collisions: u8,
    #[nom(Parse = "le_u8")]
    pub m_collisionsOffForFirstLapOnly: u8,
    #[nom(Parse = "le_u8")]
    pub m_mpUnsafePitRelease: u8,
    #[nom(Parse = "le_u8")]
    pub m_mpOffForGriefing: u8,
    #[nom(Parse = "le_u8")]
    pub m_cornerCuttingStringency: u8,
    #[nom(Parse = "le_u8")]
    pub m_parcFermeRules: u8,
    #[nom(Parse = "le_u8")]
    pub m_pitStopExperience: u8,
    #[nom(Parse = "le_u8")]
    pub m_safetyCar: u8,
    #[nom(Parse = "le_u8")]
    pub m_safetyCarExperience: u8,
    #[nom(Parse = "le_u8")]
    pub m_formationLap: u8,
    #[nom(Parse = "le_u8")]
    pub m_formationLapExperience: u8,
    #[nom(Parse = "le_u8")]
    pub m_redFlags: u8,
    #[nom(Parse = "le_u8")]
    pub m_affectsLicenceLevelSolo: u8,
    #[nom(Parse = "le_u8")]
    pub m_affectsLicenceLevelMP: u8,
    #[nom(Parse = "le_u8")]
    pub m_numSessionsInWeekend: u8,
    #[nom(Count = "12", Parse = "le_u8")]
    pub m_weekendStructure: Vec<u8>,
    #[nom(Parse = "le_f32")]
    pub m_sector2LapDistanceStart: f32,
    #[nom(Parse = "le_f32")]
    pub m_sector3LapDistanceStart: f32,
}

pub fn parse_session_packet_internal(input: &[u8]) -> IResult<&[u8], PacketSessionData> {
    PacketSessionData::parse(input)
}

            

            
            
