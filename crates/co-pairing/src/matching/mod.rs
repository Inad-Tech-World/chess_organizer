//! Maximum-weight matching on general graphs, with incremental updates.
//!
//! [`Computer`] is a port of bbpPairings' `matching::Computer`: the graph is
//! complete, missing edges have weight zero and are never matched, and after
//! changing the edges of k vertices a new matching costs O(k·n²).

mod graph;
pub mod wide;

pub use wide::Wide;

pub struct Computer {
    graph: graph::Graph,
}

impl Computer {
    /// A computer for up to `capacity` vertices and edge weights up to
    /// `max_edge_weight`, which also fixes the width of every weight.
    pub fn new(capacity: usize, max_edge_weight: &Wide) -> Self {
        Self {
            graph: graph::Graph::new(capacity, max_edge_weight),
        }
    }

    pub fn size(&self) -> usize {
        self.graph.size()
    }

    /// Adds a vertex with the next index.
    pub fn add_vertex(&mut self) {
        self.graph.add_vertex();
    }

    /// Sets the weight of the edge between `u` and `v` (zero removes it).
    pub fn set_edge_weight(&mut self, u: usize, v: usize, weight: &Wide) {
        assert_ne!(u, v);
        self.graph.set_edge_weight(u, v, weight);
    }

    pub fn compute_matching(&mut self) {
        self.graph.compute_matching();
    }

    /// Each vertex's mate; an unmatched vertex is its own mate.
    pub fn get_matching(&self) -> Vec<usize> {
        self.graph.get_matching()
    }
}

/// One-shot maximum-weight matching for small integer weights.
pub fn max_weight_matching(
    vertex_count: usize,
    edges: &[(usize, usize, u64)],
) -> Vec<Option<usize>> {
    let max = edges.iter().map(|e| e.2).max().unwrap_or(0);
    let mut max_w = Wide::from_u64(max);
    max_w.shift_grow(3);
    max_w.shr(3);
    let mut c = Computer::new(vertex_count, &max_w);
    for _ in 0..vertex_count {
        c.add_vertex();
    }
    for &(u, v, w) in edges {
        c.set_edge_weight(u, v, &max_w.small_like(w));
    }
    c.compute_matching();
    c.get_matching()
        .into_iter()
        .enumerate()
        .map(|(i, m)| (m != i).then_some(m))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
    }

    fn brute_force(n: usize, w: &[Vec<u64>]) -> u64 {
        fn go(i: usize, used: &mut Vec<bool>, n: usize, w: &[Vec<u64>]) -> u64 {
            if i == n {
                return 0;
            }
            if used[i] {
                return go(i + 1, used, n, w);
            }
            used[i] = true;
            let mut best = go(i + 1, used, n, w);
            for j in i + 1..n {
                if !used[j] && w[i][j] > 0 {
                    used[j] = true;
                    best = best.max(w[i][j] + go(i + 1, used, n, w));
                    used[j] = false;
                }
            }
            used[i] = false;
            best
        }
        go(0, &mut vec![false; n], n, w)
    }

    fn total(m: &[Option<usize>], w: &[Vec<u64>]) -> u64 {
        m.iter()
            .enumerate()
            .filter_map(|(i, j)| j.filter(|&j| j > i).map(|j| w[i][j]))
            .sum()
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn matches_brute_force_on_random_graphs() {
        let mut rng = Rng(0x1234_5678_9abc_def1);
        for case in 0..400 {
            let n = 2 + (rng.next() % 9) as usize;
            let mut w = vec![vec![0u64; n]; n];
            let mut edges = Vec::new();
            for i in 0..n {
                for j in i + 1..n {
                    if !rng.next().is_multiple_of(3) {
                        let x = 1 + rng.next() % 20;
                        w[i][j] = x;
                        w[j][i] = x;
                        edges.push((i, j, x));
                    }
                }
            }
            let m = max_weight_matching(n, &edges);
            for (i, mate) in m.iter().enumerate() {
                if let Some(j) = *mate {
                    assert_eq!(m[j], Some(i), "case {case}: asymmetric");
                    assert!(w[i][j] > 0, "case {case}: matched a non-edge");
                }
            }
            assert_eq!(total(&m, &w), brute_force(n, &w), "case {case}: n={n}");
        }
    }

    #[test]
    fn incremental_updates_stay_optimal() {
        let mut rng = Rng(42);
        let n = 9;
        let mut w = vec![vec![0u64; n]; n];
        let mut max_w = Wide::from_u64(64);
        max_w.shift_grow(3);
        max_w.shr(3);
        let mut c = Computer::new(n, &max_w);
        for _ in 0..n {
            c.add_vertex();
        }
        for _ in 0..200 {
            let u = (rng.next() % n as u64) as usize;
            let v = (rng.next() % n as u64) as usize;
            if u == v {
                continue;
            }
            let x = rng.next() % 40;
            w[u][v] = x;
            w[v][u] = x;
            c.set_edge_weight(u, v, &max_w.small_like(x));
            c.compute_matching();
            let m: Vec<Option<usize>> = c
                .get_matching()
                .into_iter()
                .enumerate()
                .map(|(i, j)| (i != j).then_some(j))
                .collect();
            assert_eq!(total(&m, &w), brute_force(n, &w));
        }
    }
}
