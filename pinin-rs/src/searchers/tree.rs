use std::collections::{BTreeSet, HashMap};

use crate::elements::CharData;
use crate::searchers::{Logic, Searcher};
use crate::utils::{Accelerator, Compressor, IndexSet};
use crate::PinIn;

/// Maximum number of (offset, id) slots in NDense before it is promoted to NSlice.
/// Matches THRESHOLD = 128 in the Java source (data list length, i.e. 64 pairs).
const THRESHOLD: usize = 128;

/// Number of distinct children in NMap before it is promoted to NAcc.
const ACC_THRESHOLD: usize = 32;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Collect all set bits of an IndexSet into a Vec (avoids closure-capture issues
/// with `&mut Accelerator` during recursion).
#[inline]
fn collect_indices(is: IndexSet) -> Vec<u32> {
    let mut v = Vec::new();
    is.foreach(|i| v.push(i));
    v
}

// ── NDense ────────────────────────────────────────────────────────────────────

/// Flat list of (compressor_offset, object_id) pairs.  Cheap for small sets;
/// converted to NSlice once it exceeds THRESHOLD entries.
struct NDense {
    /// Interleaved: [offset0, id0, offset1, id1, …]
    data: Vec<usize>,
}

impl NDense {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn get_offset(
        &self,
        strs: &Compressor,
        acc: &mut Accelerator,
        search_str: &str,
        logic: Logic,
        ret: &mut BTreeSet<usize>,
        offset: usize,
    ) {
        let full = logic == Logic::Equal;
        // For non-EQUAL: if the query is fully consumed, every entry is a valid
        // prefix-match result.
        if !full && offset == search_str.len() {
            self.get_all(ret);
            return;
        }
        let n = self.data.len() / 2;
        for i in 0..n {
            let ch = self.data[i * 2];
            let matched = if full {
                acc.matches_in(offset, ch, strs)
            } else {
                acc.begins_in(offset, ch, strs)
            };
            if matched {
                ret.insert(self.data[i * 2 + 1]);
            }
        }
    }

    fn get_all(&self, ret: &mut BTreeSet<usize>) {
        let n = self.data.len() / 2;
        for i in 0..n {
            ret.insert(self.data[i * 2 + 1]);
        }
    }

    /// Byte length of the common prefix shared by all stored offsets.
    fn common_prefix(&self, strs: &Compressor) -> usize {
        if self.data.is_empty() {
            return 0;
        }
        let mut offset = 0;
        loop {
            let base = self.data[0] + offset;
            if strs.end(base) {
                return offset;
            }
            let a = strs.get(base);
            let consumed = strs.char_len(base);
            let n = self.data.len() / 2;
            for j in 1..n {
                let other = self.data[j * 2] + offset;
                if strs.end(other) || strs.get(other) != a {
                    return offset;
                }
            }
            offset += consumed;
        }
    }

    fn put(
        mut self,
        strs: &Compressor,
        acc: &Accelerator,
        name: usize,
        id: usize,
        context: &PinIn,
    ) -> Node {
        if self.data.len() >= THRESHOLD {
            // Promote to NSlice over the common prefix, then re-insert everything.
            let pattern = self.data[0];
            let prefix_len = self.common_prefix(strs);
            let mut root = Node::Slice(NSlice::new(pattern, pattern + prefix_len));
            let n = self.data.len() / 2;
            for i in 0..n {
                let old_name = self.data[i * 2];
                let old_id = self.data[i * 2 + 1];
                root = root.put(strs, acc, old_name, old_id, context);
            }
            root.put(strs, acc, name, id, context)
        } else {
            self.data.push(name);
            self.data.push(id);
            Node::Dense(self)
        }
    }
}

// ── NMap ──────────────────────────────────────────────────────────────────────

/// Trie node that maps each child code-point to a sub-Node.
struct NMap {
    children: HashMap<u32, Box<Node>>,
    /// Object ids for strings that end exactly at this node.
    leaves: Vec<usize>,
}

impl NMap {
    fn new() -> Self {
        Self { children: HashMap::new(), leaves: Vec::new() }
    }

    fn get_offset(
        &self,
        strs: &Compressor,
        acc: &mut Accelerator,
        search_str: &str,
        context: &PinIn,
        logic: Logic,
        ret: &mut BTreeSet<usize>,
        offset: usize,
    ) {
        if offset == search_str.len() {
            if logic == Logic::Equal {
                ret.extend(self.leaves.iter().copied());
            } else {
                self.get_all(ret);
            }
            return;
        }
        for (&cp, child) in &self.children {
            let matched = acc.get(cp, offset);
            let indices = collect_indices(matched);
            for i in indices {
                child.get_offset(
                    strs, acc, search_str, context, logic, ret,
                    offset + i as usize,
                );
            }
        }
    }

    fn get_all(&self, ret: &mut BTreeSet<usize>) {
        ret.extend(self.leaves.iter().copied());
        for child in self.children.values() {
            child.get_all(ret);
        }
    }

    /// Insert (name, id) into this map.  Returns `Some(cp)` if a brand-new child
    /// was added (used by NAcc to update its phoneme index), `None` otherwise.
    fn put_inner(
        &mut self,
        strs: &Compressor,
        acc: &Accelerator,
        name: usize,
        id: usize,
        context: &PinIn,
    ) -> Option<u32> {
        if strs.end(name) {
            self.leaves.push(id);
            return None;
        }
        let cp = strs.get(name);
        let consumed = strs.char_len(name);
        let is_new = !self.children.contains_key(&cp);
        // Move the existing child out (or create a fresh NDense), transform it,
        // then store the result back.
        let child =
            *self.children.remove(&cp).unwrap_or_else(|| Box::new(Node::Dense(NDense::new())));
        let new_child = child.put(strs, acc, name + consumed, id, context);
        self.children.insert(cp, Box::new(new_child));
        if is_new { Some(cp) } else { None }
    }

    fn put(
        mut self,
        strs: &Compressor,
        acc: &Accelerator,
        name: usize,
        id: usize,
        context: &PinIn,
    ) -> Node {
        self.put_inner(strs, acc, name, id, context);
        if self.children.len() > ACC_THRESHOLD {
            Node::Acc(NAcc::from_map(self, context))
        } else {
            Node::Map(self)
        }
    }
}

// ── NSlice ────────────────────────────────────────────────────────────────────

/// Compressed trie edge: a shared infix `strs[start..end]` with a single exit
/// node.  Any divergence causes the slice to be split via `cut()`.
struct NSlice {
    start: usize,
    end: usize,
    exit: Box<Node>,
}

impl NSlice {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end, exit: Box::new(Node::Map(NMap::new())) }
    }

    fn get_offset(
        &self,
        strs: &Compressor,
        acc: &mut Accelerator,
        search_str: &str,
        context: &PinIn,
        logic: Logic,
        ret: &mut BTreeSet<usize>,
        offset: usize,
    ) {
        self.get_inner(strs, acc, search_str, context, logic, ret, offset, 0);
    }

    /// `pos` is the byte position within the slice (0 … end-start).
    fn get_inner(
        &self,
        strs: &Compressor,
        acc: &mut Accelerator,
        search_str: &str,
        context: &PinIn,
        logic: Logic,
        ret: &mut BTreeSet<usize>,
        offset: usize,
        pos: usize,
    ) {
        if self.start + pos == self.end {
            // Consumed the whole slice; delegate to the exit node.
            self.exit.get_offset(strs, acc, search_str, context, logic, ret, offset);
        } else if offset == search_str.len() {
            // Query exhausted before slice: valid only for prefix/contain match.
            if logic != Logic::Equal {
                self.exit.get_all(ret);
            }
        } else {
            let cp = strs.get(self.start + pos);
            let cp_len = strs.char_len(self.start + pos);
            let matched = acc.get(cp, offset);
            let indices = collect_indices(matched);
            for i in indices {
                self.get_inner(
                    strs, acc, search_str, context, logic, ret,
                    offset + i as usize,
                    pos + cp_len,
                );
            }
        }
    }

    fn get_all(&self, ret: &mut BTreeSet<usize>) {
        self.exit.get_all(ret);
    }

    /// Split the slice at `offset` (an absolute compressor position within
    /// `self.start..self.end`), inserting a new NMap branch point.
    fn cut(&mut self, strs: &Compressor, offset: usize) {
        let cp = strs.get(offset);
        let consumed = strs.char_len(offset);
        // Temporarily swap out the old exit.
        let old_exit =
            std::mem::replace(&mut self.exit, Box::new(Node::Map(NMap::new())));
        let mut branch = NMap::new();
        if offset + consumed == self.end {
            // The remainder of the old slice is exactly one char; make it a leaf.
            branch.children.insert(cp, old_exit);
        } else {
            let half = NSlice { start: offset + consumed, end: self.end, exit: old_exit };
            branch.children.insert(cp, Box::new(Node::Slice(half)));
        }
        self.exit = Box::new(Node::Map(branch));
        self.end = offset;
    }

    fn put(
        mut self,
        strs: &Compressor,
        acc: &Accelerator,
        name: usize,
        id: usize,
        context: &PinIn,
    ) -> Node {
        let length = self.end - self.start;
        let m = acc.common_in(self.start, name, length, strs);
        if m >= length {
            // New string shares at least the full slice prefix.
            let exit = *self.exit;
            self.exit = Box::new(exit.put(strs, acc, name + length, id, context));
        } else {
            // Diverges within the slice: split here, then insert both branches.
            self.cut(strs, self.start + m);
            let exit = *self.exit;
            self.exit = Box::new(exit.put(strs, acc, name + m, id, context));
        }
        if self.start == self.end {
            // Slice collapsed to nothing; return the exit node directly.
            *self.exit
        } else {
            Node::Slice(self)
        }
    }
}

