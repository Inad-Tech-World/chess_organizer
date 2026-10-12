//! Maximum-weight matching on general graphs (Edmonds' blossom algorithm).
//!
//! Planned for Phase 1 as the core of the Dutch engine. Weights are wide
//! unsigned integers because the Dutch quality criteria are packed into one
//! weight in strict priority order; the width grows with the number of
//! players.

use co_core::{Result, not_yet};

/// Undirected edge `(u, v, weight)` between vertices `0..vertex_count`.
pub type Edge<W> = (usize, usize, W);

/// Returns, for each vertex, its mate in a maximum-weight matching. With
/// `max_cardinality`, only maximum-cardinality matchings are considered.
pub fn max_weight_matching<W: Ord + Clone>(
    _vertex_count: usize,
    _edges: &[Edge<W>],
    _max_cardinality: bool,
) -> Result<Vec<Option<usize>>> {
    not_yet!("weighted matching", 1)
}
