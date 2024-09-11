/**
 * Bandwidth type
 */
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bandwidth {
    Bw20 = 20,
    Bw40 = 40,
    Bw80 = 80,
    Bw160 = 160,
}

impl Bandwidth {
    pub fn from_code(value: u8) -> Bandwidth {
        match value {
            0 => Bandwidth::Bw20,
            1 => Bandwidth::Bw40,
            2 => Bandwidth::Bw80,
            3 => Bandwidth::Bw160,
            _ => panic!("Invalid bandwidth value: {}", value),
        }
    }

    pub fn to_mhz(self) -> u32 {
        self as u32
    }

    pub fn to_hz(self) -> u32 {
        self.to_mhz() * 1_000_000
    }
}
