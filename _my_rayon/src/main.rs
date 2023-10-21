use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaChaRng;

fn main() {
    let mut rand = ChaChaRng::from_entropy();
    let rand_num = &rand.gen_range(1..7);

    println!("{}", rand_num);
}
