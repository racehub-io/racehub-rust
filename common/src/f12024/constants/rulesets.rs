#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Ruleset {
    PracticeAndQualifying = 0,
    Race = 1,
    TimeTrial = 2,
    TimeAttack = 4,
    CheckpointChallenge = 6,
    Autocross = 8,
    Drift = 9,
    AverageSpeedZone = 10,
    RivalDuel = 11,
}

impl Ruleset {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::PracticeAndQualifying),
            1 => Some(Self::Race),
            2 => Some(Self::TimeTrial),
            4 => Some(Self::TimeAttack),
            6 => Some(Self::CheckpointChallenge),
            8 => Some(Self::Autocross),
            9 => Some(Self::Drift),
            10 => Some(Self::AverageSpeedZone),
            11 => Some(Self::RivalDuel),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::PracticeAndQualifying => "Practice & Qualifying",
            Self::Race => "Race",
            Self::TimeTrial => "Time Trial",
            Self::TimeAttack => "Time Attack",
            Self::CheckpointChallenge => "Checkpoint Challenge",
            Self::Autocross => "Autocross",
            Self::Drift => "Drift",
            Self::AverageSpeedZone => "Average Speed Zone",
            Self::RivalDuel => "Rival Duel",
        }
    }
} 