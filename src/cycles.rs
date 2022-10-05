use std::marker::PhantomData;
use std::fmt;

pub trait Cycle<T> {
    type Item : Eq + Clone + fmt::Debug;
    fn new(width: T, height: Self::Item) -> Self;
    fn next(&mut self) -> Vec<Self::Item>;
    fn peek(&self) -> Vec<Self::Item>;
    fn root(&self) -> Vec<Self::Item>;
    fn reset(&mut self);
}


#[derive(Debug)]
pub struct ICycle<T: Copy + num::Integer> {
    height: T,
    state: Vec<T>
}


impl<T: Copy + num::Integer + fmt::Debug> Cycle<usize> for ICycle<T> {
    type Item = T;

    fn new(width : usize, height : T) -> ICycle<T> {
        ICycle{height, state: vec![T::zero(); width]}
    }

    fn next(&mut self) -> Vec<T> {
        let out = self.state.clone();
        for i in (0..self.state.len()).rev() {
            if self.state[i] == self.height - T::one() {
                self.state[i] = T::zero()
            } else {
                self.state[i] = self.state[i] + T::one();
                break;
            }
        }
        return out;
    }

    fn peek(&self) -> Vec<T> {
        self.state.clone()
    }

    fn root(&self) -> Vec<T> {
        vec![T::zero(); self.state.len()]
    }

    fn reset(&mut self) {
        for i in 0..self.state.len() {
            self.state[i] = T::zero();
        }
    }
}


#[derive(Debug)]
pub struct VCycle<S, T: Cycle<S>> {
    phantom: PhantomData<S>,
    state: Vec<T>,
}


impl<S: Clone, T: Cycle<S> + fmt::Debug> Cycle<Vec<S>> for VCycle<S, T> {

    type Item = Vec<T::Item>;

    fn new(width: Vec<S>, height: Self::Item) -> Self {
        let mut state = Vec::new();
        for i in 0..width.len() {
            state.push(T::new(width[i].clone(), height[i].clone()));
        }
        VCycle{
            state,
            phantom: PhantomData
        }
    }

    fn next(&mut self) -> Vec<Self::Item> {
        let out = self.peek();
        for i in (0..self.state.len()).rev() {
            self.state[i].next();
            let mut done = false;
            for (a, b) in self.state[i].peek().iter().zip(self.state[i].root()) {
                if *a != b {
                    done = true;
                    break;
                }
            }
            if done {
                break;
            }
        }
        return out;
    }

    fn peek(&self) -> Vec<Self::Item> {
        self.state.iter().map(|x| x.peek().clone()).collect()
    }

    fn root(&self) -> Vec<Self::Item> {
        self.state.iter().map(|x| x.root()).collect()
    }

    fn reset(&mut self) {
        for i in 0..self.state.len() {
            self.state[i].reset();
        }
    }
}


#[derive(Debug)]
pub struct AllWords<S, T: Cycle<S>> {
    internal: T,
    phantom: PhantomData<S>,
    done: bool
}


impl<S, T: Cycle<S>> AllWords<S, T> {
    pub fn new(width: S, height: T::Item) -> AllWords<S, T> {
        AllWords{
            internal: T::new(width, height),
            phantom: PhantomData,
            done: false
        }
    }

    /* fn reset(&mut self) {
        self.internal.reset();
        self.done = false;
    } */
}


impl<S, T: Cycle<S>> Iterator for AllWords<S, T> {
    type Item = Vec<T::Item>;

    fn next(&mut self) -> Option<Vec<T::Item>> {
        if self.done {
            return None;
        }
        let next = self.internal.next();
        self.done = true;
        for (a, b) in self.internal.peek().iter().zip(&self.internal.root()) {
            if a != b {
                self.done = false;
                break;
            }
        }
        return Some(next);
    }
}


#[derive(Debug)]
pub struct Combination<S, T: Cycle<S>, U, V: Cycle<U>> {
    internal: (T, V),
    phantom_l: PhantomData<S>,
    phantom_r: PhantomData<U>,
    done: bool
}


impl<S, T:Cycle<S>, U, V:Cycle<U>> Combination<S, T, U, V> {
    pub fn new(width: (S, U), height: (T::Item, V::Item)) -> Combination<S, T, U, V> {
        Combination{
            internal: (T::new(width.0, height.0),
                       V::new(width.1, height.1)),
            phantom_l: PhantomData,
            phantom_r: PhantomData,
            done: false
        }
    }
}


impl<S, T:Cycle<S>, U, V:Cycle<U>> Iterator for Combination<S, T, U, V> {
    type Item = (Vec<T::Item>, Vec<V::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let out = (self.internal.0.peek(),
                   self.internal.1.peek());
        self.internal.1.next();
        if self.internal.1.peek() == self.internal.1.root() {
            self.internal.0.next();
            if self.internal.0.peek() == self.internal.0.root() {
                self.done = true
            }
        }
        return Some(out);
    }
}
