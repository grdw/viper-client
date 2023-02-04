use crate::command::Command;
use crate::helper::Helper;

// Every replaceable character in this template
// is marked as 0xFF not 0xff.
const HS_TEMPLATE: [u8; 28] = [
    0xc0, 0x18, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x11,
    0x00, 0x40, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x10, 0x0e,
    0x00, 0x00, 0x00, 0x00
];

const OPEN_DOOR_TEMPLATE: [u8; 48] = [
    0xc0, 0x18,
    0xFF, 0xFF, 0xFF, 0xFF,  // 4 random bytes
    0x00, 0x28, 0x00, 0x01,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Actuator 1
    0x00,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Actuator 2
    0x00, 0x00, 0x01, 0x20,
    0xFF, 0xFF, 0xFF, 0xFF,  // 4 other random bytes
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Actuator 1
    0x00, 0x49, 0x49
];

const ACK_TEMPLATE: [u8; 8] = [
    0xFF, 0x18, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00
];

const TAIL_TEMPLATE: [u8; 24] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00
];

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
    // the CTPP channel is opened
    // You have to read replies (max times is 2) until a response
    // is returned that starts with [0x60, 0x18]
    pub fn connect_hs(&self, a1: &String, a2: &String) -> Vec<u8> {
        let mut req = [
            &HS_TEMPLATE[..],
            &TAIL_TEMPLATE[..]
        ].concat();

        CTPPChannel::set_bytes(&mut req, &self.bitmask, 2);
        CTPPChannel::set_bytes(&mut req, &Helper::gen_ran(2), 10);
        CTPPChannel::set_bytes(&mut req, &a1.as_bytes(), 12);
        CTPPChannel::set_bytes(&mut req, &a1.as_bytes(), 32);
        CTPPChannel::set_bytes(&mut req, &a2.as_bytes(), 42);

        Command::make(&req, &self.control)
    }

    pub fn confirm_handshake(&self, r: &[u8]) -> bool {
        r[0] == 0x60 &&
            self.bitmask[0] + 0x80 == r[2] &&
            self.bitmask[1] == r[3] &&
            self.bitmask[2] == r[5] - 1 &&
            self.bitmask[3] == r[4]
    }

    pub fn ack(&mut self,
               prefix: u8,
               a1: &String,
               a2: &String) -> Vec<u8> {

        let mut req = [
            &ACK_TEMPLATE[..],
            &TAIL_TEMPLATE[..]
        ].concat();

        if prefix == 0x00 {
            self.tick_mask();
        }

        CTPPChannel::set_bytes(&mut req, &[prefix], 0);
        CTPPChannel::set_bytes(&mut req, &self.bitmask, 2);
        CTPPChannel::set_bytes(&mut req, &a1.as_bytes(), 12);
        CTPPChannel::set_bytes(&mut req, &a2.as_bytes(), 22);

        return Command::make(&req, &self.control)
    }

    pub fn link_actuators(&mut self,
                          a1: &String,
                          a2: &String) -> Vec<u8> {

        let mut req = [
            &OPEN_DOOR_TEMPLATE[..],
            &TAIL_TEMPLATE[..]
        ].concat();

        self.bitmask = Helper::gen_ran(4);

        CTPPChannel::set_bytes(&mut req, &self.bitmask, 2);
        CTPPChannel::set_bytes(&mut req, &a1.as_bytes(), 10);
        CTPPChannel::set_bytes(&mut req, &a2.as_bytes(), 20);
        CTPPChannel::set_bytes(&mut req, &Helper::gen_ran(4), 32);
        CTPPChannel::set_bytes(&mut req, &a1.as_bytes(), 36);
        CTPPChannel::set_bytes(&mut req, &a1.as_bytes(), 52);
        CTPPChannel::set_bytes(&mut req, &a2.as_bytes(), 62);

        return Command::make(&req, &self.control)
    }

    fn tick_mask(&mut self) {
        self.bitmask[3] += 1;
    }

    fn set_bytes(template: &mut [u8], bytes: &[u8], offset: usize) {
        for (i, b) in bytes.iter().enumerate() {
            template[i + offset] = *b;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

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

        assert_eq!(&conn[2], &52);
        assert_eq!(&conn[8..10], &[192, 24]);
        assert_eq!(str::from_utf8(&conn[20..29]).unwrap(), "SB0000062");
        assert_eq!(str::from_utf8(&conn[40..49]).unwrap(), "SB0000062");
        assert_eq!(&conn[49], &0x00);
        assert_eq!(str::from_utf8(&conn[50..58]).unwrap(), "SB000006");
        assert_eq!(&conn[58..], &[0x00, 0x00]);
    }

    #[test]
    fn test_ack() {
        let mut ctpp = CTPPChannel::new(&[1, 2]);
        let mask = ctpp.bitmask[3];
        let conn = ctpp.ack(
            0x00,
            &String::from("SB0000062"),
            &String::from("SB000006")
        );

        assert_eq!(conn[13], mask + 1);
        assert_eq!(&conn[2], &32);
        assert_eq!(&conn[8..10], &[0, 24]);
        assert_eq!(str::from_utf8(&conn[20..29]).unwrap(), "SB0000062");
        assert_eq!(str::from_utf8(&conn[30..38]).unwrap(), "SB000006");

        let conn_2 = ctpp.ack(
            0x20,
            &String::from("SB0000062"),
            &String::from("SB000006")
        );

        assert_eq!(&conn_2[2], &32);
        assert_eq!(&conn_2[8..10], &[32, 24]);
        assert_eq!(str::from_utf8(&conn_2[20..29]).unwrap(), "SB0000062");
        assert_eq!(str::from_utf8(&conn_2[30..38]).unwrap(), "SB000006");
        // This is to test that the bitmask doesn't swap between
        // 0x00 and 0x20 calls
        assert_eq!(&conn_2[10..14], &conn[10..14]);
    }

    #[test]
    fn test_confirm_handshake() {
        let ctpp = CTPPChannel {
            control: [1, 2],
            bitmask: vec![0x42, 0x70, 0x2f, 0x50]
        };

        assert_eq!(
            ctpp.confirm_handshake(
                &[0x00, 0x18, 0xc2, 0x70, 0x50, 0x30]
            ),
            false
        );

        assert_eq!(
            ctpp.confirm_handshake(
                &[0x60, 0x18, 0xc2, 0x70, 0x50, 0x30]
            ),
            true
        );

        assert_eq!(
            ctpp.confirm_handshake(
                &[0x60, 0x18, 0xc1, 0x70, 0x50, 0x30]
            ),
            false
        );
    }

    #[test]
    fn test_link_actuators() {
        let mut ctpp = CTPPChannel::new(&[1, 2]);
        let conn = ctpp.link_actuators(
            &String::from("SB0000062"),
            &String::from("SB000006")
        );

        assert_eq!(&conn[2], &72);
        assert_eq!(&conn[8..10], &[192, 24]);
        assert_eq!(str::from_utf8(&conn[18..27]).unwrap(), "SB0000062");
        assert_eq!(&conn[27], &0);
        assert_eq!(str::from_utf8(&conn[28..36]).unwrap(), "SB000006");
        assert_eq!(str::from_utf8(&conn[44..53]).unwrap(), "SB0000062");
        assert_eq!(str::from_utf8(&conn[60..69]).unwrap(), "SB0000062");
        assert_eq!(&conn[69], &0);
        assert_eq!(str::from_utf8(&conn[70..78]).unwrap(), "SB000006");
    }
}
