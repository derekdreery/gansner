//! module with the layout algorithm.

use crate::{Edge, EdgeIx, Gansner, NodeIx};
use petgraph::{visit::EdgeRef, Direction::*};

impl<NodeData> Gansner<NodeData> {
    pub(crate) fn layout_impl(&mut self, debug: bool) {
        // 1) Rank nodes
        //
        // in debug mode we check that
        //   a) we reduce the graph to an acyclic one
        //   b) rank min only has outgoing edges
        //   c) rank max only has incoming edges
        //   d) when we restore the original graph, we actually get back what we had at the start
        #[cfg(debug_assertions)]
        let copy = self.graph.map(|_, _| (), |_, _| ());

        let modified = self.make_acyclic();
        debug_assert!(!petgraph::algo::is_cyclic_directed(&self.graph));
        debug_assert!(self.rank_hints.rank_min().all(|node_idx| self
            .graph
            .edges_directed(node_idx.0, Incoming)
            .next()
            .is_none()));
        debug_assert!(self.rank_hints.rank_max().all(|node_idx| self
            .graph
            .edges_directed(node_idx.0, Outgoing)
            .next()
            .is_none()));

        // TODO assign ranks

        self.undo_modify_edges(modified);
        #[cfg(debug_assertions)]
        debug_assert!(petgraph::algo::is_isomorphic(&copy, &self.graph));

        // 2) ...
    }

    // Hmm actually I don't think I can mutate the graph and re-create the original. Instead, we
    // could mark
    /// This function
    ///  1. condenses all nodes in each user-supplied sets (inc. min and max) down into a single
    ///     node.
    ///  2. ensures that all edges are outgoing for Smin and incoming for Smax.
    ///  2. removes loops.
    ///  3. merges multiple edges into a single edge whose weight is the sum of the individual
    ///     edges' weight.
    ///  4. removes leaf nodes that are not part of S1..Sk, Smin, Smax.
    ///  5. makes the graph acyclic by reversing edges. This is currently done using the greedy
    ///     algorithm, but we probably want to switch to gansner's heuristic algo.
    ///  6. adds an edge for all nodes with no incoming edge from Smin with min rank length = 0,
    ///     and the same for nodes with no outgoing edge (min rank length 0 edge to Smax).
    ///
    /// The return value is the information required to reconstruct the original graph.
    fn prepare_rank_assignment(&mut self) -> RankAssignmentAdjustment {
        let mut remove_ixs = Vec::new();
        let mut reverse_ixs = Vec::new();

        // Ensure all edges go out of min rank and into max rank
        //
        // If the edge is Smin -> Smin or Smax -> Smax then we reverse it, but it will have no
        // effect on rank assignment, since both nodes will be given min/max rank.
        for node_id in self.rank_hints.rank_min() {
            for edge in self.graph.edges_directed(node_id.0, Incoming) {
                reverse_ixs.push(edge.id());
            }
        }

        for node_id in self.rank_hints.rank_max() {
            for edge in self.graph.edges_directed(node_id.0, Outgoing) {
                reverse_ixs.push(edge.id());
            }
        }

        let mut reverse: Vec<_> = reverse_ixs
            .drain(..)
            .map(|id| self.reverse_edge(id))
            .collect();

        // Make acyclic
        //
        // TODO assuming the definition of a feedback set (FS) from rtamassi handbook, I need to
        // make sure that this algorithm (which returns a feedback arc set (FAS)) also returns a
        // feedback set.
        //
        // If we want to implement Gansner's algo (we do) the method is:
        //  1. Create a `TarjanScc`
        //  2. repeat
        //      i. `run` it on our graph
        //      ii. for each component, do a DFS and reverse the edge that
        //          participates in the most cycles
        //
        //     until there are no non-trivial strongly connected components
        //
        // This is however work that I'm avoiding for now. The scc methods in petgraph claim that
        // the node order is arbitary, but it looks like it is order of insertion, which is what we
        // want.
        for edge in petgraph::algo::greedy_feedback_arc_set(&self.graph) {
            // skip loops
            if edge.source() == edge.target() {
                remove_ixs.push(edge.id());
            } else {
                reverse_ixs.push(edge.id());
            }
        }

        // We don't actually need to remove edges, but for now we do because it means we can check
        // the output of this stage is a DAG.
        let remove = remove_ixs
            .drain(..)
            .map(|id| {
                let (from, _) = self.graph.edge_endpoints(id).unwrap();
                let weight = self.graph.remove_edge(id).unwrap();
                RemovedEdge {
                    from,
                    to: from,
                    weight,
                }
            })
            .collect();

        reverse.extend(reverse_ixs.into_iter().map(|id| self.reverse_edge(id)));

        // TODO in the paper it talks about adding temp edges from Smin to e and from e to Smax
        // when there is no incoming/outgoing edge respectively, to ensure all nodes lie on a path
        // from Smin to Smax. I'm not bothering to do this for now - will add it if/when I
        // understand why it is needed, so I know what to choose for the weight/nodes.

        (remove, reverse)
    }

    /// flip any reversed edges
    fn undo_modify_edges(&mut self, (remove, reverse): (Vec<RemovedEdge<Edge>>, Vec<EdgeIx>)) {
        for RemovedEdge { from, to, weight } in remove {
            self.graph.add_edge(from, to, weight);
        }
        for id in reverse {
            self.reverse_edge(id);
        }
    }
}

struct RankAssignmentAdjustment {
    /// edges that were removed
    edges_removed: Vec<RemovedEdge>,
}

struct RemovedEdge<T> {
    from: NodeIx,
    to: NodeIx,
    weight: T,
}