// ── NAcc ──────────────────────────────────────────────────────────────────────

/// Accelerated NMap: maintains a phoneme → child-code-points index so that
/// lookup only tests children whose first phoneme could match the current query
/// position, rather than testing every child.
struct NAcc {
    base: NMap,
    /// Maps `phoneme_id` → list of child code-points whose pinyins start with
    /// that phoneme.  Rebuilt whenever the PinIn config changes.
    index: HashMap<usize, Vec<u32>>,
}

impl NAcc {
    fn from_map(map: NMap, context: &PinIn) -> Self {
        let mut nacc = NAcc { base: map, index: HashMap::new() };
        nacc.rebuild_index(context);
        nacc
    }

    /// Rebuild the phoneme index from scratch (called on config change).
    fn rebuild_index(&mut self, context: &PinIn) {
        self.index.clear();
        let cps: Vec<u32> = self.base.children.keys().copied().collect();
        for cp in cps {
            self.add_to_index(context, cp);
        }
    }

    /// Add one code-point to the phoneme index.
    fn add_to_index(&mut self, context: &PinIn, cp: u32) {
        let char_data: CharData = context.get_char(cp);
        let inner = context.0.borrow();
        for &py_id in char_data.pinyins() {
            let py = inner.pinyin(py_id);
            if py.phonemes.is_empty() {
                continue;
            }
            let first_ph = py.phonemes[0];
            let entry = self.index.entry(first_ph).or_default();
            if !entry.contains(&cp) {
                entry.push(cp);
            }
        }
    }

