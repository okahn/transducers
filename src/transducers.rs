use super::cycles::Combination;
use super::cycles::ICycle;
use super::cycles::VCycle;

pub fn example() {
    let transition : Vec<Vec<usize>> = vec![vec![1,0], vec![1,1]];
    let flip : Vec<u8> = vec![1, 0];
    let example = Transducer {transition, flip};
    println!("step {:?}", example.step(vec![1, 0, 1, 0, 1]));
}

impl Transducer {

    fn step(&self, x : Vec<u8>) -> Vec<u8> {
        let mut state: usize = 0;
        let mut out: Vec<u8> = Vec::new();
        for c in x {
            out.push(c ^ self.flip[state]);
            state = self.transition[state][c as usize];
        }
        return out;
    }

    /* fn orbit_compare(&self, other: &Transducer, depth: u64) -> bool {
        panic!();
    } */

}

impl Clone for Transducer {
    fn clone(&self) -> Transducer {
        Transducer {
            transition: self.transition.clone(),
            flip: self.flip.clone()
        }
    }
}

pub struct AllTransducers {
    state: Combination::<Vec<usize>, VCycle<usize, ICycle<usize>>, usize, ICycle<u8>>,
}

impl AllTransducers {
    pub fn new(size: usize) -> AllTransducers {
        AllTransducers{
            state : Combination::new(
                (vec![2; size], size),
                (vec![size; size], 2)
            )
        }
    }
}

impl Iterator for AllTransducers {
    type Item = Transducer;

    fn next(&mut self) -> Option<Transducer> {
        match self.state.next() {
            None => None,
            Some(x) => Some(Transducer{transition: x.0, flip: x.1})
        }
    }
}

#[derive(Debug)]
pub struct Transducer {
    transition: Vec<Vec<usize>>,
    flip: Vec<u8>
}
