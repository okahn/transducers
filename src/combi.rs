use num::Integer;
use num::Unsigned;

/// An iterator representing a cycle.
pub trait Cycle<T: Clone>: Iterator<Item = T> {
    /// Creates a cycle.
    ///
    /// `max` represents the kind of structure to be cycled through.
    /// If `limit` is true, the iteration will end after one complete cycle rather than repeat.
    fn new(max: T, limit: bool) -> Self;
}

/// An iterator cycling through integers modulo `max`.
pub struct UCycle<T: Unsigned + Integer + Copy> {
    max: T,
    state: T,
    limit: bool,
    done: bool,
}

impl<T: Unsigned + Integer + Copy> Cycle<T> for UCycle<T> {
    fn new(max: T, limit: bool) -> Self {
        UCycle {
            max,
            state: T::zero(),
            limit,
            done: false,
        }
    }
}

impl<T: Unsigned + Integer + Copy> Iterator for UCycle<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.done {
            return None;
        }
        let out = self.state;
        self.state = self.state + T::one();
        if self.state == self.max {
            self.state = T::zero();
            self.done = self.limit;
        }
        return Some(out);
    }
}

/// An iterator cycling through the chained Cartesian product of identically-typed cycles,
/// represented as a vector.
pub struct VCycle<U: Clone, T: Cycle<U>> {
    max: Vec<U>,
    state: Vec<T>,
    prev: Vec<U>,
    limit: bool,
    done: bool,
}

impl<U: Clone, T: Cycle<U>> Cycle<Vec<U>> for VCycle<U, T> {
    fn new(max: Vec<U>, limit: bool) -> Self {
        let mut state: Vec<T> = max.iter().map(|x| T::new(x.clone(), true)).collect();
        let mut prev: Vec<U> = Vec::new();
        for i in 0..state.len() {
            prev.push(state[i].next().unwrap().clone());
        }
        return VCycle {
            max,
            state,
            prev,
            limit,
            done: false,
        };
    }
}

impl<U: Clone, T: Cycle<U>> Iterator for VCycle<U, T> {
    type Item = Vec<U>;

    fn next(&mut self) -> Option<Vec<U>> {
        if self.done {
            return None;
        } else {
            let out = self.prev.clone();
            let mut overflow = true;
            for i in (0..self.state.len()).rev() {
                match self.state[i].next() {
                    Some(x) => {
                        overflow = false;
                        self.prev[i] = x.clone();
                        break;
                    }
                    None => {
                        self.state[i] = T::new(self.max[i].clone(), true);
                        self.prev[i] = self.state[i].next().unwrap();
                    }
                }
            }
            self.done = overflow && self.limit;
            return Some(out);
        }
    }
}

/// An iterator cycling through the Cartesian product of two cycles,
/// represented as a tuple.
pub struct TCycle<U: Clone, V: Clone, S: Cycle<U>, T: Cycle<V>> {
    max: (U, V),
    state: (S, T),
    prev: (U, V),
    limit: bool,
    done: bool,
}

impl<U: Clone, V: Clone, S: Cycle<U>, T: Cycle<V>> Cycle<(U, V)> for TCycle<U, V, S, T> {
    fn new(max: (U, V), limit: bool) -> Self {
        let mut state: (S, T) = (S::new(max.0.clone(), true), T::new(max.1.clone(), true));
        let prev: (U, V) = (
            state.0.next().unwrap().clone(),
            state.1.next().unwrap().clone(),
        );
        return TCycle {
            max,
            state,
            prev,
            limit,
            done: false,
        };
    }
}

impl<U: Clone, V: Clone, S: Cycle<U>, T: Cycle<V>> Iterator for TCycle<U, V, S, T> {
    type Item = (U, V);

    fn next(&mut self) -> Option<(U, V)> {
        if self.done {
            return None;
        } else {
            let out = self.prev.clone();
            let mut overflow = true;
            match self.state.1.next() {
                Some(x) => {
                    overflow = false;
                    self.prev.1 = x.clone();
                }
                None => {
                    self.state.1 = T::new(self.max.1.clone(), true);
                    self.prev.1 = self.state.1.next().unwrap();
                    match self.state.0.next() {
                        Some(x) => {
                            overflow = false;
                            self.prev.0 = x.clone();
                        }
                        None => {
                            self.state.0 = S::new(self.max.0.clone(), true);
                            self.prev.0 = self.state.0.next().unwrap();
                        }
                    }
                }
            }
            self.done = overflow && self.limit;
            return Some(out);
        }
    }
}

/// An iterator through all permutations of the numbers from 0 to `width-1` in
/// shortlex order.
pub struct Permutation {
    state: usize,
    width: usize,
}

fn factorial(n: usize) -> usize {
    let mut res = 1;
    for i in 2..n {
        res *= i;
    }
    return res;
}

impl Permutation {
    /// Creates a cycle through permutations of numbers from 0 to `width-1`.
    pub fn new(width: usize) -> Self {
        Permutation { state: 0, width }
    }
}

impl Iterator for Permutation {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state >= factorial(self.width + 1) {
            return None;
        }
        let mut pool: Vec<usize> = (0..self.width).collect();
        let mut res: Vec<usize> = Vec::new();
        let mut index = self.state;
        for i in (1..self.width + 1).rev() {
            res.push(pool.remove(index / factorial(i)));
            index %= factorial(i);
        }
        self.state += 1;
        return Some(res);
    }
}