    fn get_offset(
        &self,
        strs: &Compressor,
        acc: &mut Accelerator,
        search_str: &str,
        context: &PinIn,
        logic: Logic,
        ret: &mut BTreeSet<usize>,
        offset: usize,
    ) {
        if offset == search_str.len() {
            if logic == Logic::Equal {
                ret.extend(self.base.leaves.iter().copied());
            } else {
                self.base.get_all(ret);
            }
            return;
        }

        // ── Path 1: direct character match ───────────────────────────────────
        // If the character at the current query position is itself a trie child,
        // advance by exactly that character's UTF-8 length.
        let search_cp = acc.search_code_point(offset);
        let search_consumed = search_str[offset..].chars().next().unwrap().len_utf8();
        if let Some(child) = self.base.children.get(&search_cp) {
            child.get_offset(
                strs, acc, search_str, context, logic, ret,
                offset + search_consumed,
            );
        }

        // ── Path 2: phoneme-index look-up ────────────────────────────────────
        // For each pre-indexed first-phoneme, check whether it matches the
        // current query position (partial match suffices here).  If it does,
        // use the full acc.get() for each code-point in that phoneme's bucket.
        for (&ph_id, cps) in &self.index {
            let ph_matches = {
                let inner = context.0.borrow();
                let ph = inner.phoneme(ph_id);
                !ph.match_str(search_str, offset, true).is_empty()
            };
            if !ph_matches {
                continue;
            }
            for &cp in cps {
                if let Some(child) = self.base.children.get(&cp) {
                    let matched = acc.get(cp, offset);
                    let indices = collect_indices(matched);
                    for i in indices {
                        child.get_offset(
                            strs, acc, search_str, context, logic, ret,
                            offset + i as usize,
                        );
                    }
                }
            }
        }
    }

    fn get_all(&self, ret: &mut BTreeSet<usize>) {
        self.base.get_all(ret);
    }

    fn put(
        mut self,
        strs: &Compressor,
        acc: &Accelerator,
        name: usize,
        id: usize,
        context: &PinIn,
    ) -> Node {
        // Insert into the base NMap (without triggering another NAcc conversion).
        self.base.put_inner(strs, acc, name, id, context);
        // Mirror Java's NAcc.put: always re-index the code-point at `name`.
        if !strs.end(name) {
            let cp = strs.get(name);
            self.add_to_index(context, cp);
        }
        Node::Acc(self)
    }
}

// ── Node ──────────────────────────────────────────────────────────────────────

enum Node {
    Dense(NDense),
    Map(NMap),
    Slice(NSlice),
    Acc(NAcc),
}

