use itertools::Itertools;

use crate::{charset::CharSet, dfa::{Graph, IntoGraph, Node}};

use super::{Action, Expr};

pub struct Quantifier {
    /// The minimum number of times we encounter the expression.
    start: usize,
    /// The maximum number of times we encounter the expression.
    /// If None: infinite number of time
    end: Option<usize>
}

/// A{n,m} or A+, or A?, or A*
pub struct Quantified {
    /// The pattern to repeat
    pattern: Box<Expr>, 
    quantifier: Quantifier
}

impl IntoGraph<CharSet, Action> for Quantified {
    fn into_graph(self) -> crate::dfa::Graph<CharSet, Action> {
        let pattern = self.pattern.into_graph();
        let q = self.quantifier;

        let mut g = Graph::default();
        
        // Append the minimum number of patterns in the graph.
        for _ in 0..q.start {
            g.append(pattern.clone());
        }

        let minimums = if q.start == 0 {
            vec![Node::Start]
        } else {
            g.iter_tails().collect()
        };

        // Append additional optional pattern
        if let Some(end) = q.end {
            for _ in q.start..end {
                g.append(pattern.clone());
            }   
        }
        // Loop back infinitely
        if q.end.is_none() {
            g.append(pattern.clone());

            // The leaving nodes of the pattern
            let tails: Vec<_> = g.iter_tails().collect();
            
            heads
                .iter()
                .cartesian_product(tails.iter())
                .for_each(|(head, tail)| {
                    for edge in g.iter_precede(h) {
                        g.on(t, h, edge.set.clone(), edge.actions.clone())
                    }
                });
            
            heads
            .iter()
            .filter(|head| !head.is_start())
            .copied()
            .for_each(|head| g.on_with_lowest_priority(head, Node::Start, CharSet::All, []))
        } else {
            
        }

        minimums.iter().copied().for_each(|h| g.on_with_lowest_priority(h, Node::End, CharSet::All, []));

        g
    }
}
