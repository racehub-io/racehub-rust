#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Buttons(pub u32);

impl Buttons {
    // PlayStation / Xbox buttons
    pub const CROSS_OR_A: u32 = 0x00000001;
    pub const TRIANGLE_OR_Y: u32 = 0x00000002;
    pub const CIRCLE_OR_B: u32 = 0x00000004;
    pub const SQUARE_OR_X: u32 = 0x00000008;

    // D-pad
    pub const DPAD_LEFT: u32 = 0x00000010;
    pub const DPAD_RIGHT: u32 = 0x00000020;
    pub const DPAD_UP: u32 = 0x00000040;
    pub const DPAD_DOWN: u32 = 0x00000080;

    // Menu
    pub const OPTIONS_OR_MENU: u32 = 0x00000100;

    // Shoulder buttons
    pub const L1_OR_LB: u32 = 0x00000200;
    pub const R1_OR_RB: u32 = 0x00000400;
    pub const L2_OR_LT: u32 = 0x00000800;
    pub const R2_OR_RT: u32 = 0x00001000;

    // Stick buttons
    pub const LEFT_STICK_CLICK: u32 = 0x00002000;
    pub const RIGHT_STICK_CLICK: u32 = 0x00004000;

    // Right stick directions
    pub const RIGHT_STICK_LEFT: u32 = 0x00008000;
    pub const RIGHT_STICK_RIGHT: u32 = 0x00010000;
    pub const RIGHT_STICK_UP: u32 = 0x00020000;
    pub const RIGHT_STICK_DOWN: u32 = 0x00040000;

    // Special
    pub const SPECIAL: u32 = 0x00080000;

    // UDP Actions
    pub const UDP_ACTION_1: u32 = 0x00100000;
    pub const UDP_ACTION_2: u32 = 0x00200000;
    pub const UDP_ACTION_3: u32 = 0x00400000;
    pub const UDP_ACTION_4: u32 = 0x00800000;
    pub const UDP_ACTION_5: u32 = 0x01000000;
    pub const UDP_ACTION_6: u32 = 0x02000000;
    pub const UDP_ACTION_7: u32 = 0x04000000;
    pub const UDP_ACTION_8: u32 = 0x08000000;
    pub const UDP_ACTION_9: u32 = 0x10000000;
    pub const UDP_ACTION_10: u32 = 0x20000000;
    pub const UDP_ACTION_11: u32 = 0x40000000;
    pub const UDP_ACTION_12: u32 = 0x80000000;

    pub fn new(flags: u32) -> Self {
        Self(flags)
    }

    pub fn contains(&self, flag: u32) -> bool {
        (self.0 & flag) == flag
    }

    pub fn name(flag: u32) -> &'static str {
        match flag {
            Self::CROSS_OR_A => "Cross or A",
            Self::TRIANGLE_OR_Y => "Triangle or Y",
            Self::CIRCLE_OR_B => "Circle or B",
            Self::SQUARE_OR_X => "Square or X",
            Self::DPAD_LEFT => "D-pad Left",
            Self::DPAD_RIGHT => "D-pad Right",
            Self::DPAD_UP => "D-pad Up",
            Self::DPAD_DOWN => "D-pad Down",
            Self::OPTIONS_OR_MENU => "Options or Menu",
            Self::L1_OR_LB => "L1 or LB",
            Self::R1_OR_RB => "R1 or RB",
            Self::L2_OR_LT => "L2 or LT",
            Self::R2_OR_RT => "R2 or RT",
            Self::LEFT_STICK_CLICK => "Left Stick Click",
            Self::RIGHT_STICK_CLICK => "Right Stick Click",
            Self::RIGHT_STICK_LEFT => "Right Stick Left",
            Self::RIGHT_STICK_RIGHT => "Right Stick Right",
            Self::RIGHT_STICK_UP => "Right Stick Up",
            Self::RIGHT_STICK_DOWN => "Right Stick Down",
            Self::SPECIAL => "Special",
            Self::UDP_ACTION_1 => "UDP Action 1",
            Self::UDP_ACTION_2 => "UDP Action 2",
            Self::UDP_ACTION_3 => "UDP Action 3",
            Self::UDP_ACTION_4 => "UDP Action 4",
            Self::UDP_ACTION_5 => "UDP Action 5",
            Self::UDP_ACTION_6 => "UDP Action 6",
            Self::UDP_ACTION_7 => "UDP Action 7",
            Self::UDP_ACTION_8 => "UDP Action 8",
            Self::UDP_ACTION_9 => "UDP Action 9",
            Self::UDP_ACTION_10 => "UDP Action 10",
            Self::UDP_ACTION_11 => "UDP Action 11",
            Self::UDP_ACTION_12 => "UDP Action 12",
            _ => "Unknown Button",
        }
    }

    pub fn active_buttons(&self) -> Vec<u32> {
        let mut buttons = Vec::new();
        let mut flag: u32 = 1;
        
        while flag != 0 {
            if self.contains(flag) {
                buttons.push(flag);
            }
            flag = flag.checked_shl(1).unwrap_or(0);
        }
        
        buttons
    }
} 