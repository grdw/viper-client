use super::{Command, Helper};

const C0_PREFIX: [u8; 2] = [0xc0, 0x18];
const R0_PREFIX: [u8; 2] = [0x00, 0x18];
const R1_PREFIX: [u8; 2] = [0x20, 0x18];

// TODO: What is this for?
const PADDING: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

#[derive(Debug)]
pub struct CTPPChannel {
    control: [u8; 2],
    bitmask: Vec<u8>
}

impl CTPPChannel {
    pub fn new(control: &[u8; 2]) -> CTPPChannel {
        CTPPChannel {
            control: *control,
            bitmask: Helper::gen_ran(4)
        }
    }

    pub fn open(&self, sub: &String) -> Vec<u8> {
        Command::channel(
            &String::from("CTPP"),
            &self.control,
            Some(&sub.as_bytes()),
        )
    }

    pub fn close(&self) -> Vec<u8> {
        Command::close(&self.control)
    }

    // This is the initial call that's made right after
    // the CTPP and CSPB call
    pub fn connect_hs(&self, a1: &String, a2: &String) -> Vec<u8> {
        let q1 = [0x00, 0x11, 0x00, 0x70];
        let q2 = [0x00, 0x10, 0x0e, 0x00, 0x00, 0x00, 0x00];
        let q3 = Helper::gen_ran(2);

        let req = [
            &C0_PREFIX[..],
            &self.bitmask,
            &q1[..],
            &q3[..],
            &a2.as_bytes(),
            &q2[..]
        ].concat();

        return self.template(&req, &a2, &a1)
    }

    pub fn connect_actuators(&mut self,
                             prefix: u8,
                             a1: &String,
                             a2: &String) -> Vec<u8> {

        let pre = match prefix {
            0 => {
                self.tick_mask();
                &R0_PREFIX
            },
            1 => &R1_PREFIX,
            _ => panic!("Invalid prefix")
        };

        let req = [
            &pre[..],
            &self.bitmask[..],
            &[0x00, 0x00]
        ].concat();

        return self.template(&req, a1, a2)
    }

    pub fn link_actuators(&mut self,
                          a1: &String,
                          a2: &String) -> Vec<u8> {

        self.tick_mask();

        let q1 = [0x00, 0x28, 0x00, 0x01];
        let q2 = [0x00, 0x00, 0x01, 0x20];
        let q3 = Helper::gen_ran(4);
        let q4 = [0x00, 0x49, 0x49];

        let req = [
            &C0_PREFIX,
            &self.bitmask[..],
            &q1[..],
            &a1.as_bytes(),
            &[0x00],
            &a2.as_bytes(),
            &q2[..],
            &q3[..],
            &a1.as_bytes(),
            &q4[..]
        ].concat();

        return self.template(&req, a1, a2)
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

        let link = format!("{}\0{}", actuator, other_actuator);

        let req = [
            &prefix[..],
            &PADDING[..],
            &link.as_bytes(),
            &[0x00, 0x00]
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
        let ctpp = CTPPChannel::new(&[1, 2]);
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
        let ctpp = CTPPChannel::new(&[1, 2]);
        let conn = ctpp.open(&String::from("SB0000062"));

        assert_eq!(conn[2], 0x1e);
        assert_eq!(&conn[8..16],
                   &[0xcd, 0xab, 0x01, 0x00, 0x07, 0x00, 0x00, 0x00]);
        assert_eq!(str::from_utf8(&conn[16..20]).unwrap(), "CTPP");
        assert_eq!(str::from_utf8(&conn[28..37]).unwrap(), "SB0000062");
        assert_eq!(conn[37], 0x00);
    }

    #[test]
    fn test_connect_hs() {
        let ctpp = CTPPChannel::new(&[1, 2]);
        let conn = ctpp.connect_hs(
            &String::from("SB0000062"),
            &String::from("SB000006")
        );

        assert_eq!(&conn[2], &51);
        assert_eq!(&conn[8..10], &[192, 24])
    }

    #[test]
    fn test_connect_actuators() {
        let mut ctpp = CTPPChannel::new(&[1, 2]);
        let conn = ctpp.connect_actuators(
            0,
            &String::from("SB0000062"),
            &String::from("SB000006")
        );

        assert_eq!(&conn[2], &32);
        assert_eq!(&conn[8..10], &[0, 24]);

        let conn = ctpp.connect_actuators(
            1,
            &String::from("SB0000062"),
            &String::from("SB000006")
        );

        assert_eq!(&conn[2], &32);
        assert_eq!(&conn[8..10], &[32, 24])
    }

    #[test]
    fn test_link_actuators() {
        let mut ctpp = CTPPChannel::new(&[1, 2]);
        let conn = ctpp.link_actuators(
            &String::from("SB0000062"),
            &String::from("SB000006")
        );

        assert_eq!(&conn[2], &72);
        assert_eq!(&conn[8..10], &[192, 24])
    }
}
