use super::Command;
use rand::Rng;
use rand::distributions::{Distribution, Uniform};

const R2_PREFIX: [u8; 2] = [64, 24];
const R2_SUFFIX: [u8; 6] = [0, 3, 0, 14, 0, 0];
const R3_SUFFIX: [u8; 10] = [0, 8, 0, 3, 73, 0, 39, 0, 0, 0];

#[derive(Debug)]
pub struct CTPP {
    control: [u8; 3],
    bitmask: Vec<u8>
}

impl CTPP {
    pub fn new(control: [u8; 3]) -> CTPP {
        CTPP {
            control: control,
            bitmask: CTPP::generate_mask()
        }
    }

    fn generate_mask() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let die = Uniform::from(1..255);

        (0..4)
            .map(|_| die.sample(&mut rng))
            .collect::<Vec<u8>>()
    }

    fn r1(&self,
          actuator: &String,
          other_actuator: &String) -> Vec<u8> {

        let prefix = [192, 24];
        let suffix = [0, 16, 14, 0, 0];

        let req = [
            &prefix[..],
            &self.bitmask,
            &actuator.as_bytes(),
            &suffix[..]
        ].concat();

        return self.template(&req, actuator, other_actuator)
    }

    fn r2(&mut self,
          actuator: &String,
          other_actuator: &String) -> Vec<u8> {

        let mask = CTPP::generate_mask();

        let req = [
            &R2_PREFIX[..],
            &mask[..],
            &R2_SUFFIX[..]
        ].concat();

        return self.template(&req, actuator, other_actuator)
    }

    fn r3(&mut self,
          actuator: &String,
          other_actuator: &String) -> Vec<u8> {

        let mask = CTPP::generate_mask();

        let req = [
            &R2_PREFIX[..],
            &mask[..],
            &R3_SUFFIX[..]
        ].concat();

        return self.template(&req, actuator, other_actuator)
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
        let ctpp = CTPP::new([1, 2, 3]);
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
    fn test_r2() {
        let mut ctpp = CTPP::new([1, 2, 3]);
        let r2 = ctpp.r2(
            &String::from("SB0000062"),
            &String::from("SB000006")
        );

        assert_eq!(r2.len(), 44)
    }
}
