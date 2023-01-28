use super::{Command, Helper};
use rand::distributions::{Distribution, Uniform};

const C0_PREFIX: [u8; 2] = [192, 24];
const R0_PREFIX: [u8; 2] = [0, 24];
const R1_PREFIX: [u8; 2] = [32, 24];

#[derive(Debug)]
pub struct CTPPChannel {
    control: [u8; 2],
    bitmask: Vec<u8>,
    apt: String,
    sub: String
}

impl CTPPChannel {
    pub fn new(control: &[u8; 2], apt: String, sub: String) -> CTPPChannel {
        CTPPChannel {
            control: *control,
            apt: apt,
            sub: sub,
            bitmask: Helper::gen_ran(4)
        }
    }

    pub fn open(&self) -> Vec<u8> {
        Command::channel(
            &String::from("CTPP"),
            &self.control,
            Some(&self.sub.as_bytes()),
        )
    }

    // This is the initial call that's made right after
    // the CTPP and CSPB call
    pub fn connect_hs(&self) -> Vec<u8> {
        let suffix = [0, 16, 14, 0, 0, 0, 0];

        let req = [
            &C0_PREFIX[..],
            &self.bitmask,
            &[0, 17, 0, 64],
            &Helper::gen_ran(3),
            &self.sub.as_bytes(),
            &suffix[..]
        ].concat();

        return self.template(&req, &self.sub, &self.apt)
    }

    pub fn connect_reply(&mut self) -> Vec<u8> {
        // IMPORTANT
        self.tick_mask();

        let req = [
            &R0_PREFIX[..],
            &self.bitmask[..],
            &[0, 0]
        ].concat();

        return self.template(&req, &self.sub, &self.apt)
    }

    pub fn connect_second_reply(&mut self) -> Vec<u8> {
        let req = [
            &R1_PREFIX[..],
            &self.bitmask[..],
            &[0, 0]
        ].concat();

        return self.template(&req, &self.sub, &self.apt)
    }

    fn tick_mask(&mut self) {
        self.bitmask[1] += 1;
        self.bitmask[2] += 1;
    }

    // All the CTPP requests follow the same template:
    fn template(&self,
                prefix: &[u8],
                actuator: &String,
                other_actuator: &String) -> Vec<u8> {

        let stuff = [255, 255, 255, 255];
        let link = format!("{}\0{}\0\0", actuator, other_actuator);

        let req = [
            &prefix[..],
            &stuff[..],
            &link.as_bytes()
        ].concat();

        Command::make(&req, &self.control)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn test_template() {
        let ctpp = CTPPChannel::new(
            &[1, 2],
            String::from("SB0000062"),
            String::from("SB000006")
        );
        let t = ctpp.template(
            &[0, 0, 0, 0, 0, 0, 0, 0],
            &String::from("SB0000062"),
            &String::from("SB000006")
        );

        assert_eq!(&t[16..20], &[255, 255, 255, 255]);
        assert_eq!(str::from_utf8(&t[20..29]).unwrap(), "SB0000062");
        assert_eq!(str::from_utf8(&t[30..38]).unwrap(), "SB000006");
        assert_eq!(&t[29], &0);
        assert_eq!(&t[38..40], &[0, 0]);
    }

    #[test]
    fn test_connect_open() {
        let ctpp = CTPPChannel::new(
            &[1, 2],
            String::from("SB0000062"),
            String::from("SB000006")
        );
        let conn = ctpp.open();

        assert_eq!(conn.len(), 37)
    }

    #[test]
    fn test_connect_hs() {
        let ctpp = CTPPChannel::new(
            &[1, 2],
            String::from("SB0000062"),
            String::from("SB000006")
        );
        let conn = ctpp.connect_hs();

        assert_eq!(conn.len(), 60);
        assert_eq!(&conn[8..10], &[192, 24])
    }
}
