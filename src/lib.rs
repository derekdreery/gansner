//! A library for laying out graphs according to the algorithm in the Gansner et al paper.
//!
//! The type of graph this draws is sometimes called a [layered graph].
//!
//! This library aims to be agnostic of any actual drawing: it takes minimal information about
//! nodes and edges as input (specifically node bounding boxes). We follow the Gansner paper but
//! do not promise to stick rigidly to it where subsequent advances mean there are better methods
//! available.
//!
//! I'd love to use `layout-rs` but can't figure out how to just get the draw coordinates without
//! using their render backends.
//!
//! # References
//!  - [Handbook of Graph Drawing and Visualization (Ed. Roberto
//!    Tamassia)](https://cs.brown.edu/people/rtamassi/gdhandbook/)
//!  - [A Technique for Drawing Directed Graphs (Gansner et.
//!    al.)](https://www.researchgate.net/publication/3187542_A_Technique_for_Drawing_Directed_Graphs)
//!
//! [layered graph]: https://en.wikipedia.org/wiki/Layered_graph_drawing
pub use crate::rank_set::RankIdx;
use crate::rank_set::RankSets;
use kurbo::{Point, Size};
use petgraph::graph::Graph;

mod layout;
mod rank_set;

type GansnerGraph<NodeData> = Graph<NodeWeight<NodeData>, Edge>;

pub struct Gansner<NodeData> {
    graph: GansnerGraph<NodeData>,
    /// User-supplied hints that certain nodes should share the same rank.
    rank_hints: RankSets,

    /// Has the layout algorithm been run since the last node/edge was added?
    fresh: bool,
}

impl<NodeData> Gansner<NodeData> {
    pub fn new() -> Self {
        Self::from_graph(Graph::new())
    }

    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self::from_graph(Graph::with_capacity(nodes, edges))
    }

    fn from_graph(graph: GansnerGraph<NodeData>) -> Self {
        Self {
            graph,
            rank_hints: RankSets::new(),
            fresh: false,
        }
    }

    /// Add a node to the graph.
    ///
    /// Note that here order matters! When breaking cycles, the direction of edges will be reversed
    /// for edges from later nodes to eariler ones. This is generally what you want as a user.
    pub fn add_node(&mut self, ix: NodeData, size: Size) -> Node {
        self.fresh = false;
        Node(self.graph.add_node(NodeWeight::new(ix, size)))
    }

    pub fn add_edge(&mut self, from: Node, to: Node) {
        self.fresh = false;
        self.graph.add_edge(from.0, to.0, Edge::new());
    }

    /// Add an edge, and in addition request that the edge traverses more than 1 rank (the number
    /// is specified by `min_rank`.
    pub fn add_edge_with_options(
        &mut self,
        from: Node,
        to: Node,
        min_rank_len: RankIdx,
        weight: f64,
    ) {
        self.fresh = false;
        self.graph.add_edge(
            from.0,
            to.0,
            Edge::new()
                .with_min_rank_len(min_rank_len)
                .with_weight(weight),
        );
    }

    /// Set the rank hint for a particular node to max.
    pub fn set_rank_max(&mut self, node: Node) {
        self.fresh = false;
        self.rank_hints.set_rank_max(node)
    }

    /// Set the rank hint for a particular node to min.
    pub fn set_rank_min(&mut self, node: Node) {
        self.fresh = false;
        self.rank_hints.set_rank_min(node)
    }

    /// Set the rank hint for a particular node to min.
    pub fn set_rank_same(&mut self, a: Node, b: Node) {
        self.fresh = false;
        self.rank_hints.set_rank(a, b)
    }

    /// Run the layout algorithm
    pub fn layout(&mut self) {
        if self.fresh == true {
            return;
        }
        self.layout_impl(false);
        self.fresh = true;
    }

    /// Run the layout algorithm, writing debug information to stdout.
    pub fn layout_debug(&mut self) {
        if self.fresh == true {
            return;
        }
        self.layout_impl(true);
        self.fresh = true;
    }

    /// Reverse an edge and return the index of the new edge.
    fn reverse_edge(&mut self, edge_ix: EdgeIx) -> EdgeIx {
        let (to, from) = self.graph.edge_endpoints(edge_ix).unwrap();
        let weight = self.graph.remove_edge(edge_ix).unwrap();
        self.graph.add_edge(from, to, weight)
    }
}

impl<NodeData: Clone> Gansner<NodeData> {
    pub fn iter_nodes(&self) -> impl Iterator<Item = (NodeData, Point)> + '_ {
        if !self.fresh {
            panic!("must call `layout` before iterating over nodes");
        }
        self.graph
            .node_weights()
            .map(|node| (node.ix.clone(), node.position))
    }
}

/// Called `NodeWeight` so we can use `Node` for returned handles.
#[derive(Clone)]
struct NodeWeight<Ix> {
    /// The index that was supplied by the user when adding the node.
    ix: Ix,
    /// The user-supplied size of the node's bounding box.
    size: Size,
    /// The calculated position of the node
    position: Point,
}

impl<Ix> NodeWeight<Ix> {
    fn new(ix: Ix, size: Size) -> Self {
        Self {
            ix,
            size,
            position: Point::ZERO,
        }
    }
}

struct Edge {
    /// Minimum number of ranks between edges (δ in paper). Defaults to `1`.
    min_rank_len: RankIdx,
    /// The edge weight, which should be a non-negative rational number (ω in paper). Defaults to
    /// `1`.
    weight: f64,
    /// Calculated path of edge when drawn TODO type
    position: (),
}

impl Edge {
    fn new() -> Self {
        Self {
            min_rank_len: 1,
            weight: 1.,
            position: (),
        }
    }

    fn with_min_rank_len(mut self, min_rank_len: RankIdx) -> Self {
        self.set_min_rank_len(min_rank_len);
        self
    }

    fn set_min_rank_len(&mut self, min_rank_len: RankIdx) {
        self.min_rank_len = min_rank_len;
    }

    fn with_weight(mut self, weight: f64) -> Self {
        self.set_weight(weight);
        self
    }

    fn set_weight(&mut self, weight: f64) {
        assert!(weight >= 0., "edge weight must be >= 0");
        self.weight = weight;
    }
}

/// Node handle used to insert edges.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node(NodeIx);

type NodeIx = petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>;
type EdgeIx = petgraph::graph::EdgeIndex<petgraph::graph::DefaultIx>;