impl Node {
    fn get_offset(
        &self,
        strs: &Compressor,
        acc: &mut Accelerator,
        search_str: &str,
        context: &PinIn,
        logic: Logic,
        ret: &mut BTreeSet<usize>,
        offset: usize,
    ) {
        match self {
            Node::Dense(n) => n.get_offset(strs, acc, search_str, logic, ret, offset),
            Node::Map(n) => n.get_offset(strs, acc, search_str, context, logic, ret, offset),
            Node::Slice(n) => n.get_offset(strs, acc, search_str, context, logic, ret, offset),
            Node::Acc(n) => n.get_offset(strs, acc, search_str, context, logic, ret, offset),
        }
    }

    fn get_all(&self, ret: &mut BTreeSet<usize>) {
        match self {
            Node::Dense(n) => n.get_all(ret),
            Node::Map(n) => n.get_all(ret),
            Node::Slice(n) => n.get_all(ret),
            Node::Acc(n) => n.get_all(ret),
        }
    }

    fn put(
        self,
        strs: &Compressor,
        acc: &Accelerator,
        name: usize,
        id: usize,
        context: &PinIn,
    ) -> Node {
        match self {
            Node::Dense(n) => n.put(strs, acc, name, id, context),
            Node::Map(n) => n.put(strs, acc, name, id, context),
            Node::Slice(n) => n.put(strs, acc, name, id, context),
            Node::Acc(n) => n.put(strs, acc, name, id, context),
        }
    }

    /// Recursively reload all NAcc phoneme indexes after a config change.
    fn reload(&mut self, context: &PinIn) {
        match self {
            Node::Acc(n) => {
                n.rebuild_index(context);
                for child in n.base.children.values_mut() {
                    child.reload(context);
                }
            }
            Node::Map(n) => {
                for child in n.children.values_mut() {
                    child.reload(context);
                }
            }
            Node::Slice(n) => n.exit.reload(context),
            Node::Dense(_) => {}
        }
    }
}

// ── TreeSearcher ──────────────────────────────────────────────────────────────

pub struct TreeSearcher<T> {
    root: Node,
    objects: Vec<T>,
    strs: Compressor,
    acc: Accelerator,
    context: PinIn,
    logic: Logic,
    last_modification: u64,
}

impl<T> TreeSearcher<T> {
    pub fn new(logic: Logic, context: PinIn) -> Self {
        Self {
            root: Node::Dense(NDense::new()),
            objects: Vec::new(),
            strs: Compressor::default(),
            acc: Accelerator::new(context.clone()),
            context,
            logic,
            last_modification: 0,
        }
    }

    /// Sync to the current config version; reload NAcc indexes and flush caches
    /// when the PinIn context has been modified since the last call.
    fn renew(&mut self) {
        let cur = self.context.modification();
        if self.last_modification != cur {
            self.last_modification = cur;
            self.root.reload(&self.context);
            self.acc.reset();
        }
    }

    pub fn refresh(&mut self) {
        self.renew();
    }
}

impl<T> Searcher<T> for TreeSearcher<T> {
    fn put(&mut self, name: &str, identifier: T) {
        self.renew();
        let pos = self.strs.put(name);
        let id = self.objects.len();

        if self.logic == Logic::Contain {
            // For CONTAIN, insert an entry for every starting position in name
            // so that any substring of name can be found from the root.
            let mut i = 0;
            while i < name.len() {
                let root =
                    std::mem::replace(&mut self.root, Node::Dense(NDense::new()));
                self.root = root.put(&self.strs, &self.acc, pos + i, id, &self.context);
                i += name[i..].chars().next().unwrap().len_utf8();
            }
        } else {
            let root = std::mem::replace(&mut self.root, Node::Dense(NDense::new()));
            self.root = root.put(&self.strs, &self.acc, pos, id, &self.context);
        }

        self.objects.push(identifier);
    }

    fn search<'a>(&'a mut self, name: &str) -> Vec<&'a T> {
        self.renew();
        self.acc.search(name);

        // Ensure the partial flag matches the current logic and flush the cache
        // if it needs to change (same semantics as Java's begins/matches).
        let is_partial = self.logic != Logic::Equal;
        if self.acc.is_partial() != is_partial {
            self.acc.set_partial(is_partial);
            self.acc.reset();
        }

        let mut ret = BTreeSet::new();
        // Borrow individual fields so the borrow-checker can track them
        // independently from `self.root`.
        let strs = &self.strs;
        let context = &self.context;
        let logic = self.logic;
        let acc = &mut self.acc;
        self.root.get_offset(strs, acc, name, context, logic, &mut ret, 0);

        // Results are collected as sorted object indices; map to references.
        ret.into_iter().map(|i| &self.objects[i]).collect()
    }

    fn context(&self) -> PinIn {
        self.context.clone()
    }
}
