use rand::distributions::{Distribution, Uniform};

pub struct Helper {}

impl Helper {
    pub fn gen_ran(size: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let die = Uniform::from(1..255);

        (0..size)
            .map(|_| die.sample(&mut rng))
            .collect::<Vec<u8>>()
    }

    pub fn control() -> [u8; 2] {
        let mut rng = rand::thread_rng();
        let die = Uniform::from(1..255);

        [
            die.sample(&mut rng),
            die.sample(&mut rng)
        ]
    }
}
