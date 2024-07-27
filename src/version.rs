#[derive(Clone, Copy, Debug)]
#[derive(PartialEq)]
pub enum DWGVersion {
    AC1012, // R13
    AC1014, // R14
    AC1015, // R2000
    AC1018, // R2004
    AC1021, // R2007
    AC1027, // R2013
    AC1032, // R2018
}

impl DWGVersion {
    pub fn from_magic(magic: &[u8; 6]) -> Option<Self> {
        match magic {
            b"AC1012" => Some(Self::AC1012), 
            b"AC1014" => Some(Self::AC1014), 
            b"AC1015" => Some(Self::AC1015), 
            b"AC1018" => Some(Self::AC1018), 
            b"AC1021" => Some(Self::AC1021), 
            b"AC1027" => Some(Self::AC1027), 
            b"AC1032" => Some(Self::AC1032), 
            _ => None,
        }
    }
}


#[test]
fn test_from_magic() {
    assert_eq!(DWGVersion::from_magic(b"AC1012"), Some(DWGVersion::AC1012));
}
