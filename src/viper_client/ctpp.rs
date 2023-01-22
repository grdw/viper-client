use super::Command;

#[derive(Debug)]
pub struct CTPP {
    control: [u8; 3]
}

impl CTPP {
    pub fn new(control: [u8; 3]) -> CTPP {
        CTPP { control: control }
    }
}
