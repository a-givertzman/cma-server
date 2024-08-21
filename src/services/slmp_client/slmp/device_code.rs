///
/// Classifies a Device Code for SLMP
#[repr(u8)] // must be regarding SLMP DOCs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceCode {
    Sm = 0x91,     // Special relay (SM)
    Sd = 0xA9,     // Special register (SD)
    X  = 0x9C,     // Input (X)
    Y  = 0x9D,     // Output (Y)
    M  = 0x90,     // Internal relay (M)
    L  = 0x92,     // Latch relay (L)
    F  = 0x93,     // Annunciator (F)
    V  = 0x94,     // Edge relay (V) 
    B  = 0xA0,     // Link relay (B)
    D  = 0xA8,     // Data register (D)
    W  = 0xB4,     // Link register (W) 
}
//
//
impl From<&str> for DeviceCode {
    fn from(value: &str) -> Self {
        let value = value.to_lowercase();
        match value.as_str() {
            "sm" => Self::Sm,
            "sd" => Self::Sd,
            "x"  => Self::X,
            "y"  => Self::Y,
            "m"  => Self::M,
            "l"  => Self::L,
            "f"  => Self::F,
            "v"  => Self::V,
            "b"  => Self::B,
            "d"  => Self::D,
            "w"  => Self::W,
            _    => panic!("SlmpDeviceCode.from | Uncnown SLMP Device Code '{}'", value),
        }
    }
}
