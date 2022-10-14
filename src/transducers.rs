use super::cycles::Cycle;
use super::cycles::Permutation;
use super::cycles::TCycle;
use super::cycles::UCycle;
use super::cycles::VCycle;
use core::hash::Hash;
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use std::collections::HashMap;

// TODO move DFA stuff to another file.

/// A DFA, or deterministic finite automaton.
#[derive(Debug, PartialEq, Eq)]
pub struct DFA<T: Copy + Eq + Hash> {
    alphabet: Vec<T>,
    transition: Vec<HashMap<T, usize>>,
    accept: Vec<bool>,
}

/// TODO Change accept from vector to set? Decide.
impl<T: Copy + Eq + Hash> DFA<T> {
    /// Create a DFA with given alphabet, transition function, and accept states.
    pub fn new(alphabet: Vec<T>, transition: Vec<Vec<(T, usize)>>, accept: Vec<bool>) -> Self {
        let transition = transition
            .iter()
            .map(|x| x.into_iter().map(|&x| x.clone()).collect())
            .collect();
        DFA {
            alphabet,
            transition,
            accept,
        }
    }

    fn prune(&self) -> Self {
        let mut marked: Vec<usize> = vec![0];
        let mut frontier: Vec<usize> = vec![0];
        while frontier.len() > 0 {
            let mut new_frontier = Vec::new();
            for &s1 in &frontier {
                for (_, &s2) in &self.transition[s1] {
                    if !marked.contains(&s2) {
                        marked.push(s2);
                        new_frontier.push(s2);
                    }
                }
            }
            frontier = new_frontier;
        }
        let mut states_a = HashMap::new();
        let mut c = 0;
        for state in 0..self.transition.len() {
            if marked.contains(&state) {
                states_a.insert(state, c);
                c += 1;
            }
        }
        let mut new_transition: Vec<HashMap<T, usize>> = Vec::new();
        let mut new_accept = Vec::new();
        for state in 0..self.transition.len() {
            if states_a.contains_key(&state) {
                let mut nt = HashMap::new();
                for (&l, &r) in &self.transition[state] {
                    nt.insert(l, states_a[&r]);
                }
                new_transition.push(nt);
                new_accept.push(self.accept[state]);
            }
        }
        return DFA {
            alphabet: self.alphabet.clone(),
            transition: new_transition,
            accept: new_accept,
        };
    }

    /// Returns the minimal equivalent DFA.
    ///
    /// At least O(n^2) time in the number of states.
    /// Does not canonicalize the resulting DFA: to check equality you must also find an isomorphism.
    pub fn minimize(&self) -> Self {
        let _s = self.prune();
        let mut marked = Vec::new();
        let mut unmarked = Vec::new();
        for i in 0.._s.transition.len() {
            for j in i + 1.._s.transition.len() {
                if _s.accept[i] == _s.accept[j] {
                    unmarked.push((i, j));
                } else {
                    marked.push((i, j));
                }
            }
        }
        loop {
            let mut new_unmarked = Vec::new();
            for (i, j) in unmarked.clone() {
                let mut found = false;
                for sym in _s.transition[i].keys() {
                    let a = _s.transition[i][sym];
                    let b = _s.transition[j][sym];
                    if marked.contains(&(a, b)) || marked.contains(&(b, a)) {
                        marked.push((i, j));
                        found = true;
                        break;
                    }
                }
                if !found {
                    new_unmarked.push((i, j))
                }
            }
            if unmarked == new_unmarked {
                break;
            } else {
                unmarked = new_unmarked;
            }
        }
        let mut states_a: HashMap<usize, usize> = HashMap::new();
        for (i, j) in unmarked {
            if !states_a.contains_key(&j) {
                states_a.insert(j, i);
            } else if states_a[&j] > i {
                states_a.insert(j, i);
            }
        }
        let mut states_b: HashMap<usize, usize> = HashMap::new();
        let mut c = 0;
        for state in 0.._s.transition.len() {
            if !states_a.contains_key(&state) {
                states_a.insert(state, state);
                states_b.insert(state, c);
                c += 1;
            }
        }
        let mut states_c: HashMap<usize, usize> = HashMap::new();
        for state in 0.._s.transition.len() {
            states_c.insert(state, states_b[&states_a[&state]]);
        }
        let mut new_transition: Vec<HashMap<T, usize>> = Vec::new();
        let mut new_accept = Vec::new();
        for state in 0.._s.transition.len() {
            if states_b.contains_key(&state) {
                let mut nt = HashMap::new();
                for (&l, &r) in &_s.transition[state] {
                    nt.insert(l, states_b[&states_a[&r]]);
                }
                new_transition.push(nt);
                new_accept.push(_s.accept[state]);
            }
        }
        let out = DFA {
            alphabet: _s.alphabet.clone(),
            transition: new_transition,
            accept: new_accept,
        };
        return out;
    }

