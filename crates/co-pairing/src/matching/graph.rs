//! Maximum-weight matching on general graphs: a port of bbpPairings'
//! `matching::detail` (Apache-2.0, Jeremy Bierema), itself based on Galil,
//! Micali and Gabow, "An O(EV log V) Algorithm for Finding a Maximal Weighted
//! Matching in General Graphs" (1986), extended to support updates.
//!
//! The port keeps the original's data structures, iteration orders and
//! tie-breaking, so that among several maximum matchings it returns the same
//! one. The Dutch engine's decisions depend on that. Pointers become indices:
//!
//! * Blossoms (vertices and parent blossoms) share one id space: ids below
//!   `cap` are vertices, the rest are parent blossoms.
//! * Root blossoms live in [`RootPool`], which reproduces the original
//!   `IterablePool`: slot reuse is LIFO and iteration follows construction
//!   order, skipping hidden or destroyed entries.

use super::wide::{self, Wide};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Label {
    Outer,
    Zero,
    Inner,
    Free,
}

/// Fields every blossom has.
#[derive(Debug, Clone, Copy)]
struct Node {
    root: usize,
    parent: Option<usize>,
    head: usize,
    tail: usize,
    to_prev_sibling: usize,
    to_next_sibling: usize,
    next_blossom: usize,
    prev_blossom: usize,
}

impl Node {
    fn new(root: usize, head: usize, tail: usize) -> Self {
        Self {
            root,
            parent: None,
            head,
            tail,
            to_prev_sibling: usize::MAX,
            to_next_sibling: usize::MAX,
            next_blossom: usize::MAX,
            prev_blossom: usize::MAX,
        }
    }
}

struct VertexData {
    /// Edge weights to every vertex (`n` × width limbs, flattened).
    edges: Vec<u64>,
    min_outer_res: Wide,
    min_outer_edge: Option<usize>,
    next_vertex: Option<usize>,
}

struct ParentData {
    dual: Wide,
    subblossom: usize,
    iteration_starts_with_subblossom: bool,
}

struct RootData {
    min_outer_edges: Vec<Option<usize>>,
    root_child: usize,
    base: usize,
    base_match: Option<usize>,
    label: Label,
    labeling: Option<usize>,
    labeled: Option<usize>,
}

/// The original `IterablePool<RootBlossom>`.
struct RootPool {
    slots: Vec<Option<RootData>>,
    forward: Vec<Option<usize>>,
    backward: Vec<Option<usize>>,
    head: Option<usize>,
    tail: Option<usize>,
    allocated_head: Option<usize>,
    unallocated_head: Option<usize>,
}

impl RootPool {
    fn new(size: usize) -> Self {
        let mut forward = vec![None; size];
        for i in 1..size {
            forward[i - 1] = Some(i);
        }
        Self {
            slots: (0..size).map(|_| None).collect(),
            forward,
            backward: vec![None; size],
            head: None,
            tail: None,
            allocated_head: None,
            unallocated_head: if size > 0 { Some(0) } else { None },
        }
    }

    /// Takes a free slot (the C++ `construct` before the constructor runs).
    fn alloc(&mut self) -> usize {
        let slot = self.unallocated_head.expect("root pool capacity");
        self.unallocated_head = self.forward[slot];
        slot
    }

    /// Links a constructed slot at the tail (the rest of `construct`).
    fn link(&mut self, slot: usize) {
        if let Some(t) = self.tail {
            self.forward[t] = Some(slot);
        }
        self.backward[slot] = self.tail;
        self.tail = Some(slot);
        self.forward[slot] = None;
        if self.head.is_none() {
            self.head = Some(slot);
        }
        if self.allocated_head.is_none() {
            self.allocated_head = Some(slot);
        }
    }

    fn unlink(&mut self, slot: usize) {
        let fwd = self.forward[slot];
        let bwd = self.backward[slot];
        match fwd {
            Some(f) => self.backward[f] = bwd,
            None => self.tail = bwd,
        }
        match bwd {
            Some(b) => self.forward[b] = fwd,
            None => self.allocated_head = fwd,
        }
        if self.head == Some(slot) {
            self.head = fwd;
        }
    }

    fn hide(&mut self, slot: usize) {
        self.unlink(slot);
        if let Some(ah) = self.allocated_head {
            self.backward[ah] = Some(slot);
        }
        self.forward[slot] = self.allocated_head;
        self.allocated_head = Some(slot);
        self.backward[slot] = None;
        if self.tail.is_none() {
            self.tail = self.allocated_head;
        }
    }

    fn destroy(&mut self, slot: usize) {
        self.unlink(slot);
        self.slots[slot] = None;
        self.forward[slot] = self.unallocated_head;
        self.unallocated_head = Some(slot);
    }

    fn first(&self) -> Option<usize> {
        self.head
    }

    fn next(&self, slot: usize) -> Option<usize> {
        self.forward[slot]
    }

    /// Visible slots in iteration order.
    fn visible(&self) -> Vec<usize> {
        let mut v = Vec::new();
        let mut it = self.head;
        while let Some(s) = it {
            v.push(s);
            it = self.forward[s];
        }
        v
    }
}

