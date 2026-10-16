#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SurfaceType {
    Tarmac = 0,
    RumbleStrip = 1,
    Concrete = 2,
    Rock = 3,
    Gravel = 4,
    Mud = 5,
    Sand = 6,
    Grass = 7,
    Water = 8,
    Cobblestone = 9,
    Metal = 10,
    Ridged = 11,
}

impl SurfaceType {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::Tarmac),
            1 => Some(Self::RumbleStrip),
            2 => Some(Self::Concrete),
            3 => Some(Self::Rock),
            4 => Some(Self::Gravel),
            5 => Some(Self::Mud),
            6 => Some(Self::Sand),
            7 => Some(Self::Grass),
            8 => Some(Self::Water),
            9 => Some(Self::Cobblestone),
            10 => Some(Self::Metal),
            11 => Some(Self::Ridged),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Tarmac => "Tarmac",
            Self::RumbleStrip => "Rumble strip",
            Self::Concrete => "Concrete",
            Self::Rock => "Rock",
            Self::Gravel => "Gravel",
            Self::Mud => "Mud",
            Self::Sand => "Sand",
            Self::Grass => "Grass",
            Self::Water => "Water",
            Self::Cobblestone => "Cobblestone",
            Self::Metal => "Metal",
            Self::Ridged => "Ridged",
        }
    }
} 