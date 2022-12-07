use super::combi::Cycle;
use super::combi::Permutation;
use super::combi::TCycle;
use super::combi::UCycle;
use super::combi::VCycle;
use super::dfa::DFA;
use core::hash::Hash;
use graphviz_rust::cmd::CommandArg;
use graphviz_rust::cmd::Format;
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::exec;
use graphviz_rust::printer::PrinterContext;
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

fn repr(word: &[u8]) -> String {
    word.iter()
        .map(|&x| x.to_string())
        .collect::<Vec<_>>()
        .join("")
}

impl Transducer {
    /// Create a new transducer.
    pub fn new(transition: Vec<Vec<usize>>, flip: Vec<u8>) -> Self {
        Transducer { transition, flip }
    }

    /// TODO
    pub fn len(&self) -> usize {
        return self.transition.len();
    }

    /// TODO
    pub fn step(&self, x: &mut Vec<u8>) -> usize {
        let mut state: usize = 0;
        for i in 0..x.len() {
            let c = x[i];
            x[i] = c ^ self.flip[state];
            state = self.transition[state][c as usize];
        }
        return state;
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
                        self.transition[i][k] * other.transition.len()
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

    /// TODO
    pub fn detailed_orbit_tree(&self, depth: usize) {
        let mut cycle_trace = FxHashMap::default();
        let mut prev: Vec<Vec<u8>> = vec![Vec::new()];
        for _ in 1..depth + 1 {
            let mut new_prev = Vec::new();
            for word in &prev {
                let mut target0 = word.clone();
                let mut target1 = word.clone();
                target0.push(0);
                target1.push(1);
                let mut seen1 = false;
                let mut v = target0.clone();
                let mut res = Vec::new();
                new_prev.push(target0.clone());
                loop {
                    res.push(self.step(&mut v));
                    if v == target1 {
                        seen1 = true;
                    }
                    if v == target0 {
                        break;
                    }
                }
                cycle_trace.insert(target0, res);
                if !seen1 {
                    new_prev.push(target1.clone());
                    res = Vec::new();
                    let mut v = target1.clone();
                    loop {
                        res.push(self.step(&mut v));
                        if v == target1 {
                            break;
                        }
                    }
                    cycle_trace.insert(target1, res);
                }
            }
            prev = new_prev;
        }
        let mut out = cycle_trace.into_iter().collect::<Vec<_>>();
        out.sort_by(|(x, _), (y, _)| match x.len().cmp(&y.len()) {
            std::cmp::Ordering::Equal => x.cmp(&y),
            other => other,
        });
        for elem in out {
            let r0 = elem
                .0
                .iter()
                .map(|&x| (('0' as u8) + x) as char)
                .collect::<String>();
            let r1 = elem
                .1
                .iter()
                .map(|&x| (('A' as u8) + (x as u8)) as char)
                /* .map(|x| match x {
                    'A' => '$',
                    'B' => '.',
                    _ => '!',
                }) */
                .collect::<String>();
            println!("{:depth$} {}", r0, r1);
        }
    }

    /// Create a graph corresponding to the transducer's orbit tree up to a
    /// given `depth`.
    pub fn orbit_tree(&self, depth: usize) -> Graph {
        let mut res = graph!(strict di id!();
          node!(esc ""; attr!("label", "\u{03b5}"), attr!("shape", "circle"), attr!("root", "true")));
        let mut words = vec![Vec::new()];
        for _ in 0..depth {
            let mut new_words = vec![];
            for word in words {
                let mut l = word.clone();
                let mut r = word.clone();
                l.push(0);
                r.push(1);
                let wlabel = repr(&word);
                let l_label = repr(&l);
                let r_label = repr(&r);
                new_words.push(l.clone());
                res.add_stmt(stmt!(node!(esc l_label; attr!("shape", "circle"))));
                res.add_stmt(stmt!(edge!(node_id!(esc wlabel) => node_id!(esc l_label))));
                if self.min_word(&r) != l {
                    new_words.push(r.clone());
                    res.add_stmt(stmt!(node!(esc r_label; attr!("shape", "circle"))));
                    res.add_stmt(stmt!(edge!(node_id!(esc wlabel) => node_id!(esc r_label))));
                }
            }
            words = new_words;
        }
        return res;
    }

    /// TODO
    pub fn residues(&self) -> Vec<Self> {
        let mut remap_l = FxHashMap::default();
        let mut remap_r = FxHashMap::default();
        for i in 0..self.transition.len() {
            remap_l.insert(i, i);
            remap_r.insert(i, i);
        }
        remap_l.insert(0, self.transition[0][0]);
        remap_r.insert(0, self.transition[0][1]);
        remap_l.insert(self.transition[0][0], 0);
        remap_r.insert(self.transition[0][1], 0);
        let l = self.relabel(remap_l);
        let r = self.relabel(remap_r);
        if self.flip[0] == 0 {
            return vec![l.minimize(), r.minimize()];
        } else {
            return vec![l.product(&r).minimize(), r.product(&l).minimize()];
        }
    }
}

/// An iterator through all transducers of a particular size.
pub struct AllTransducers {
    state: (
        usize,
        TCycle<
            Vec<u8>,
            Vec<Vec<usize>>,
            VCycle<u8, UCycle<u8>>,
            VCycle<Vec<usize>, VCycle<usize, UCycle<usize>>>,
        >,
    ),
}

impl AllTransducers {
    /// Create a new iterator through all transducers of a particular `size`.
    pub fn new() -> AllTransducers {
        AllTransducers {
            state: (1, TCycle::new((vec![2; 1], vec![vec![1; 2]; 1]), true)),
        }
    }
}

impl Iterator for AllTransducers {
    type Item = Transducer;

