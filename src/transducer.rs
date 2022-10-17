use super::cycles::Cycle;
use super::cycles::Permutation;
use super::cycles::TCycle;
use super::cycles::UCycle;
use super::cycles::VCycle;
use super::dfa::DFA;
use core::hash::Hash;
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

/// A transducer. By convention, state `0` is the start state.
///
/// Warning: the representation only assumes the transducer is deterministic
/// and alphabetic, but most methods also assume it's reversible.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash, Ord)]
pub struct Transducer {
    transition: Vec<Vec<usize>>,
    flip: Vec<u8>,
}

impl Transducer {
    fn step(&self, x: &mut Vec<u8>) {
        let mut state: usize = 0;
        for i in 0..x.len() {
            let c = x[i];
            x[i] = c ^ self.flip[state];
            state = self.transition[state][c as usize];
        }
    }

    fn min_word(&self, word: &[u8]) -> Vec<u8> {
        let mut min = word.to_vec();
        let mut next = word.to_vec();
        self.step(&mut next);
        loop {
            if next < min {
                min = next.clone();
            }
            if next == word {
                return min;
            }
            self.step(&mut next);
        }
    }

    /// Compare the orbits of two transducers on strings up to length `depth`.
    ///
    /// At least O(2^n) time where n is `depth`, but memory-efficient.
    pub fn orbit_compare(&self, other: &Transducer, depth: usize) -> bool {
        let xs = VCycle::<u8, UCycle<u8>>::new(vec![2; depth], true);
        return xs
            .par_bridge()
            .all(|word| self.min_word(&word) == other.min_word(&word));
    }

    /// Minimize a given transducer.
    ///
    /// This uses the DFA representation of a transducer. DFAs minimization is
    /// well-known and the injection from transducers to DFAs preserves
    /// equivalence.
    pub fn minimize(&self) -> Self {
        let alphabet = TCycle::<u8, u8, UCycle<u8>, UCycle<u8>>::new((2, 2), true).collect();
        let mut accept = vec![false; self.transition.len()];
        accept.push(true);
        let mut transition = Vec::new();
        for s1 in 0..self.transition.len() {
            let mut nt = FxHashMap::default();
            for a in 0..2 {
                let s2 = self.transition[s1][a as usize];
                let b = (a + self.flip[s1]) % 2;
                let b0 = (a + self.flip[s1] + 1) % 2;
                nt.insert((a, b), s2);
                nt.insert((a, b0), self.transition.len());
            }
            transition.push(nt);
        }
        let mut nt = FxHashMap::default();
        for &c in &alphabet {
            nt.insert(c, self.transition.len());
        }
        transition.push(nt);
        let dfa = DFA {
            alphabet,
            transition,
            accept,
        }
        .minimize();
        //println!("{:?}", dfa);
        let mut sink = 0;
        for (s, &v) in dfa.accept.iter().enumerate() {
            if v {
                sink = s;
            }
        }
        let mut new_transition = Vec::new();
        let mut new_flip = Vec::new();
        for s1 in 0..dfa.transition.len() {
            let mut tn = vec![0; 2];
            let mut flip = false;
            if s1 == sink {
                continue;
            }
            for &(a, b) in &dfa.alphabet {
                if dfa.transition[s1][&(a, b)] != sink {
                    flip = a != b;
                    let s2 = dfa.transition[s1][&(a, b)];
                    if s2 < sink {
                        tn[a as usize] = s2;
                    } else {
                        tn[a as usize] = s2 - 1;
                    }
                }
            }
            new_flip.push(flip as u8);
            new_transition.push(tn);
        }
        //println!("{:?}, {:?}", self, dfa);
        return Transducer {
            transition: new_transition,
            flip: new_flip,
        };
    }

    /// Given a permutation of its states, create the corresponding
    ///  graph-isomorphic transducer.
    ///
    /// Warning: since `0` is the start state by convention, only permutations
    /// mapping `0` to itself are guaranteed to behave identically.
    pub fn relabel(&self, map: FxHashMap<usize, usize>) -> Self {
        // Validity check for the relabeling.
        for i in 0..map.len() {
            if !map.contains_key(&i) {
                panic!("map {:?} is not a function.", map);
            }
            for j in i..map.len() {
                if map[&i] == map[&j] {
                    for (a, b) in self.transition[i].iter().zip(self.transition[j].iter()) {
                        if map[&a] != map[&b] {
                            panic!("map {:?} is not a valid endomorphism.", map);
                        }
                    }
                }
            }
        }

        let mut new_transition: Vec<Vec<usize>> = vec![Vec::new(); self.transition.len()];
        let mut new_flip = vec![0; self.transition.len()];
        for state in 0..self.transition.len() {
            for sym in 0..2 {
                new_transition[map[&state]].push(map[&self.transition[state][sym]]);
            }
            new_flip[map[&state]] = self.flip[state];
        }
        return Transducer {
            transition: new_transition,
            flip: new_flip,
        };
    }

    /// Produce the inverse transducer, that is, one that undoes the operation
    /// of the given transducer.
    pub fn inverse(&self) -> Self {
        let mut new_transition: Vec<Vec<usize>> = Vec::new();
        for state in 0..self.transition.len() {
            if self.flip[state] != 0 {
                new_transition.push(self.transition[state].clone().into_iter().rev().collect());
            } else {
                new_transition.push(self.transition[state].clone());
            }
        }
        return Transducer {
            transition: new_transition,
            flip: self.flip.clone(),
        };
    }

