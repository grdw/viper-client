use super::Command;
use rand::Rng;
use rand::distributions::{Distribution, Uniform};

const R2_PREFIX: [u8; 2] = [64, 24];
const R2_SUFFIX: [u8; 6] = [0, 3, 0, 14, 0, 0];
const R3_SUFFIX: [u8; 10] = [0, 8, 0, 3, 73, 0, 39, 0, 0, 0];

#[derive(Debug)]
pub struct CTPP {
    control: [u8; 3],
    bitmask: Vec<u8>,
    apt: String,
    sub: String
}

impl CTPP {
    pub fn new(control: [u8; 3], apt: String, sub: String) -> CTPP {
        CTPP {
            control: control,
            apt: apt,
            sub: sub,
            bitmask: CTPP::generate_mask(4)
        }
    }

    fn generate_mask(size: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let die = Uniform::from(1..255);

        (0..size)
            .map(|_| die.sample(&mut rng))
            .collect::<Vec<u8>>()
    }

    // This is the initial call that's made right after the CTPP and CSPB
    // call
    // Question: do I need CSPB?
    pub fn connect(&self) -> Vec<u8> {
        let prefix = [192, 24];
        let suffix = [0, 16, 14, 0, 0, 0, 0];

        let req = [
            &prefix[..],
            &self.bitmask,
            &[0, 17, 0, 64],
            &CTPP::generate_mask(3),
            &self.sub.as_bytes(),
            &suffix[..]
        ].concat();

        return self.template(&req, &self.sub, &self.apt)
    }

    //fn r2(&mut self, actuator: &String) -> Vec<u8> {
    //    let mask = CTPP::generate_mask(4);

    //    let req = [
    //        &R2_PREFIX[..],
    //        &mask[..],
    //        &R2_SUFFIX[..]
    //    ].concat();

    //    return self.template(&req, &self.sub, actuator)
    //}

    //fn r3(&mut self,
    //      actuator: &String,
    //      other_actuator: &String) -> Vec<u8> {

    //    let mask = CTPP::generate_mask(4);

    //    let req = [
    //        &R2_PREFIX[..],
    //        &mask[..],
    //        &R3_SUFFIX[..]
    //    ].concat();

    //    return self.template(&req, actuator, other_actuator)
    //}

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
        let ctpp = CTPP::new(
            [1, 2, 3],
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
    fn test_connect() {
        let mut ctpp = CTPP::new(
            [1, 2, 3],
            String::from("SB0000062"),
            String::from("SB000006")
        );
        let conn = ctpp.connect();

        assert_eq!(conn.len(), 60)
    }
}
