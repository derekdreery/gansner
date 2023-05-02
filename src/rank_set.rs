use crate::Node;
use std::collections::HashMap;

/// A set of disjoint subsets of all nodes, indicating preferred rank.
///
/// A node should be in at most 1 `rank`, if a node is added twice it will result in a panic.
// Note some special rank indexes: 0 = min rank, RankIdx::MAX = max rank.
pub struct RankSets {
    ranks: HashMap<Node, RankIdx>,
    next_rank_idx: RankIdx,
}

impl RankSets {
    const MIN_RANK: RankIdx = 0;
    const MAX_RANK: RankIdx = RankIdx::MAX;
    pub fn new() -> Self {
        Self {
            ranks: HashMap::new(),
            next_rank_idx: 1,
        }
    }

    /// Request that the node be given max rank.
    pub fn set_rank_max(&mut self, node: Node) {
        assert!(
            self.node_rank(node).is_none(),
            "node already has a rank hint"
        );
        self.ranks.insert(node, Self::MAX_RANK);
    }

    /// Request that the node be given max rank.
    pub fn set_rank_min(&mut self, node: Node) {
        assert!(
            self.node_rank(node).is_none(),
            "node already has a rank hint"
        );
        self.ranks.insert(node, Self::MIN_RANK);
    }

    /// Indicate that the two nodes should be ranked together.
    ///
    /// If both nodes already have hints, then those two groups will be merged.
    ///
    /// # Panics
    ///
    /// The function will panic if one node is from the max rank group and the other is from the
    /// min rank group
    pub fn set_rank(&mut self, a: Node, b: Node) {
        match (self.node_rank(a), self.node_rank(b)) {
            (Some(rank_a), Some(rank_b)) => {
                let (rank_a, rank_b) = if rank_a > rank_b {
                    (rank_b, rank_a)
                } else {
                    (rank_a, rank_b)
                };
                // now rank_a <= rank_b
                if rank_a == Self::MIN_RANK && rank_b == Self::MAX_RANK {
                    panic!("attempted to merge min and max ranks");
                }
                if rank_a == Self::MIN_RANK {
                    self.merge_ranks(rank_b, rank_a)
                } else if rank_b == Self::MAX_RANK {
                    self.merge_ranks(rank_a, rank_b)
                } else {
                    // doesn't matter which way we merge. Could optimize the branch above but
                    // hopefully the optimizer will do it anyway - code is more readable as-is.
                    self.merge_ranks(rank_a, rank_b)
                }
            }
            (Some(rank), None) => self.add_rank(b, rank),
            (None, Some(rank)) => self.add_rank(a, rank),
            (None, None) => {
                let rank = self.new_rank();
                self.add_rank(a, rank);
                self.add_rank(b, rank);
            }
        }
    }

    /// Get all nodes with minimum rank
    pub fn rank_min(&self) -> impl Iterator<Item = &Node> + '_ {
        self.ranks.iter().filter_map(|(node, rank_idx)| {
            if *rank_idx == Self::MIN_RANK {
                Some(node)
            } else {
                None
            }
        })
    }

    /// Get all nodes with maximum rank
    pub fn rank_max(&self) -> impl Iterator<Item = &Node> + '_ {
        self.ranks.iter().filter_map(|(node, rank_idx)| {
            if *rank_idx == Self::MAX_RANK {
                Some(node)
            } else {
                None
            }
        })
    }

    /// Get all nodes with given rank
    fn rank(&self, idx: RankIdx) -> impl Iterator<Item = &Node> + '_ {
        self.ranks.iter().filter_map(
            move |(node, rank_idx)| {
                if *rank_idx == idx {
                    Some(node)
                } else {
                    None
                }
            },
        )
    }

    /// Get the rank for a particular node.
    pub fn node_rank(&self, node: Node) -> Option<RankIdx> {
        self.ranks.get(&node).copied()
    }

    fn merge_ranks(&mut self, from: RankIdx, to: RankIdx) {
        for rank in self.ranks.values_mut() {
            if *rank == from {
                *rank = to;
            }
        }
    }

    fn add_rank(&mut self, node: Node, rank: RankIdx) {
        self.ranks.insert(node, rank);
    }

    fn new_rank(&mut self) -> RankIdx {
        assert_ne!(
            self.next_rank_idx,
            RankIdx::MAX,
            "number of ranks overflowed index type"
        );
        let rank = self.next_rank_idx;
        self.next_rank_idx += 1;
        rank
    }
}

pub type RankIdx = usize;
