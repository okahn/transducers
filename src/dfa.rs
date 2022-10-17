use core::hash::Hash;
use rustc_hash::FxHashMap;

/// A DFA, or deterministic finite automaton.
#[derive(Debug, PartialEq, Eq)]
pub struct DFA<T: Copy + Eq + Hash> {
    /// The alphabet of the DFA.
    pub alphabet: Vec<T>,
    /// The transition function of the DFA.
    /// Because states are consecutive integers beginning with 0,
    /// each partial function is an element of a vector.
    pub transition: Vec<FxHashMap<T, usize>>,
    /// The accept set of the DFA represented as a boolean vector.
    pub accept: Vec<bool>,
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
        let mut states_a = FxHashMap::default();
        let mut c = 0;
        for state in 0..self.transition.len() {
            if marked.contains(&state) {
                states_a.insert(state, c);
                c += 1;
            }
        }
        let mut new_transition: Vec<FxHashMap<T, usize>> = Vec::new();
        let mut new_accept = Vec::new();
        for state in 0..self.transition.len() {
            if states_a.contains_key(&state) {
                let mut nt = FxHashMap::default();
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
        let mut states_a: FxHashMap<usize, usize> = FxHashMap::default();
        for (i, j) in unmarked {
            if !states_a.contains_key(&j) {
                states_a.insert(j, i);
            } else if states_a[&j] > i {
                states_a.insert(j, i);
            }
        }
        let mut states_b: FxHashMap<usize, usize> = FxHashMap::default();
        let mut c = 0;
        for state in 0.._s.transition.len() {
            if !states_a.contains_key(&state) {
                states_a.insert(state, state);
                states_b.insert(state, c);
                c += 1;
            }
        }
        let mut states_c: FxHashMap<usize, usize> = FxHashMap::default();
        for state in 0.._s.transition.len() {
            states_c.insert(state, states_b[&states_a[&state]]);
        }
        let mut new_transition: Vec<FxHashMap<T, usize>> = Vec::new();
        let mut new_accept = Vec::new();
        for state in 0.._s.transition.len() {
            if states_b.contains_key(&state) {
                let mut nt = FxHashMap::default();
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