    /// Create a canonical representation of a transducer under graph
    /// isomorphism and inversion.
    ///
    /// Exponential in the number of states.
    pub fn canonicalize(&self) -> Self {
        let mut min = self.clone();
        for m in Permutation::new(self.transition.len() - 1) {
            let mut map = FxHashMap::default();
            map.insert(0, 0);
            for (i, j) in m.iter().enumerate() {
                map.insert(i + 1, j + 1);
            }
            let cand1 = self.relabel(map);
            let cand2 = cand1.inverse();
            if cand1 < min {
                min = cand1;
            }
            if cand2 < min {
                min = cand2;
            }
        }
        return min;
    }

    /// Create the minimized semigroup product of two transducers.
    pub fn product(&self, other: &Transducer) -> Self {
        let mut new_transition = Vec::new();
        let mut new_flip = Vec::new();
        for i in 0..self.transition.len() {
            for j in 0..other.transition.len() {
                let mut nt = Vec::new();
                new_flip.push((self.flip[i] + other.flip[j]) % 2);
                for k in 0..2 {
                    nt.push(
                        self.transition[i][k] * self.transition.len()
                            + other.transition[j][(k + (self.flip[i] as usize)) % 2],
                    );
                }
                new_transition.push(nt);
            }
        }
        return Transducer {
            transition: new_transition,
            flip: new_flip,
        }
        .minimize();
    }

    /// Create a graph corresponding to the transducer's structure.
    pub fn graph(&self) -> Graph {
        let mut res = graph!(strict di id!());
        for i in 0..self.transition.len() {
            if self.flip[i] > 0 {
                res.add_stmt(stmt!(node!(i; attr!("shape", "diamond"))));
            } else {
                res.add_stmt(stmt!(node!(i; attr!("shape", "circle"))));
            }
            if self.transition[i][0] == self.transition[i][1] {
                res.add_stmt(stmt!(
                    edge!(node_id!(i) => node_id!(self.transition[i][0]); attr!("label", " a"))
                ));
            } else {
                for j in 0..2 {
                    res.add_stmt(stmt!(
                    edge!(node_id!(i) => node_id!(self.transition[i][j]); attr!("label", (format!(" {}", j))))
                ));
                }
            }
        }
        return res;
    }

    /// Create a graph corresponding to the transducer's orbit tree up to a
    /// given `depth`.
    pub fn orbit_tree(&self, depth: usize) -> Graph {
        let mut res = graph!(strict di id!();
          node!("\"\""; attr!("label", "\u{03b5}"), attr!("shape", "circle")));
        let mut words = vec![Vec::new()];
        for _ in 0..depth {
            let mut new_words = vec![];
            for word in words {
                let mut l = word.clone();
                let mut r = word.clone();
                l.push(0);
                r.push(1);
                let wlabel = format!(
                    "\"{}\"",
                    word.iter()
                        .map(|&x| x.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                );
                let l_label = format!(
                    "\"{}\"",
                    l.iter()
                        .map(|&x| x.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                );
                let r_label = format!(
                    "\"{}\"",
                    r.iter()
                        .map(|&x| x.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                );
                new_words.push(l.clone());
                res.add_stmt(stmt!(node!(l_label; attr!("shape", "circle"))));
                res.add_stmt(stmt!(edge!(node_id!(wlabel) => node_id!(l_label))));
                if self.min_word(&r) != l {
                    new_words.push(r.clone());
                    res.add_stmt(stmt!(node!(r_label; attr!("shape", "circle"))));
                    res.add_stmt(stmt!(edge!(node_id!(wlabel) => node_id!(r_label))));
                }
            }
            words = new_words;
        }
        return res;
    }
}

/// An iterator through all transducers of a particular size.
pub struct AllTransducers {
    state: TCycle<
        Vec<u8>,
        Vec<Vec<usize>>,
        VCycle<u8, UCycle<u8>>,
        VCycle<Vec<usize>, VCycle<usize, UCycle<usize>>>,
    >,
}

impl AllTransducers {
    /// Create a new iterator through all transducers of a particular `size`.
    pub fn new(size: usize) -> AllTransducers {
        AllTransducers {
            state: TCycle::new((vec![2; size], vec![vec![size; 2]; size]), true),
        }
    }
}

impl Iterator for AllTransducers {
    type Item = Transducer;

    fn next(&mut self) -> Option<Transducer> {
        match self.state.next() {
            None => None,
            Some((flip, transition)) => Some(Transducer {
                transition: transition
                    .into_iter()
                    .rev()
                    .map(|x| x.into_iter().rev().collect())
                    .collect(),
                flip: flip.into_iter().rev().collect(),
            }),
        }
    }
}

fn distinguish(class: &FxHashSet<Transducer>, depth: usize) -> Vec<FxHashSet<Transducer>> {
    let mut res: Vec<FxHashSet<Transducer>> = Vec::new();
    let mut remainder: Vec<_> = class.into_iter().collect();
    loop {
        if remainder.len() == 0 {
            break;
        }
        let cand = remainder.pop().unwrap();
        let (mut l, r): (Vec<_>, Vec<_>) = remainder
            .into_par_iter()
            .partition(|x| cand.orbit_compare(x, depth));
        l.push(cand);
        res.push(l.into_iter().map(|x| x.clone()).collect());
        remainder = r;
    }
    return res;
}

/// Divide transducers of a given size into *-equal classes.
pub fn classify_transducers(size: usize, depth: usize) -> Vec<FxHashSet<Transducer>> {
    let mut classes = Vec::new();
    let mut initial = FxHashSet::default();
    for m in AllTransducers::new(size).map(|x| x.minimize().canonicalize()) {
        if initial.contains(&m) {
            continue;
        } else {
            initial.insert(m.clone());
        }
    }
    classes.push(initial);
    for i in 1..depth + 1 {
        classes = classes
            .par_iter()
            .map(|x| distinguish(x, i))
            .collect::<Vec<_>>()
            .concat();
        println!("{} {}", i, classes.len());
    }
    return classes;
}
