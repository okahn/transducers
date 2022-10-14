#![warn(missing_docs)]

//! Tools for manipulating transducers.
//!
//! Currently this program only handles reversible alphabetic transducers on a
//! binary alphabet.

/// Iterator representations of products and permutations.
pub mod cycles;

/// Representations of transducers and DFAs.
pub mod transducers;

use graphviz_rust::cmd::{CommandArg, Format};
use graphviz_rust::exec;
use graphviz_rust::printer::PrinterContext;
use std::collections::HashSet;
use transducers::AllTransducers;
use transducers::Transducer;

fn main() {
    /* let x = Permutation { state: 0, width: 4 };
    for a in x {
        println!("{:?}", a);
    }
    let a = DFA::new(
        vec!['a', 'b'],
        vec![
            vec![('a', 2), ('b', 1)],
            vec![('a', 0), ('b', 0)],
            vec![('a', 0), ('b', 0)]
            ],
        vec![true, false, true]);
    println!("{:?}\n{:?}", a, a.minimize());
    */

    let n_states = 3;
    let depth = 10;
    let mut classes: Vec<Vec<Transducer>> = Vec::new();
    let mut seen: HashSet<Transducer> = HashSet::new();

    let mut ctx = PrinterContext::default();
    ctx.always_inline();
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
        /* let graph_png = exec(
            m.graph(),
            &mut PrinterContext::default(),
            vec![CommandArg::Format(Format::Png)],
        )
        .unwrap(); */

        /* let pathstr = format!("images/{}.png", i);
        let path = Path::new(&pathstr);
        let display = path.display();
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };
        match file.write_all(graph_png.as_bytes()) {
            Err(why) => panic!("couldn't write to {}: {}", display, why),
            Ok(_) => println!("successfully wrote to {}", display),
        } */

        let ci;
        match found {
            Some(c) => {
                classes[c].push(m.clone());
                ci = c;
            }
            None => {
                ci = classes.len();
                classes.push(vec![m.clone()]);
            }
        }

        println!("{} {:?}", i + 1, m);

        let g1 = m.graph();
        exec(
            g1,
            &mut ctx,
            vec![
                CommandArg::Custom("-Gdpi=300".to_string()),
                CommandArg::Format(Format::Png),
                CommandArg::Output(format!("images/m_{}_{}_{}.png", n_states, ci, i).to_string()),
            ],
        )
        .unwrap();
    }
    println!(
        "{} {:?}",
        classes.len(),
        classes.iter().map(|x| x.len()).collect::<Vec<_>>()
    );

    for (i, class) in classes.iter().enumerate() {
        if class.len() > 1 {
            println!("{} {}", i, class.len());

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
        }
    }
}