    /* fn canonicalize(&self) -> Self {
        let mut states_a = Vec::new();
        let mut frontier = Vec::new();
        states_a.push(0);
        frontier.push(0);
        while frontier.len() > 0 {
            let mut new_frontier = Vec::new();
            for &s1 in &frontier {
                for &sym in &self.alphabet {
                    let s2 = self.transition[s1][&sym];
                    if !states_a.contains(&s2) {
                        new_frontier.push(s2);
                        states_a.push(s2);
                    }
                }
            }
            frontier = new_frontier;
        }

        let mut new_transition: Vec<HashMap<T, usize>> = Vec::new();
        let mut new_accept =  Vec::new();
        for &state in &states_a {
            let mut nt = HashMap::new();
            for (&l, &r) in &self.transition[state] {
                nt.insert(l, states_a[r]);
            }
            new_transition.push(nt);
            new_accept.push(self.accept[state]);
        }
        return DFA {
            alphabet: self.alphabet.clone(),
            transition: new_transition,
            accept: new_accept
        };
    } */
}

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
    fn step(&self, x: &[u8]) -> Vec<u8> {
        let mut state: usize = 0;
        let mut out: Vec<u8> = Vec::new();
        for &c in x {
            out.push(c ^ self.flip[state]);
            state = self.transition[state][c as usize];
        }
        return out;
    }

    fn min_word(&self, word: &[u8]) -> Vec<u8> {
        let mut min = word.to_vec();
        let mut next = self.step(&word);
        loop {
            if next < min {
                min = next.clone();
            }
            if next == word {
                return min;
            }
            next = self.step(&next);
        }
    }

    /// Compare the orbits of two transducers on strings up to length `depth`.
    ///
    /// At least O(2^n) time where n is `depth`, but memory-efficient.
    pub fn orbit_compare(&self, other: &Transducer, depth: usize) -> bool {
        for word in VCycle::<u8, UCycle<u8>>::new(vec![2; depth], true) {
            if self.min_word(&word) != other.min_word(&word) {
                return false;
            }
        }
        return true;
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
            let mut nt = HashMap::new();
            for a in 0..2 {
                let s2 = self.transition[s1][a as usize];
                let b = (a + self.flip[s1]) % 2;
                let b0 = (a + self.flip[s1] + 1) % 2;
                nt.insert((a, b), s2);
                nt.insert((a, b0), self.transition.len());
            }
            transition.push(nt);
        }
        let mut nt = HashMap::new();
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

    // TODO Check that this is actually a permutation before proceeding?
    // If not, guarantee that the merging of states is well-defined somehow.

    /// Given a permutation of its states, create the corresponding
    ///  graph-isomorphic transducer.
    ///
    /// Warning: since `0` is the start state by convention, only permutations
    /// mapping `0` to itself are guaranteed to behave identically.
    pub fn relabel(&self, map: HashMap<usize, usize>) -> Self {
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

    // TODO Rename `reverse` to `invert` or `inverse`.

    /// Produce the inverse transducer, that is, one that undoes the operation
    /// of the given transducer.
    pub fn reverse(&self) -> Self {
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
            let mut map = HashMap::new();
            map.insert(0, 0);
            for (i, j) in m.iter().enumerate() {
                map.insert(i + 1, j + 1);
            }
            let cand1 = self.relabel(map);
            let cand2 = cand1.reverse();
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
