#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GameMode {
    EventMode = 0,
    GrandPrix = 3,
    GrandPrix23 = 4,
    TimeTrial = 5,
    Splitscreen = 6,
    OnlineCustom = 7,
    OnlineLeague = 8,
    CareerInvitational = 11,
    ChampionshipInvitational = 12,
    Championship = 13,
    OnlineChampionship = 14,
    OnlineWeeklyEvent = 15,
    StoryMode = 17,
    Career22 = 19,
    Career22Online = 20,
    Career23 = 21,
    Career23Online = 22,
    DriverCareer24 = 23,
    Career24Online = 24,
    MyTeamCareer24 = 25,
    CuratedCareer24 = 26,
    Benchmark = 127,
}

impl GameMode {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::EventMode),
            3 => Some(Self::GrandPrix),
            4 => Some(Self::GrandPrix23),
            5 => Some(Self::TimeTrial),
            6 => Some(Self::Splitscreen),
            7 => Some(Self::OnlineCustom),
            8 => Some(Self::OnlineLeague),
            11 => Some(Self::CareerInvitational),
            12 => Some(Self::ChampionshipInvitational),
            13 => Some(Self::Championship),
            14 => Some(Self::OnlineChampionship),
            15 => Some(Self::OnlineWeeklyEvent),
            17 => Some(Self::StoryMode),
            19 => Some(Self::Career22),
            20 => Some(Self::Career22Online),
            21 => Some(Self::Career23),
            22 => Some(Self::Career23Online),
            23 => Some(Self::DriverCareer24),
            24 => Some(Self::Career24Online),
            25 => Some(Self::MyTeamCareer24),
            26 => Some(Self::CuratedCareer24),
            127 => Some(Self::Benchmark),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::EventMode => "Event Mode",
            Self::GrandPrix => "Grand Prix",
            Self::GrandPrix23 => "Grand Prix '23",
            Self::TimeTrial => "Time Trial",
            Self::Splitscreen => "Splitscreen",
            Self::OnlineCustom => "Online Custom",
            Self::OnlineLeague => "Online League",
            Self::CareerInvitational => "Career Invitational",
            Self::ChampionshipInvitational => "Championship Invitational",
            Self::Championship => "Championship",
            Self::OnlineChampionship => "Online Championship",
            Self::OnlineWeeklyEvent => "Online Weekly Event",
            Self::StoryMode => "Story Mode",
            Self::Career22 => "Career '22",
            Self::Career22Online => "Career '22 Online",
            Self::Career23 => "Career '23",
            Self::Career23Online => "Career '23 Online",
            Self::DriverCareer24 => "Driver Career '24",
            Self::Career24Online => "Career '24 Online",
            Self::MyTeamCareer24 => "My Team Career '24",
            Self::CuratedCareer24 => "Curated Career '24",
            Self::Benchmark => "Benchmark",
        }
    }
} 