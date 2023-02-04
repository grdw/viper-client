use rand::distributions::{Distribution, Uniform};

const START: u8 = 0x01;

// The reason why it stops at 0x80 is because in some parts of the
// protocol the server adds 0x80 to the total, which could result in
// a number higher than 0xff, which doesn't exist.
const END: u8 = 0x80;

pub struct Helper {}

impl Helper {
    pub fn gen_ran(size: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let die = Uniform::from(START..END);

        (0..size)
            .map(|_| die.sample(&mut rng))
            .collect::<Vec<u8>>()
    }

    pub fn control() -> [u8; 2] {
        let mut rng = rand::thread_rng();
        let die = Uniform::from(START..END);

        [
            die.sample(&mut rng),
            die.sample(&mut rng)
        ]
    }
}