    fn next(&mut self) -> Option<Transducer> {
        match self.state.1.next() {
            None => {
                self.state.0 += 1;
                self.state.1 = TCycle::new(
                    (
                        vec![2; self.state.0],
                        vec![vec![self.state.0; 2]; self.state.0],
                    ),
                    true,
                );
                self.next()
            }
            Some((flip, transition)) => Some(Transducer {
                transition: transition
                    .into_iter()
                    .rev()
                    .map(|x| x.into_iter().rev().collect())
                    .collect(),
                flip: flip.into_iter().map(|x| 1 - x).collect(),
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
/*
/// Determine whether a transducer's orbit is the maximal cycle.
fn maximal(m: &Transducer) -> bool {
    let mut trace: Vec<u8> = vec![0; m.transition.len()];
    let mut seen: FxHashSet<Vec<u8>> = FxHashSet::default();
    trace[0] = 1;
    loop {
        if trace
            .iter()
            .zip(m.flip.iter())
            .map(|(a, b)| a * b)
            .fold(0, |a, b| a + b)
            % 2
            == 0
        {
            return false;
        }
        if seen.contains(&trace) {
            return true;
        }
        let mut next = vec![0; m.transition.len()];
        for i in 0..m.transition.len() {
            for j in 0..2 {
                next[m.transition[i][j]] += 1 * trace[i];
                next[m.transition[i][j]] %= 2;
            }
        }
        seen.insert(trace.clone());
        trace = next;
    }
}

fn finitely_branching(m: &Transducer, depth: usize) -> bool {
    if depth == 0 {
        return maximal(m);
    } else {
        return maximal(m)
            || m.residues()
                .iter()
                .all(|m| finitely_branching(m, depth - 1));
    }
}

fn self_branching(m: &Transducer, depth: usize) -> bool {
    let residues = m.residues();
    let (l, r) = (residues[0].clone(), residues[1].clone());
    return (l.transition.len() == m.transition.len()
        && l.canonicalize() == m.canonicalize()
        && finitely_branching(&r, depth - 1))
        || (r.transition.len() == m.transition.len()
            && r.canonicalize() == m.canonicalize()
            && finitely_branching(&l, depth - 1));
}
*/

/// Divide transducers of a given size into *-equal classes.
pub fn classify_transducers(depth: usize) -> Vec<FxHashSet<Transducer>> {
    let mut pre_classes = Vec::new();
    let mut classes = Vec::new();
    let mut initial = FxHashSet::default();
    let mut key = FxHashMap::default();
    // let mut seen = 0;

    let mut ctx = PrinterContext::default();
    ctx.always_inline();
    for (i, m) in AllTransducers::new()
        .enumerate()
        .map(|(i, x)| (i, x.minimize().canonicalize()))
    {
        if i >= 66 {
            break;
        }
        if initial.contains(&m) {
            continue;
        } else {
            initial.insert(m.clone());
            key.insert(m.clone(), i);

            let m2 = m.clone();
            for _ in 1..9 {
                if m2.transition.len() > 12 {
                    break;
                }
                /* let g = m2.graph();
                exec(
                    g,
                    &mut ctx,
                    vec![
                        CommandArg::Custom("-Gdpi=300".to_string()),
                        //CommandArg::Custom("-Kcirco".to_string()),
                        CommandArg::Format(Format::Png),
                        CommandArg::Output(format!("images/m_{}_{}.png", i, j).to_string()),
                    ],
                )
                .unwrap();
                m2 = m2.product(&m); */
            }
            let mut depth = 6;
            if i == 27 || i == 23 {
                depth = 16;
            }
            let g = m.orbit_tree(depth);
            exec(
                g,
                &mut ctx,
                vec![
                    CommandArg::Custom("-Gdpi=300".to_string()),
                    CommandArg::Format(Format::Png),
                    //CommandArg::Custom("-Ktwopi".to_string()),
                    CommandArg::Output(format!("images/t_{}.png", i).to_string()),
                ],
            )
            .unwrap();
        }
    }
    /*
    let preds = vec![maximal];
    for (i, pred) in preds.iter().enumerate() {
        let (l, r): (Vec<_>, Vec<_>) = initial.into_par_iter().partition(pred);
        if l.len() > 0 {
            println!("{} {}", i, l.len());
            pre_classes.push(l.into_iter().map(|x| x.clone()).collect());
        }
        initial = r.into_iter().map(|x| x.clone()).collect();
    }
    */
    classes.push(initial);
    for i in 1..depth + 1 {
        classes = classes
            .par_iter()
            .map(|x| distinguish(x, i))
            .collect::<Vec<_>>()
            .concat();
    }
    /*
    let mut ctx = PrinterContext::default();
    ctx.always_inline();
    for class in &classes {
        if class.len() > 1 {
            let exemplar = class.iter().next().unwrap();
            if !finitely_branching(exemplar, depth) {
                let g = exemplar.orbit_tree(depth);
                exec(
                    g,
                    &mut ctx,
                    vec![
                        CommandArg::Custom("-Gdpi=300".to_string()),
                        CommandArg::Format(Format::Png),
                        CommandArg::Output(format!("images/t_{}.png", key[exemplar]).to_string()),
                    ],
                )
                .unwrap();
                for m in class {
                    for (i, m2) in m.residues().iter().enumerate() {
                        let g = m2.graph();
                        exec(
                            g,
                            &mut ctx,
                            vec![
                                CommandArg::Custom("-Gdpi=300".to_string()),
                                CommandArg::Format(Format::Png),
                                CommandArg::Output(
                                    format!("images/m_{}_{}.png", key[m], i).to_string(),
                                ),
                            ],
                        )
                        .unwrap();
                    }
                }
                println!(
                    "{} unclassified at depth {}: {:?}",
                    class.len(),
                    depth,
                    class.iter().map(|x| key[x]).collect::<Vec<_>>()
                );
            }
        }
    }
    */
    classes.append(&mut pre_classes);
    return classes;
}
