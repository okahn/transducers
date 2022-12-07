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

use combi::{Cycle, UCycle, VCycle};
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::{
    cmd::{CommandArg, Format},
    exec,
    printer::PrinterContext,
};
use rustc_hash::FxHashSet;
use std::env;
use transducer::{classify_transducers, AllTransducers, Transducer};

/// TODO
pub fn write_orbit_trees(depth: usize) {
    let res = classify_transducers(depth);
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

fn min_necklace(x: Vec<usize>) -> Vec<usize> {
    let mut min = x.clone();
    for offset in 0..x.len() {
        let xs: Vec<_> = (0..x.len()).map(|i| x[(i + offset) % x.len()]).collect();
        if xs < min {
            min = xs
        }
    }
    return min;
}

fn necklaces(width: usize) -> Vec<Vec<usize>> {
    let mut res = FxHashSet::default();
    for x in VCycle::<usize, UCycle<usize>>::new(vec![2; width], true) {
        res.insert(min_necklace(x));
    }
    let mut out: Vec<_> = res.into_iter().collect();
    out.sort();
    return out;
}

fn is_odd(x: &[usize]) -> bool {
    x.iter().filter(|&&k| k == 0).count() % 2 == 1
}

fn main() {
    let mut seen = FxHashSet::default();
    for (i, m) in AllTransducers::new().enumerate() {
        let m2 = m.minimize().canonicalize();
        if seen.contains(&m2) {
            continue;
        } else {
            seen.insert(m2);
        }
        if m.len() > 2 {
            break;
        }
        println!("{i} {m:?}")
    }
    let args: Vec<String> = env::args().collect();
    let depth = args[1].parse::<usize>().unwrap();
    let m = Transducer::new(vec![vec![0, 1], vec![1, 0]], vec![1, 0]);
    let m2 = Transducer::new(vec![vec![0, 1], vec![1, 0]], vec![0, 1]);
    println!(
        "{:?}",
        [
            &m,
            &m.product(&m).minimize(),
            &m.product(&m).product(&m).minimize()
        ]
    );
    m.detailed_orbit_tree(8);
    m.product(&m).product(&m).detailed_orbit_tree(8);
    let mut g = graph!(strict di id!());
    let width = depth;
    let mut g_even = subgraph!(id!("cluster_s0"));
    let mut g_odd = subgraph!(id!("cluster_s1"));
    g_even.stmts.push(stmt!(attr!("color", "white")));
    g_odd.stmts.push(stmt!(attr!("color", "white")));
    for x in necklaces(width) {
        let l_label: String = x.iter().map(|x| x.to_string()).collect();
        if is_odd(&x) {
            if x == vec![1; width] {
                g_odd.stmts.push(stmt!(
                    node!(esc l_label; /*(attr!("shape", "circle"),*/ attr!("root", "true"))
                ));
            } else {
                g_odd
                    .stmts
                    .push(stmt!(node!(esc l_label /*;attr!("shape", "circle"),*/)));
            }
        } else {
            if x == vec![1; width] {
                g_even.stmts.push(stmt!(
                    node!(esc l_label; /*(attr!("shape", "circle"),*/ attr!("root", "true"))
                ));
            } else {
                g_even
                    .stmts
                    .push(stmt!(node!(esc l_label/*;attr!("shape", "circle"),*/)));
            }
        }
    }
    g.add_stmt(stmt!(g_even));
    g.add_stmt(stmt!(g_odd));
    for y in necklaces(width) {
        for offset in 0..y.len() {
            let x: Vec<_> = (0..y.len()).map(|i| y[(i + offset) % y.len()]).collect();
            let color: Attribute;
            let constraint: Attribute;
            let l_label: String = y.iter().map(|x| x.to_string()).collect();
            let mut t0 = vec![0];
            let mut t1 = vec![1];
            let mut o0 = Vec::new();
            let mut o1 = Vec::new();
            for i in 0..width {
                match x[i] {
                    0 => {
                        o0.push(m.step(&mut t0));
                        o1.push(m.step(&mut t1));
                    }
                    1 => {
                        // This relabeling is NEEDED because M2 is labeled differently than M1.
                        o0.push(1 - m2.step(&mut t0));
                        o1.push(1 - m2.step(&mut t1));
                    }
                    _ => panic!(),
                }
            }
            if is_odd(&x) == is_odd(&o0) {
                color = attr!("style", "solid");
                // continue;
            } else if is_odd(&x) {
                color = attr!("style", "solid");
            } else {
                color = attr!("style", "solid");
            }
            constraint = attr!("constraint", ((!is_odd(&x)) || is_odd(&x) == is_odd(&o0)));
            // constraint = attr!("constraint", "true");
            let r0: String = min_necklace(o0).iter().map(|x| x.to_string()).collect();
            let r1: String = min_necklace(o1).iter().map(|x| x.to_string()).collect();

            if r1 == r0 {
                g.add_stmt(stmt!(
                    edge!(node_id!(esc l_label) => node_id!(esc r0); constraint, color)
                ));
            } else {
                g.add_stmt(stmt!(
                edge!(node_id!(esc l_label) => node_id!(esc r0); attr!("color", "blue"), constraint.clone(), color.clone())
            ));
                g.add_stmt(stmt!(
                edge!(node_id!(esc l_label) => node_id!(esc r1); attr!("color", "red"), constraint, color)
            ));
            }
            println!("{} {} {}", l_label, r0, r1);
        }
    }
    let mut ctx = PrinterContext::default();
    ctx.always_inline();
    // let g = m.orbit_tree(depth);
    exec(
        g,
        &mut ctx,
        vec![
            CommandArg::Custom("-Gdpi=300".to_string()),
            CommandArg::Custom("-Kdot".to_string()),
            CommandArg::Custom("-Goverlap=prism".to_string()),
            CommandArg::Custom("-Gsplines=ortho".to_string()),
            CommandArg::Format(Format::Png),
            CommandArg::Output(format!("images/m_interest.png").to_string()),
        ],
    )
    .unwrap();
    // write_orbit_trees(size, depth);
    /*
    let res = classify_transducers(depth);
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
    */
}
