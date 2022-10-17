#![warn(missing_docs)]

//! Tools for manipulating transducers.
//!
//! Currently this program only handles reversible alphabetic transducers on a
//! binary alphabet.

/// Iterator representations of products and permutations.
pub mod cycles;

/// Representations of DFAs.
pub mod dfa;

/// Representations of transducers and DFAs.
pub mod transducer;

/* use graphviz_rust::cmd::{CommandArg, Format};
use graphviz_rust::exec;
use graphviz_rust::printer::PrinterContext; */
// use std::collections::HashSet;
use std::env;
// use transducers::AllTransducers;
// use transducers::Transducer;

use rustc_hash::FxHashMap;

use transducer::classify_transducers;

fn main() {
    let args: Vec<String> = env::args().collect();
    let size = args[1].parse::<usize>().unwrap();
    let depth = args[2].parse::<usize>().unwrap();
    let res = classify_transducers(size, depth);
    let mut cs: FxHashMap<usize, u64> = FxHashMap::default();
    for i in res.iter().map(|x| x.len()) {
        if cs.contains_key(&i) {
            cs.insert(i, cs[&i] + 1);
        } else {
            cs.insert(i, 1);
        }
    }
    let mut cs2: Vec<_> = cs.into_iter().collect();
    cs2.sort();
    println!("{} {:?}", res.len(), cs2);
    /*
    let n_states = 3;
    let depth = 10;
    let mut classes: Vec<Vec<Transducer>> = Vec::new();
    let mut seen: HashSet<Transducer> = HashSet::new();

    // let mut ctx = PrinterContext::default();
    // ctx.always_inline();
    for (i, m) in AllTransducers::new(n_states)
        .map(|x| x.minimize().canonicalize())
        .enumerate()
    {
        if seen.contains(&m) {
            continue;
        } else {
            seen.insert(m.clone());
        }
        assert!(m.orbit_compare(&m.minimize().canonicalize(), 4));
        let mut found = None;
        for (i, class) in classes.iter().enumerate() {
            if m.orbit_compare(&class[0], depth) {
                found = Some(i);
                break;
            }
        }

        // let ci;
        match found {
            Some(c) => {
                classes[c].push(m.clone());
                // ci = c;
            }
            None => {
                // ci = classes.len();
                classes.push(vec![m.clone()]);
            }
        }

        println!("{} {:?}", i + 1, m);

        /* let g1 = m.graph();
        exec(
            g1,
            &mut ctx,
            vec![
                CommandArg::Custom("-Gdpi=300".to_string()),
                CommandArg::Format(Format::Png),
                CommandArg::Output(format!("images/m_{}_{}_{}.png", n_states, ci, i).to_string()),
            ],
        )
        .unwrap(); */
    }
    println!(
        "{} {:?}",
        classes.len(),
        classes.iter().map(|x| x.len()).collect::<Vec<_>>()
    );

    for (i, class) in classes.iter().enumerate() {
        if class.len() > 1 {
            println!("{} {}", i, class.len());

            /*
            let g2 = class[0].orbit_tree(depth);
            exec(
                g2,
                &mut ctx,
                vec![
                    CommandArg::Custom("-Gdpi=300".to_string()),
                    CommandArg::Format(Format::Png),
                    CommandArg::Output(format!("images/c_{}_{}.png", n_states, i).to_string()),
                ],
            )
            .unwrap();
            */
        }
    }
    */
}
