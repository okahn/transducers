#![warn(missing_docs)]

//! Tools for manipulating transducers.
//!
//! Currently this program only handles reversible alphabetic transducers on a
//! binary alphabet.

/// Iterator representations of products and permutations.
pub mod combi;

/// Representations of DFAs.
pub mod dfa;

/// Representations of transducers and DFAs.
pub mod transducer;

use graphviz_rust::{
    cmd::{CommandArg, Format},
    exec,
    printer::PrinterContext,
};
use rustc_hash::FxHashMap;
use std::env;
use transducer::classify_transducers;

fn write_orbit_trees(size: usize, depth: usize) {
    let res = classify_transducers(size, depth);
    let mut ctx = PrinterContext::default();
    ctx.always_inline();
    for (i, m) in res.iter().enumerate() {
        let g = m.into_iter().next().unwrap().orbit_tree(depth);
        exec(
            g,
            &mut ctx,
            vec![
                CommandArg::Custom("-Gdpi=300".to_string()),
                CommandArg::Format(Format::Png),
                CommandArg::Output(format!("images/t_{}.png", i).to_string()),
            ],
        )
        .unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let size = args[1].parse::<usize>().unwrap();
    let depth = args[2].parse::<usize>().unwrap();
    // write_orbit_trees(size, depth);
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
}
