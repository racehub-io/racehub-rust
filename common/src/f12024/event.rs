use serde::{Deserialize, Serialize};
use nom_derive::{Nom, Parse};
use nom::IResult;
use nom::number::complete::{le_u8, le_f32, le_u32};
use nom::bytes::complete::take;
use super::header::PacketHeader;
use std::str;

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct FastestLap {
    #[nom(Parse = "le_u8")]
    pub vehicleIdx: u8,
    #[nom(Parse = "le_f32")]
    pub lapTime: f32,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct Retirement {
    #[nom(Parse = "le_u8")]
    pub vehicleIdx: u8,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct TeamMateInPits {
    #[nom(Parse = "le_u8")]
    pub vehicleIdx: u8,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct RaceWinner {
    #[nom(Parse = "le_u8")]
    pub vehicleIdx: u8,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct Penalty {
    #[nom(Parse = "le_u8")]
    pub penaltyType: u8,
    #[nom(Parse = "le_u8")]
    pub infringementType: u8,
    #[nom(Parse = "le_u8")]
    pub vehicleIdx: u8,
    #[nom(Parse = "le_u8")]
    pub otherVehicleIdx: u8,
    #[nom(Parse = "le_u8")]
    pub time: u8,
    #[nom(Parse = "le_u8")]
    pub lapNum: u8,
    #[nom(Parse = "le_u8")]
    pub placesGained: u8,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct SpeedTrap {
    #[nom(Parse = "le_u8")]
    pub vehicleIdx: u8,
    #[nom(Parse = "le_f32")]
    pub speed: f32,
    #[nom(Parse = "le_u8")]
    pub isOverallFastestInSession: u8,
    #[nom(Parse = "le_u8")]
    pub isDriverFastestInSession: u8,
    #[nom(Parse = "le_u8")]
    pub fastestVehicleIdxInSession: u8,
    #[nom(Parse = "le_f32")]
    pub fastestSpeedInSession: f32,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct StartLights {
    #[nom(Parse = "le_u8")]
    pub numLights: u8,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct DriveThroughPenaltyServed {
    #[nom(Parse = "le_u8")]
    pub vehicleIdx: u8,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct StopGoPenaltyServed {
    #[nom(Parse = "le_u8")]
    pub vehicleIdx: u8,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct Flashback {
    #[nom(Parse = "le_u32")]
    pub flashbackFrameIdentifier: u32,
    #[nom(Parse = "le_f32")]
    pub flashbackSessionTime: f32,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct Buttons {
    #[nom(Parse = "le_u32")]
    pub buttonStatus: u32,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct Overtake {
    #[nom(Parse = "le_u8")]
    pub overtakingVehicleIdx: u8,
    #[nom(Parse = "le_u8")]
    pub beingOvertakenVehicleIdx: u8,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct SafetyCar {
    #[nom(Parse = "le_u8")]
    pub safetyCarType: u8,
    #[nom(Parse = "le_u8")]
    pub eventType: u8,
}

#[derive(Debug, Serialize, Deserialize, Nom)]
pub struct Collision {
    #[nom(Parse = "le_u8")]
    pub vehicle1Idx: u8,
    #[nom(Parse = "le_u8")]
    pub vehicle2Idx: u8,
}

// Simple empty struct for events with no data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmptyEvent {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventDetails {
    FastestLap(FastestLap),
    Retirement(Retirement),
    TeamMateInPits(TeamMateInPits),
    RaceWinner(RaceWinner),
    Penalty(Penalty),
    SpeedTrap(SpeedTrap),
    StartLights(StartLights),
    DriveThroughPenaltyServed(DriveThroughPenaltyServed),
    StopGoPenaltyServed(StopGoPenaltyServed),
    Flashback(Flashback),
    Buttons(Buttons),
    Overtake(Overtake),
    SafetyCar(SafetyCar),
    Collision(Collision),
    // Events with no additional data
    SessionStarted(EmptyEvent),
    SessionEnded(EmptyEvent),
    DrsEnabled(EmptyEvent),
    DrsDisabled(EmptyEvent),
    ChequeredFlag(EmptyEvent),
    LightsOut(EmptyEvent),
    RedFlag(EmptyEvent),
    Unknown
}

// Structure to represent event data from the UDP feed
#[derive(Debug, Serialize, Deserialize)]
pub struct PacketEventData {
    pub m_header: PacketHeader,
    pub m_eventStringCode: String,  // 4-character code like "SSTA", "FTLP", etc.
    pub m_eventDetails: EventDetails  // Details specific to the event type
}

// Custom parser function to convert 4 bytes to a String
fn parse_event_code(input: &[u8]) -> IResult<&[u8], String> {
    // Take 4 bytes for the event code
    let (remaining, code_bytes) = take(4u8)(input)?;
    
    // Convert bytes to string, replacing invalid UTF-8 with placeholders
    let event_code = str::from_utf8(code_bytes)
        .unwrap_or("INVL") // Fallback to "INVL" for invalid UTF-8
        .to_string();
    
    Ok((remaining, event_code))
}

// Define a raw struct for parsing the basic event packet structure
#[derive(Debug, Nom)]
struct RawEventPacket {
    pub m_header: PacketHeader,
    #[nom(Parse = "parse_event_code")]
    pub m_eventStringCode: String,
    // The remaining bytes will be parsed based on the event code
}

pub fn parse_event_packet_internal(input: &[u8]) -> IResult<&[u8], PacketEventData> {
    // Parse the header and event code
    let (remaining, raw_packet) = RawEventPacket::parse(input)?;
    
    // Parse the appropriate event details based on the event code
    let event_details = match raw_packet.m_eventStringCode.as_str() {
        "FTLP" => {
            match FastestLap::parse(remaining) {
                Ok((_, data)) => EventDetails::FastestLap(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "RTMT" => {
            match Retirement::parse(remaining) {
                Ok((_, data)) => EventDetails::Retirement(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "TMPT" => {
            match TeamMateInPits::parse(remaining) {
                Ok((_, data)) => EventDetails::TeamMateInPits(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "RCWN" => {
            match RaceWinner::parse(remaining) {
                Ok((_, data)) => EventDetails::RaceWinner(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "PENA" => {
            match Penalty::parse(remaining) {
                Ok((_, data)) => EventDetails::Penalty(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "SPTP" => {
            match SpeedTrap::parse(remaining) {
                Ok((_, data)) => EventDetails::SpeedTrap(data),
                Err(_) => {
                    if remaining.len() >= 12 {
                        // Try to manually construct the SpeedTrap from the raw data
                        let speed_trap = SpeedTrap {
                            vehicleIdx: remaining[0],
                            speed: f32::from_le_bytes([remaining[1], remaining[2], remaining[3], remaining[4]]),
                            isOverallFastestInSession: remaining[5],
                            isDriverFastestInSession: remaining[6],
                            fastestVehicleIdxInSession: remaining[7],
                            fastestSpeedInSession: f32::from_le_bytes([remaining[8], remaining[9], remaining[10], remaining[11]]),
                        };
                        
                        EventDetails::SpeedTrap(speed_trap)
                    } else {
                        EventDetails::Unknown
                    }
                }
            }
        },
        "STLG" => {
            match StartLights::parse(remaining) {
                Ok((_, data)) => EventDetails::StartLights(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "DTSV" => {
            match DriveThroughPenaltyServed::parse(remaining) {
                Ok((_, data)) => EventDetails::DriveThroughPenaltyServed(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "SGSV" => {
            match StopGoPenaltyServed::parse(remaining) {
                Ok((_, data)) => EventDetails::StopGoPenaltyServed(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "FLBK" => {
            match Flashback::parse(remaining) {
                Ok((_, data)) => EventDetails::Flashback(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "BUTN" => {
            match Buttons::parse(remaining) {
                Ok((_, data)) => EventDetails::Buttons(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "OVTK" => {
            match Overtake::parse(remaining) {
                Ok((_, data)) => EventDetails::Overtake(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "SCAR" => {
            match SafetyCar::parse(remaining) {
                Ok((_, data)) => EventDetails::SafetyCar(data),
                Err(_) => EventDetails::Unknown
            }
        },
        "COLL" => {
            match Collision::parse(remaining) {
                Ok((_, data)) => EventDetails::Collision(data),
                Err(_) => EventDetails::Unknown
            }
        },
        // Events with no additional data
        "SSTA" => EventDetails::SessionStarted(EmptyEvent {}),
        "SEND" => EventDetails::SessionEnded(EmptyEvent {}),
        "DRSE" => EventDetails::DrsEnabled(EmptyEvent {}),
        "DRSD" => EventDetails::DrsDisabled(EmptyEvent {}),
        "CHQF" => EventDetails::ChequeredFlag(EmptyEvent {}),
        "LGOT" => EventDetails::LightsOut(EmptyEvent {}),
        "RDFL" => EventDetails::RedFlag(EmptyEvent {}),
        _ => EventDetails::Unknown,
    };
    
    // Construct the final packet with the parsed event details
    let packet = PacketEventData {
        m_header: raw_packet.m_header,
        m_eventStringCode: raw_packet.m_eventStringCode,
        m_eventDetails: event_details,
    };
    
    Ok((remaining, packet))
}
