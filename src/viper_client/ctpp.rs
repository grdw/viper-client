use super::Command;

#[derive(Debug)]
pub struct CTPP {
    control: [u8; 3]
}

impl CTPP {
    pub fn new(control: [u8; 3]) -> CTPP {
        CTPP {
            control: control
        }
    }

    fn r1(&self, actuator: &String, other_actuator: &String) -> Vec<u8> {
        let start = [0, 24];
        // TODO: What are these 4 bytes
        // And why do they change every once in a while, it happens
        // at r3() and r4()
        let four = [0, 0, 0, 0];
        // TODO: Also why do I stuff like such. Why 4x 255?
        let stuff = [0, 0, 255, 255, 255, 255];
        let link = format!("{}\0{}", actuator, other_actuator);

        let req = [
            &start[..],
            &four[..],
            &stuff[..],
            &link.as_bytes()
        ].concat();

        Command::make(&req, &self.control)
    }

    fn r2(&self) {
        let start = [64, 24];
    }

    fn r3(&self) {
        let start = [96, 24];
    }

    fn r4(&self) {
        let start = [192, 24];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn test_r1() {
        let ctpp = CTPP { control: [1, 2, 3] };
        let r1 = ctpp.r1(
            &String::from("SB0000062"),
            &String::from("SB000006")
        );

        assert_eq!(str::from_utf8(&r1[20..29]).unwrap(), "SB0000062");
        assert_eq!(str::from_utf8(&r1[30..38]).unwrap(), "SB000006");
    }
}
