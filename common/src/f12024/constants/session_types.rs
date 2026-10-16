#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SessionType {
    Unknown = 0,
    Practice1 = 1,
    Practice2 = 2,
    Practice3 = 3,
    ShortPractice = 4,
    Qualifying1 = 5,
    Qualifying2 = 6,
    Qualifying3 = 7,
    ShortQualifying = 8,
    OneShotQualifying = 9,
    SprintShootout1 = 10,
    SprintShootout2 = 11,
    SprintShootout3 = 12,
    ShortSprintShootout = 13,
    OneShotSprintShootout = 14,
    Race = 15,
    Race2 = 16,
    Race3 = 17,
    TimeTrial = 18,
}

impl SessionType {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::Unknown),
            1 => Some(Self::Practice1),
            2 => Some(Self::Practice2),
            3 => Some(Self::Practice3),
            4 => Some(Self::ShortPractice),
            5 => Some(Self::Qualifying1),
            6 => Some(Self::Qualifying2),
            7 => Some(Self::Qualifying3),
            8 => Some(Self::ShortQualifying),
            9 => Some(Self::OneShotQualifying),
            10 => Some(Self::SprintShootout1),
            11 => Some(Self::SprintShootout2),
            12 => Some(Self::SprintShootout3),
            13 => Some(Self::ShortSprintShootout),
            14 => Some(Self::OneShotSprintShootout),
            15 => Some(Self::Race),
            16 => Some(Self::Race2),
            17 => Some(Self::Race3),
            18 => Some(Self::TimeTrial),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Practice1 => "Practice 1",
            Self::Practice2 => "Practice 2",
            Self::Practice3 => "Practice 3",
            Self::ShortPractice => "Short Practice",
            Self::Qualifying1 => "Qualifying 1",
            Self::Qualifying2 => "Qualifying 2",
            Self::Qualifying3 => "Qualifying 3",
            Self::ShortQualifying => "Short Qualifying",
            Self::OneShotQualifying => "One-Shot Qualifying",
            Self::SprintShootout1 => "Sprint Shootout 1",
            Self::SprintShootout2 => "Sprint Shootout 2",
            Self::SprintShootout3 => "Sprint Shootout 3",
            Self::ShortSprintShootout => "Short Sprint Shootout",
            Self::OneShotSprintShootout => "One-Shot Sprint Shootout",
            Self::Race => "Race",
            Self::Race2 => "Race 2",
            Self::Race3 => "Race 3",
            Self::TimeTrial => "Time Trial",
        }
    }
} 