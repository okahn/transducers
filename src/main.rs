pub mod transducers;
pub mod cycles;
use transducers::AllTransducers;

fn main() {
    transducers::example();
    for (i, m) in AllTransducers::new(3).enumerate() {
        if i % 10_000 == 0 {
            println!("{} {:?}", i, m);
        }
    }
}

