#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PenaltyType {
    DriveThrough = 0,
    StopGo = 1,
    GridPenalty = 2,
    PenaltyReminder = 3,
    TimePenalty = 4,
    Warning = 5,
    Disqualified = 6,
    RemovedFromFormationLap = 7,
    ParkedTooLongTimer = 8,
    TyreRegulations = 9,
    ThisLapInvalidated = 10,
    ThisAndNextLapInvalidated = 11,
    ThisLapInvalidatedWithoutReason = 12,
    ThisAndNextLapInvalidatedWithoutReason = 13,
    ThisAndPreviousLapInvalidated = 14,
    ThisAndPreviousLapInvalidatedWithoutReason = 15,
    Retired = 16,
    BlackFlagTimer = 17,
}

impl PenaltyType {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::DriveThrough),
            1 => Some(Self::StopGo),
            2 => Some(Self::GridPenalty),
            3 => Some(Self::PenaltyReminder),
            4 => Some(Self::TimePenalty),
            5 => Some(Self::Warning),
            6 => Some(Self::Disqualified),
            7 => Some(Self::RemovedFromFormationLap),
            8 => Some(Self::ParkedTooLongTimer),
            9 => Some(Self::TyreRegulations),
            10 => Some(Self::ThisLapInvalidated),
            11 => Some(Self::ThisAndNextLapInvalidated),
            12 => Some(Self::ThisLapInvalidatedWithoutReason),
            13 => Some(Self::ThisAndNextLapInvalidatedWithoutReason),
            14 => Some(Self::ThisAndPreviousLapInvalidated),
            15 => Some(Self::ThisAndPreviousLapInvalidatedWithoutReason),
            16 => Some(Self::Retired),
            17 => Some(Self::BlackFlagTimer),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::DriveThrough => "Drive through",
            Self::StopGo => "Stop Go",
            Self::GridPenalty => "Grid penalty",
            Self::PenaltyReminder => "Penalty reminder",
            Self::TimePenalty => "Time penalty",
            Self::Warning => "Warning",
            Self::Disqualified => "Disqualified",
            Self::RemovedFromFormationLap => "Removed from formation lap",
            Self::ParkedTooLongTimer => "Parked too long timer",
            Self::TyreRegulations => "Tyre regulations",
            Self::ThisLapInvalidated => "This lap invalidated",
            Self::ThisAndNextLapInvalidated => "This and next lap invalidated",
            Self::ThisLapInvalidatedWithoutReason => "This lap invalidated without reason",
            Self::ThisAndNextLapInvalidatedWithoutReason => "This and next lap invalidated without reason",
            Self::ThisAndPreviousLapInvalidated => "This and previous lap invalidated",
            Self::ThisAndPreviousLapInvalidatedWithoutReason => "This and previous lap invalidated without reason",
            Self::Retired => "Retired",
            Self::BlackFlagTimer => "Black flag timer",
        }
    }
} 