pub(crate) struct Graph {
    cap: usize,
    width: usize,
    n: usize,
    nodes: Vec<Node>,
    vertices: Vec<VertexData>,
    parents: Vec<Option<ParentData>>,
    parent_free: Vec<usize>,
    roots: RootPool,
    root_min_res: Vec<Wide>,
    /// Vertex dual variables, `cap` × `width` limbs.
    duals: Vec<u64>,
    above_max: Wide,
}

impl Graph {
    pub(crate) fn new(capacity: usize, max_edge_weight: &Wide) -> Self {
        let mut above_max = max_edge_weight.clone();
        above_max.shl(2);
        above_max.add_small(1);
        let parent_cap = (capacity / 2).max(1);
        Self {
            cap: capacity,
            width: max_edge_weight.width(),
            n: 0,
            nodes: Vec::with_capacity(capacity + parent_cap),
            vertices: Vec::with_capacity(capacity),
            parents: (0..parent_cap).map(|_| None).collect(),
            parent_free: (0..parent_cap).rev().collect(),
            roots: RootPool::new(capacity + 1),
            root_min_res: vec![max_edge_weight.clone(); capacity + 1],
            duals: vec![0u64; capacity * max_edge_weight.width()],
            above_max,
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.n
    }

    pub(crate) fn zero(&self) -> Wide {
        self.above_max.zero_like()
    }

    // ----- small accessors ---------------------------------------------

    fn is_vertex(&self, b: usize) -> bool {
        b < self.cap
    }

    fn node(&self, b: usize) -> &Node {
        &self.nodes[self.node_index(b)]
    }

    fn node_mut(&mut self, b: usize) -> &mut Node {
        let i = self.node_index(b);
        &mut self.nodes[i]
    }

    /// Vertices occupy `nodes[0..n)`; parent blossom `cap + p` is stored at
    /// `nodes[cap + p]`, so the vector is grown on demand.
    fn node_index(&self, b: usize) -> usize {
        b
    }

    fn ensure_node(&mut self, b: usize) {
        if self.nodes.len() <= b {
            let filler = Node::new(usize::MAX, usize::MAX, usize::MAX);
            self.nodes.resize(b + 1, filler);
        }
    }

    fn parent_data(&self, b: usize) -> &ParentData {
        self.parents[b - self.cap]
            .as_ref()
            .expect("live parent blossom")
    }

    fn parent_data_mut(&mut self, b: usize) -> &mut ParentData {
        self.parents[b - self.cap]
            .as_mut()
            .expect("live parent blossom")
    }

    fn root(&self, r: usize) -> &RootData {
        self.roots.slots[r].as_ref().expect("live root blossom")
    }

    fn root_mut(&mut self, r: usize) -> &mut RootData {
        self.roots.slots[r].as_mut().expect("live root blossom")
    }

    fn root_of_vertex(&self, v: usize) -> usize {
        self.nodes[v].root
    }

    fn dual(&self, v: usize) -> &[u64] {
        &self.duals[v * self.width..(v + 1) * self.width]
    }

    fn dual_mut(&mut self, v: usize) -> &mut [u64] {
        let w = self.width;
        &mut self.duals[v * w..(v + 1) * w]
    }

    fn edge(&self, a: usize, b: usize) -> &[u64] {
        &self.vertices[a].edges[b * self.width..(b + 1) * self.width]
    }

    /// `out = dual[a] + dual[b] - w(a, b)`.
    fn resistance_into(&self, out: &mut Wide, a: usize, b: usize) {
        wide::add_sub_into(out.limbs_mut(), self.dual(a), self.dual(b), self.edge(a, b));
    }

    fn resistance(&self, a: usize, b: usize) -> Wide {
        let mut out = self.zero();
        self.resistance_into(&mut out, a, b);
        out
    }

    fn vertex_list(&self, blossom: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut it = Some(self.node(blossom).head);
        while let Some(v) = it {
            out.push(v);
            it = self.vertices[v].next_vertex;
        }
        out
    }

    fn base_index(&self, r: usize) -> usize {
        self.root(r).base
    }

    // ----- public operations (matching::Computer) ----------------------

    pub(crate) fn add_vertex(&mut self) {
        let idx = self.n;
        assert!(idx < self.cap, "matching capacity exceeded");
        for slot in self.roots.visible() {
            self.root_mut(slot).min_outer_edges.push(None);
        }
        // Vertex construction: its RootBlossom is constructed first.
        let slot = self.roots.alloc();
        self.roots.slots[slot] = Some(RootData {
            min_outer_edges: vec![None; idx + 1],
            root_child: idx,
            base: idx,
            base_match: None,
            label: Label::Zero,
            labeling: None,
            labeled: None,
        });
        self.roots.link(slot);
        self.ensure_node(idx);
        self.nodes[idx] = Node::new(slot, idx, idx);
        self.vertices.push(VertexData {
            edges: Vec::new(),
            min_outer_res: self.above_max.clone(),
            min_outer_edge: None,
            next_vertex: None,
        });
        self.n += 1;
        let zero = vec![0u64; self.width];
        for v in 0..self.n {
            if v != idx {
                self.vertices[v].edges.extend_from_slice(&zero);
            }
            self.vertices[idx].edges.extend_from_slice(&zero);
        }
    }

    /// Sets the weight of edge (u, v); zero removes the edge.
    pub(crate) fn set_edge_weight(&mut self, u: usize, v: usize, weight: &Wide) {
        let mut w = weight.clone();
        w.shl(1);
        let r = self.root_of_vertex(u);
        self.prepare_vertex_for_weight_adjustments(r, u);
        let width = self.width;
        self.vertices[u].edges[v * width..(v + 1) * width].copy_from_slice(w.limbs());
        self.vertices[v].edges[u * width..(u + 1) * width].copy_from_slice(w.limbs());
    }

    pub(crate) fn compute_matching(&mut self) {
        self.compute_matching_inner();
        for slot in self.roots.visible() {
            self.put_vertices_in_matching_order(slot);
        }
    }

    /// For each vertex, its mate (itself if unmatched).
    pub(crate) fn get_matching(&self) -> Vec<usize> {
        let mut result = vec![0usize; self.n];
        for slot in self.roots.visible() {
            let r = self.root(slot);
            match r.base_match {
                Some(m) => result[r.base] = m,
                None => result[r.base] = r.base,
            }
            let mut it = self.vertices[self.node(r.root_child).head].next_vertex;
            while let Some(first) = it {
                let second = self.vertices[first].next_vertex.expect("matched pair");
                result[first] = second;
                result[second] = first;
                it = self.vertices[second].next_vertex;
            }
        }
        result
    }

    // ----- blossom helpers ----------------------------------------------

    fn ancestor_of_vertex(&self, vertex: usize, ancestor: Option<usize>) -> usize {
        let mut b = vertex;
        while self.node(b).parent != ancestor {
            b = self.node(b).parent.expect("ancestor reachable");
        }
        b
    }

    fn set_pointers_from_ancestor(&mut self, vertex: usize, ancestor: usize, flag: bool) {
        let mut b = vertex;
        while b != ancestor {
            let p = self.node(b).parent.expect("on path to ancestor");
            let pd = self.parent_data_mut(p);
            pd.subblossom = b;
            pd.iteration_starts_with_subblossom = flag;
            b = p;
        }
    }

    fn update_root_in_descendants(&mut self, of_root: usize, new_root: usize) {
        let mut it = Some(self.node(self.root(of_root).root_child).head);
        while let Some(v) = it {
            self.nodes[v].root = new_root;
            let mut parent = self.nodes[v].parent;
            while let Some(p) = parent {
                if self.node(p).tail != v {
                    break;
                }
                self.node_mut(p).root = new_root;
                parent = self.node(p).parent;
            }
            it = self.vertices[v].next_vertex;
        }
    }

    /// RootBlossom constructor for an existing child blossom.
    fn construct_root_from_child(
        &mut self,
        child: usize,
        base: usize,
        base_match: Option<usize>,
        label: Label,
        labeling: Option<usize>,
        labeled: Option<usize>,
    ) -> usize {
        let slot = self.roots.alloc();
        let moe_len = self.root(self.node(child).root).min_outer_edges.len();
        self.roots.slots[slot] = Some(RootData {
            min_outer_edges: vec![None; moe_len],
            root_child: child,
            base,
            base_match,
            label,
            labeling,
            labeled,
        });
        self.node_mut(child).parent = None;
        let tail = self.node(child).tail;
        self.vertices[tail].next_vertex = None;
        self.update_root_in_descendants(slot, slot);
        self.roots.link(slot);
        slot
    }

    /// RootBlossom constructor from an odd cycle of blossoms (`path`).
    fn construct_root_from_path(&mut self, path: &[usize]) -> usize {
        let slot = self.roots.alloc();
        // ParentBlossom constructor.
        let pid = self.parent_free.pop().unwrap_or_else(|| {
            self.parents.push(None);
            self.parents.len() - 1
        });
        let pb = self.cap + pid;
        self.ensure_node(pb);
        let head = self
            .node(
                self.root(self.root_of_vertex(path[path.len() - 2]))
                    .root_child,
            )
            .head;
        let tail = self
            .node(self.root(self.root_of_vertex(path[0])).root_child)
            .tail;
        self.nodes[pb] = Node::new(slot, head, tail);
        self.parents[pid] = Some(ParentData {
            dual: self.zero(),
            subblossom: usize::MAX,
            iteration_starts_with_subblossom: false,
        });
        // connectChildren
        let mut prev = self.ancestor_of_vertex(path[0], None);
        let mut i = 0;
        let mut sub = prev;
        while i < path.len() {
            self.node_mut(prev).to_next_sibling = path[i];
            i += 1;
            sub = self.ancestor_of_vertex(path[i], None);
            self.node_mut(prev).next_blossom = sub;
            let n = self.node_mut(sub);
            n.to_prev_sibling = path[i];
            n.parent = Some(pb);
            n.prev_blossom = prev;
            prev = sub;
            i += 1;
        }
        self.parent_data_mut(pb).subblossom = sub;

        let originals: Vec<usize> = path
            .iter()
            .step_by(2)
            .map(|&v| self.root_of_vertex(v))
            .collect();
        let base_root = self.root_of_vertex(path[0]);
        let b = self.root(base_root);
        let data = RootData {
            min_outer_edges: b.min_outer_edges.clone(),
            root_child: pb,
            base: b.base,
            base_match: b.base_match,
            label: b.label,
            labeling: b.labeling,
            labeled: b.labeled,
        };
        self.roots.slots[slot] = Some(data);
        self.initialize_from_children(slot, &originals);
        self.roots.link(slot);
        slot
    }

    fn initialize_from_children(&mut self, this: usize, originals: &[usize]) {
        self.root_min_res[this] = self.above_max.clone();
        for &r in originals {
            self.roots.hide(r);
            self.update_root_in_descendants(r, this);
        }
        let mut storage = self.above_max.clone();
        let mut min_res = self.above_max.clone();
        let mut it = self.roots.first();
        while let Some(other) = it {
            if self.root(other).label == Label::Outer {
                min_res.assign(self.above_max.limbs());
                for &b in originals {
                    if self.root(b).label == Label::Inner {
                        self.update_outer_outer_edges(b, other, &mut min_res, &mut storage);
                    } else {
                        let other_base = self.base_index(other);
                        let b_base = self.base_index(b);
                        let e0 = self.root(b).min_outer_edges[other_base];
                        let e1 = self.root(other).min_outer_edges[b_base];
                        if let (Some(e0), Some(e1)) = (e0, e1) {
                            let res = self.resistance(e0, e1);
                            if res < min_res {
                                min_res = res;
                                let this_base = self.base_index(this);
                                self.root_mut(this).min_outer_edges[other_base] = Some(e0);
                                self.root_mut(other).min_outer_edges[this_base] = Some(e1);
                                if min_res < self.root_min_res[this] {
                                    self.root_min_res[this] = min_res.clone();
                                }
                                if min_res < self.root_min_res[other] {
                                    self.root_min_res[other] = min_res.clone();
                                }
                            }
                        }
                    }
                }
            }
            it = self.roots.next(other);
        }
        let mut previous_head: Option<usize> = None;
        for &r in originals {
            if self.root(r).label != Label::Outer {
                self.update_inner_outer_edges(r);
            }
            let child = self.root(r).root_child;
            let tail = self.node(child).tail;
            self.vertices[tail].next_vertex = previous_head;
            previous_head = Some(self.node(child).head);
            self.roots.destroy(r);
        }
    }

    fn free_ancestor_of_base(&mut self, this: usize, ancestor: usize) {
        let root_child = self.root(this).root_child;
        if ancestor == root_child {
            return;
        }
        let mut adj = self.zero();
        let mut p = self.node(ancestor).parent;
        while let Some(b) = p {
            let mut half = self.parent_data(b).dual.clone();
            half.shr(1);
            adj.add(half.limbs());
            p = self.node(b).parent;
        }
        let mut blossom = self.node(ancestor).parent;
        let mut next_blossom = self.node(ancestor).next_blossom;
        let base = self.root(this).base;
        let base_match = self.root(this).base_match;
        self.construct_root_from_child(ancestor, base, base_match, Label::Zero, None, None);
        for v in self.vertex_list(ancestor) {
            wide::add(self.dual_mut(v), adj.limbs());
        }
        let mut child_to_free = ancestor;
        while let Some(b) = blossom {
            let mut links_forward = true;
            let mut previous = usize::MAX;
            let mut current = next_blossom;
            while current != child_to_free {
                next_blossom = self.node(current).next_blossom;
                let (vbase, vmatch) = if links_forward {
                    (
                        self.node(current).to_next_sibling,
                        self.node(next_blossom).to_prev_sibling,
                    )
                } else {
                    (
                        self.node(current).to_prev_sibling,
                        self.node(previous).to_next_sibling,
                    )
                };
                self.construct_root_from_child(
                    current,
                    vbase,
                    Some(vmatch),
                    Label::Zero,
                    None,
                    None,
                );
                for v in self.vertex_list(current) {
                    wide::add(self.dual_mut(v), adj.limbs());
                }
                links_forward = !links_forward;
                previous = current;
                current = next_blossom;
            }
            let mut half = self.parent_data(b).dual.clone();
            half.shr(1);
            adj.sub(half.limbs());
            if child_to_free != ancestor {
                self.destroy_parent(child_to_free);
            }
            child_to_free = b;
            next_blossom = self.node(b).next_blossom;
            blossom = self.node(b).parent;
        }
        self.destroy_parent(root_child);
        self.roots.destroy(this);
    }

    fn destroy_parent(&mut self, b: usize) {
        let pid = b - self.cap;
        self.parents[pid] = None;
        self.parent_free.push(pid);
    }

    fn prepare_vertex_for_weight_adjustments(&mut self, this: usize, vertex: usize) {
        if let Some(m) = self.root(this).base_match {
            let mr = self.root_of_vertex(m);
            self.root_mut(mr).base_match = None;
            self.root_mut(this).base_match = None;
        }
        self.root_mut(this).base = vertex;
        self.free_ancestor_of_base(this, vertex);
        let mut d = self.above_max.clone();
        d.shr(1);
        self.dual_mut(vertex).copy_from_slice(d.limbs());
    }

    fn put_vertices_in_matching_order(&mut self, root: usize) {
        let root_child = self.root(root).root_child;
        let mut current_blossom = root_child;
        let mut current_vertex = self.root(root).base;
        let mut starts_with_base = true;
        loop {
            self.set_pointers_from_ancestor(current_vertex, current_blossom, starts_with_base);
            current_blossom = current_vertex;
            while current_blossom != root_child {
                let parent = self.node(current_blossom).parent.expect("inside root");
                starts_with_base =
                    !starts_with_base && self.parent_data(parent).subblossom != current_blossom;
                let prev = self.node(current_blossom).prev_blossom;
                let prev_tail = self.node(prev).tail;
                self.vertices[prev_tail].next_vertex = Some(self.node(current_blossom).head);
                current_blossom = self.node(current_blossom).next_blossom;
                let parent = self.node(current_blossom).parent.expect("inside root");
                if current_blossom == self.parent_data(parent).subblossom {
                    starts_with_base = self.parent_data(parent).iteration_starts_with_subblossom;
                    let prev = self.node(current_blossom).prev_blossom;
                    let prev_tail = self.node(prev).tail;
                    self.vertices[prev_tail].next_vertex = Some(self.node(current_blossom).head);
                    let head_from = if starts_with_base {
                        current_blossom
                    } else {
                        self.node(current_blossom).next_blossom
                    };
                    let tail_from = if starts_with_base {
                        self.node(current_blossom).prev_blossom
                    } else {
                        current_blossom
                    };
                    let head = self.node(head_from).head;
                    let tail = self.node(tail_from).tail;
                    let pn = self.node_mut(parent);
                    pn.head = head;
                    pn.tail = tail;
                    current_blossom = parent;
                } else {
                    current_vertex = if starts_with_base {
                        self.node(current_blossom).to_prev_sibling
                    } else {
                        self.node(current_blossom).to_next_sibling
                    };
                    break;
                }
            }
            if current_blossom == root_child {
                break;
            }
        }
        let tail = self.node(root_child).tail;
        self.vertices[tail].next_vertex = None;
    }

    // ----- augmentation -------------------------------------------------

    fn update_outer_outer_edges(
        &mut self,
        r0: usize,
        r1: usize,
        min_res: &mut Wide,
        storage: &mut Wide,
    ) {
        let child0 = self.root(r0).root_child;
        let child1 = self.root(r1).root_child;
        let a0 = self.node(child0).root;
        let a1 = self.node(child1).root;
        let head1 = self.node(child1).head;
        let mut it0 = Some(self.node(child0).head);
        while let Some(v0) = it0 {
            let mut it1 = Some(head1);
            while let Some(v1) = it1 {
                self.resistance_into(storage, v0, v1);
                if *storage < *min_res {
                    min_res.assign(storage.limbs());
                    let b1 = self.base_index(a1);
                    let b0 = self.base_index(a0);
                    self.root_mut(a0).min_outer_edges[b1] = Some(v0);
                    self.root_mut(a1).min_outer_edges[b0] = Some(v1);
                    if *min_res < self.root_min_res[a0] {
                        self.root_min_res[a0].assign(min_res.limbs());
                    }
                    if *min_res < self.root_min_res[a1] {
                        self.root_min_res[a1].assign(min_res.limbs());
                    }
                }
                it1 = self.vertices[v1].next_vertex;
            }
            it0 = self.vertices[v0].next_vertex;
        }
    }

    fn initialize_labeling(&mut self) {
        let mut it = self.roots.first();
        while let Some(slot) = it {
            it = self.roots.next(slot);
            let label = {
                let r = self.root(slot);
                if r.base_match.is_some() {
                    Label::Free
                } else if !wide::is_zero(self.dual(r.base)) {
                    Label::Outer
                } else {
                    Label::Zero
                }
            };
            let r = self.root_mut(slot);
            r.label = label;
            r.labeled = None;
            r.labeling = None;
        }
    }

    fn label_of_vertex(&self, v: usize) -> Label {
        self.root(self.root_of_vertex(v)).label
    }

    fn update_inner_outer_edge(&mut self, inner: usize, outer: usize, storage: &mut Wide) {
        self.resistance_into(storage, outer, inner);
        if *storage < self.vertices[inner].min_outer_res {
            self.vertices[inner].min_outer_res.assign(storage.limbs());
            self.vertices[inner].min_outer_edge = Some(outer);
        }
    }

    fn update_inner_outer_edges(&mut self, outer_root: usize) {
        let mut storage = self.above_max.clone();
        let head = self.node(self.root(outer_root).root_child).head;
        for v in 0..self.n {
            if self.label_of_vertex(v) != Label::Outer {
                let mut it = Some(head);
                while let Some(o) = it {
                    self.update_inner_outer_edge(v, o, &mut storage);
                    it = self.vertices[o].next_vertex;
                }
            }
        }
    }

    fn initialize_inner_outer_edges(&mut self) {
        let outer: Vec<usize> = (0..self.n)
            .filter(|&v| self.label_of_vertex(v) == Label::Outer)
            .collect();
        let mut storage = self.above_max.clone();
        for v in 0..self.n {
            if self.label_of_vertex(v) != Label::Outer {
                let above = self.above_max.limbs().to_vec();
                self.vertices[v].min_outer_res.assign(&above);
                for &o in &outer {
                    self.update_inner_outer_edge(v, o, &mut storage);
                }
            }
        }
    }

    fn initialize_outer_outer_edges_of(
        &mut self,
        b0: usize,
        storage0: &mut Wide,
        storage1: &mut Wide,
    ) {
        self.root_min_res[b0] = self.above_max.clone();
        let mut it = self.roots.first();
        while let Some(other) = it {
            storage0.assign(self.above_max.limbs());
            if self.root(other).label == Label::Outer && other != b0 {
                let ob = self.base_index(other);
                self.root_mut(b0).min_outer_edges[ob] = None;
                self.update_outer_outer_edges(b0, other, storage0, storage1);
            }
            it = self.roots.next(other);
        }
    }

    fn initialize_outer_outer_edges_one(&mut self, b0: usize) {
        let mut s0 = self.above_max.clone();
        let mut s1 = self.above_max.clone();
        self.initialize_outer_outer_edges_of(b0, &mut s0, &mut s1);
    }

    fn initialize_outer_outer_edges_all(&mut self) {
        let mut s0 = self.above_max.clone();
        let mut s1 = self.above_max.clone();
        let mut it = self.roots.first();
        while let Some(r) = it {
            if self.root(r).label == Label::Outer {
                self.initialize_outer_outer_edges_of(r, &mut s0, &mut s1);
            }
            it = self.roots.next(r);
        }
    }

    fn initialize_min_outer_outer(&self) -> (Option<usize>, Wide) {
        let mut min = self.above_max.clone();
        let mut min_root = None;
        let mut it = self.roots.first();
        while let Some(slot) = it {
            if self.root(slot).label == Label::Outer && self.root_min_res[slot] < min {
                min.assign(self.root_min_res[slot].limbs());
                min_root = Some(slot);
            }
            it = self.roots.next(slot);
        }
        (min_root, min)
    }

    fn update_min_outer_outer(&self, r: usize, min_root: &mut Option<usize>, min: &mut Wide) {
        if self.root_min_res[r] < *min {
            min.assign(self.root_min_res[r].limbs());
            *min_root = Some(r);
        }
    }

    fn update_min_inner_dual(&self, r: usize, min_blossom: &mut Option<usize>, min: &mut Wide) {
        let child = self.root(r).root_child;
        if !self.is_vertex(child) && self.parent_data(child).dual < *min {
            min.assign(self.parent_data(child).dual.limbs());
            *min_blossom = Some(child);
        }
    }

    fn initialize_min_inner_dual(&self) -> (Option<usize>, Wide) {
        let mut min = self.above_max.clone();
        let mut blossom = None;
        let mut it = self.roots.first();
        while let Some(slot) = it {
            if self.root(slot).label == Label::Inner {
                self.update_min_inner_dual(slot, &mut blossom, &mut min);
            }
            it = self.roots.next(slot);
        }
        (blossom, min)
    }

    fn update_min_outer_dual(&self, v: usize, min_vertex: &mut Option<usize>, min: &mut Wide) {
        if wide::lt(self.dual(v), min.limbs()) {
            *min_vertex = Some(v);
            let d = self.dual(v).to_vec();
            min.assign(&d);
        }
    }

    fn augment_to_source(&mut self, vertex: usize, new_match: Option<usize>) {
        let mut vertex = vertex;
        let mut new_match = new_match;
        loop {
            let r = self.root_of_vertex(vertex);
            let Some(old) = self.root(r).base_match else {
                break;
            };
            self.root_mut(r).base = vertex;
            let original = self.root_of_vertex(old);
            self.root_mut(r).base_match = new_match;
            let labeled = self.root(original).labeled;
            let labeling = self.root(original).labeling;
            let om = self.root_mut(original);
            om.base = labeled.expect("labeled vertex");
            om.base_match = labeling;
            vertex = labeling.expect("labeling vertex");
            new_match = labeled;
        }
        let r = self.root_of_vertex(vertex);
        let rm = self.root_mut(r);
        rm.base = vertex;
        rm.base_match = new_match;
    }

    fn augment_matching(&mut self) -> bool {
        self.initialize_labeling();

        let mut min_outer_dual = self.above_max.clone();
        let mut min_outer_dual_vertex: Option<usize> = None;
        for v in 0..self.n {
            if self.label_of_vertex(v) == Label::Outer {
                self.update_min_outer_dual(v, &mut min_outer_dual_vertex, &mut min_outer_dual);
            }
        }
        if min_outer_dual_vertex.is_none() {
            return false;
        }

        self.initialize_inner_outer_edges();
        self.initialize_outer_outer_edges_all();

        let (mut min_oo_root, mut min_oo) = self.initialize_min_outer_outer();
        let mut min_inner_dual = self.above_max.clone();
        let mut min_inner_dual_blossom: Option<usize> = None;

        loop {
            let mut min_io = self.above_max.clone();
            let mut min_io_vertex: Option<usize> = None;
            for v in 0..self.n {
                let l = self.label_of_vertex(v);
                if (l == Label::Free || l == Label::Zero) && self.vertices[v].min_outer_res < min_io
                {
                    min_io.assign(self.vertices[v].min_outer_res.limbs());
                    min_io_vertex = Some(v);
                }
            }

            let mut half_oo = min_oo.clone();
            half_oo.shr(1);
            let mut half_inner = min_inner_dual.clone();
            half_inner.shr(1);
            let mut adj = min_outer_dual.clone();
            for cand in [&min_io, &half_oo, &half_inner] {
                if *cand < adj {
                    adj = cand.clone();
                }
            }
            if !adj.is_zero() {
                let mut twice = adj.clone();
                twice.shl(1);
                min_outer_dual.sub(adj.limbs());
                min_io.sub(adj.limbs());
                min_oo.sub(twice.limbs());
                min_inner_dual.sub(twice.limbs());
                for v in 0..self.n {
                    let r = self.root_of_vertex(v);
                    let label = self.root(r).label;
                    match label {
                        Label::Outer => wide::sub(self.dual_mut(v), adj.limbs()),
                        Label::Inner => wide::add(self.dual_mut(v), adj.limbs()),
                        _ => {
                            if self.vertices[v].min_outer_res < self.above_max {
                                self.vertices[v].min_outer_res.sub(adj.limbs());
                            }
                        }
                    }
                    if self.root(r).base == v {
                        let child = self.root(r).root_child;
                        if label == Label::Outer {
                            if self.root_min_res[r] < self.above_max {
                                self.root_min_res[r].sub(twice.limbs());
                            }
                            if !self.is_vertex(child) {
                                self.parent_data_mut(child).dual.add(twice.limbs());
                            }
                        } else if label == Label::Inner && !self.is_vertex(child) {
                            self.parent_data_mut(child).dual.sub(twice.limbs());
                        }
                    }
                }
            }

            if min_outer_dual.is_zero() {
                self.augment_to_source(min_outer_dual_vertex.expect("set"), None);
                return true;
            }
            if min_io.is_zero() && self.label_of_vertex(min_io_vertex.expect("set")) == Label::Zero
            {
                let v = min_io_vertex.expect("set");
                let other = self.vertices[v].min_outer_edge.expect("min outer edge");
                self.augment_to_source(other, Some(v));
                self.augment_to_source(v, Some(other));
                return true;
            } else if min_oo.is_zero() {
                let min_root = min_oo_root.expect("set");
                let (vertex0, vertex1);
                let mut it = self.roots.first();
                loop {
                    let r = it.expect("zero-resistance outer edge exists");
                    if self.root(r).label == Label::Outer && r != min_root {
                        let v0 = self.root(min_root).min_outer_edges[self.base_index(r)];
                        let v1 = self.root(r).min_outer_edges[self.base_index(min_root)];
                        if let (Some(a), Some(b)) = (v0, v1)
                            && self.resistance(a, b).is_zero()
                        {
                            vertex0 = a;
                            vertex1 = b;
                            break;
                        }
                    }
                    it = self.roots.next(r);
                }

                let mut path: VecDeque<usize> = VecDeque::new();
                path.push_front(vertex0);
                path.push_back(vertex1);
                loop {
                    let front = *path.front().expect("non-empty");
                    let r = self.root_of_vertex(front);
                    if self.root(r).base_match.is_none() {
                        break;
                    }
                    let base = self.root(r).base;
                    path.push_front(base);
                    let m = self
                        .root(self.root_of_vertex(base))
                        .base_match
                        .expect("matched");
                    path.push_front(m);
                    let lr = self.root_of_vertex(m);
                    let labeled = self.root(lr).labeled.expect("labeled");
                    path.push_front(labeled);
                    let labeling = self
                        .root(self.root_of_vertex(labeled))
                        .labeling
                        .expect("labeling");
                    path.push_front(labeling);
                }
                loop {
                    let back = *path.back().expect("non-empty");
                    let r = self.root_of_vertex(back);
                    if self.root(r).base_match.is_none() {
                        break;
                    }
                    let base = self.root(r).base;
                    path.push_back(base);
                    let m = self
                        .root(self.root_of_vertex(base))
                        .base_match
                        .expect("matched");
                    path.push_back(m);
                    let lr = self.root_of_vertex(m);
                    let labeled = self.root(lr).labeled.expect("labeled");
                    path.push_back(labeled);
                    let labeling = self
                        .root(self.root_of_vertex(labeled))
                        .labeling
                        .expect("labeling");
                    path.push_back(labeling);
                }

                let front_root = self.root_of_vertex(*path.front().unwrap());
                let back_root = self.root_of_vertex(*path.back().unwrap());
                if front_root == back_root {
                    while self.root_of_vertex(path[1]) == self.root_of_vertex(path[path.len() - 2])
                    {
                        for _ in 0..4 {
                            path.pop_front();
                            path.pop_back();
                        }
                    }
                    let path: Vec<usize> = path.into_iter().collect();
                    let new_root = self.construct_root_from_path(&path);
                    for v in self.vertex_list(self.root(new_root).root_child) {
                        self.update_min_outer_dual(
                            v,
                            &mut min_outer_dual_vertex,
                            &mut min_outer_dual,
                        );
                    }
                    let (r, m) = self.initialize_min_outer_outer();
                    min_oo_root = r;
                    min_oo = m;
                    let (b, m) = self.initialize_min_inner_dual();
                    min_inner_dual_blossom = b;
                    min_inner_dual = m;
                } else {
                    self.augment_to_source(vertex0, Some(vertex1));
                    self.augment_to_source(vertex1, Some(vertex0));
                    return true;
                }
            } else if min_io.is_zero() {
                let v = min_io_vertex.expect("set");
                let vr = self.root_of_vertex(v);
                let matched_root =
                    self.root_of_vertex(self.root(vr).base_match.expect("free root is matched"));
                self.root_mut(vr).label = Label::Inner;
                self.root_mut(matched_root).label = Label::Outer;
                let edge = self.vertices[v].min_outer_edge;
                self.root_mut(vr).labeling = edge;
                self.root_mut(vr).labeled = Some(v);
                self.update_inner_outer_edges(matched_root);
                self.initialize_outer_outer_edges_one(matched_root);
                for u in self.vertex_list(self.root(matched_root).root_child) {
                    self.update_min_outer_dual(u, &mut min_outer_dual_vertex, &mut min_outer_dual);
                }
                self.update_min_outer_outer(matched_root, &mut min_oo_root, &mut min_oo);
                let vr = self.root_of_vertex(v);
                self.update_min_inner_dual(vr, &mut min_inner_dual_blossom, &mut min_inner_dual);
                continue;
            } else if min_inner_dual.is_zero() {
                let blossom = min_inner_dual_blossom.expect("set");
                let old_root = self.node(blossom).root;
                self.roots.hide(old_root);
                let root_vertex = self.root(old_root).base;
                let root_child = self.ancestor_of_vertex(root_vertex, Some(blossom));
                let labeled_vertex = self.root(old_root).labeled.expect("inner is labeled");
                let connect_child = self.ancestor_of_vertex(labeled_vertex, Some(blossom));
                let mut connect_forward = true;
                let mut c = root_child;
                while c != connect_child {
                    connect_forward = !connect_forward;
                    c = self.node(c).next_blossom;
                }
                let old_match = self.root(old_root).base_match;
                let old_labeling = self.root(old_root).labeling;
                let old_labeled = self.root(old_root).labeled;

                let mut links_to_next = false;
                let mut is_free = false;
                let mut previous_child = self.node(root_child).prev_blossom;
                let mut current = root_child;
                let mut next_child: Option<usize> = None;
                while next_child != Some(root_child) {
                    let nc = self.node(current).next_blossom;
                    next_child = Some(nc);
                    if current == connect_child && !connect_forward {
                        is_free = false;
                    }
                    let label = if is_free {
                        Label::Free
                    } else if (links_to_next ^ connect_forward) || root_child == current {
                        Label::Inner
                    } else {
                        Label::Outer
                    };
                    let base = if root_child == current {
                        root_vertex
                    } else if links_to_next {
                        self.node(current).to_next_sibling
                    } else {
                        self.node(current).to_prev_sibling
                    };
                    let base_match = if root_child == current {
                        old_match
                    } else if links_to_next {
                        Some(self.node(nc).to_prev_sibling)
                    } else {
                        Some(self.node(previous_child).to_next_sibling)
                    };
                    let labeling = if connect_child == current {
                        old_labeling
                    } else if label == Label::Inner {
                        if connect_forward {
                            Some(self.node(nc).to_prev_sibling)
                        } else {
                            Some(self.node(previous_child).to_next_sibling)
                        }
                    } else {
                        None
                    };
                    let labeled = if connect_child == current {
                        old_labeled
                    } else if label == Label::Inner {
                        if connect_forward {
                            Some(self.node(current).to_next_sibling)
                        } else {
                            Some(self.node(current).to_prev_sibling)
                        }
                    } else {
                        None
                    };
                    let new_root = self.construct_root_from_child(
                        current, base, base_match, label, labeling, labeled,
                    );
                    if label == Label::Outer {
                        self.update_inner_outer_edges(new_root);
                        self.initialize_outer_outer_edges_one(new_root);
                        for u in self.vertex_list(current) {
                            self.update_min_outer_dual(
                                u,
                                &mut min_outer_dual_vertex,
                                &mut min_outer_dual,
                            );
                        }
                        self.update_min_outer_outer(new_root, &mut min_oo_root, &mut min_oo);
                    }
                    if current
                        == (if connect_forward {
                            connect_child
                        } else {
                            root_child
                        })
                    {
                        is_free = true;
                    }
                    links_to_next = !links_to_next;
                    previous_child = current;
                    current = nc;
                }
                self.roots.destroy(old_root);
                self.destroy_parent(blossom);
                let (b, m) = self.initialize_min_inner_dual();
                min_inner_dual_blossom = b;
                min_inner_dual = m;
            }
        }
    }

    fn compute_matching_inner(&mut self) {
        // Make all exposed vertex duals the same parity.
        let mut it = self.roots.first();
        while let Some(r) = it {
            it = self.roots.next(r);
            let base = self.root(r).base;
            if self.root(r).base_match.is_none() && self.dual(base)[0] & 1 == 1 {
                let mut adjustable = self.root(r).root_child;
                self.set_pointers_from_ancestor(base, adjustable, true);
                while !self.is_vertex(adjustable) && self.parent_data(adjustable).dual.is_zero() {
                    adjustable = self.parent_data(adjustable).subblossom;
                }
                self.free_ancestor_of_base(r, adjustable);
                if !self.is_vertex(adjustable) {
                    self.parent_data_mut(adjustable).dual.sub_small(2);
                }
                for v in self.vertex_list(adjustable) {
                    wide::add_small(self.dual_mut(v), 1);
                }
            }
        }
        while self.augment_matching() {}
    }
}
