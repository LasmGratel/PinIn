use std::collections::HashMap;

use crate::pinin::PinIn;
use crate::utils::{Compressor, IndexSet};

#[derive(Clone)]
enum ProviderKind {
    None,
    Str(String),
    Compressor(Compressor),
}

impl Default for ProviderKind {
    fn default() -> Self {
        Self::None
    }
}

impl ProviderKind {
    fn end(&self, i: usize) -> bool {
        match self {
            ProviderKind::None => true,
            ProviderKind::Str(s) => i >= s.len(),
            ProviderKind::Compressor(c) => c.end(i),
        }
    }

    fn get(&self, i: usize) -> u32 {
        match self {
            ProviderKind::None => 0,
            ProviderKind::Str(s) => s[i..].chars().next().unwrap() as u32,
            ProviderKind::Compressor(c) => c.get(i),
        }
    }

    fn char_len(&self, i: usize) -> usize {
        match self {
            ProviderKind::None => 0,
            ProviderKind::Str(s) => s[i..].chars().next().unwrap().len_utf8(),
            ProviderKind::Compressor(c) => c.char_len(i),
        }
    }
}

#[derive(Clone)]
pub struct Accelerator {
    context: PinIn,
    cache: Vec<HashMap<usize, IndexSet>>,
    search_str: String,
    provider: ProviderKind,
    partial: bool,
}

impl Accelerator {
    pub fn new(context: PinIn) -> Self {
        Self { context, cache: Vec::new(), search_str: String::new(), provider: ProviderKind::None, partial: false }
    }

    pub fn search(&mut self, s: &str) {
        if self.search_str != s {
            self.search_str.clear();
            self.search_str.push_str(s);
            self.reset();
        }
    }

    pub fn set_provider_str(&mut self, s: &str) {
        self.provider = ProviderKind::Str(s.to_string());
    }

    pub fn set_provider_compressor(&mut self, c: Compressor) {
        self.provider = ProviderKind::Compressor(c);
    }

    pub fn reset(&mut self) {
        self.cache.clear();
    }

    pub fn search_str(&self) -> &str {
        &self.search_str
    }

    pub fn search_code_point(&self, offset: usize) -> u32 {
        self.search_str[offset..].chars().next().unwrap() as u32
    }

    fn search_char_len(&self, offset: usize) -> usize {
        self.search_str[offset..].chars().next().unwrap().len_utf8()
    }

    pub fn get(&mut self, code_point: u32, offset: usize) -> IndexSet {
        let inner = self.context.0.borrow();
        let ch = inner.get_char_data(code_point);
        drop(inner);
        let mut ret = IndexSet::NONE;
        if offset < self.search_str.len() {
            let search_code = self.search_code_point(offset);
            if search_code == ch.code_point {
                ret.set(self.search_char_len(offset) as u32);
            }
        }
        for py in ch.pinyins() {
            ret.merge(self.get_pinyin(*py, offset));
        }
        ret
    }

    pub fn get_pinyin(&mut self, pinyin: usize, offset: usize) -> IndexSet {
        while self.cache.len() <= offset {
            self.cache.push(HashMap::new());
        }
        if let Some(ret) = self.cache[offset].get(&pinyin).copied() {
            return ret;
        }
        let inner = self.context.0.borrow();
        let ret = inner.pinyin(pinyin).match_str(&self.search_str, offset, self.partial, &inner);
        drop(inner);
        self.cache[offset].insert(pinyin, ret);
        ret
    }

    pub fn common(&self, s1: usize, s2: usize, max: usize) -> usize {
        let mut o1 = s1;
        let mut o2 = s2;
        let mut matched = 0;
        while matched < max {
            if self.provider.end(o1) || self.provider.end(o2) {
                return matched;
            }
            let a = self.provider.get(o1);
            let b = self.provider.get(o2);
            if a != b {
                return matched;
            }
            let consumed = self.provider.char_len(o1);
            o1 += consumed;
            o2 += consumed;
            matched += consumed;
        }
        max
    }

    pub fn check(&mut self, offset: usize, start: usize) -> bool {
        if offset == self.search_str.len() {
            return self.partial || self.provider.end(start);
        }
        if self.provider.end(start) {
            return false;
        }
        let code_point = self.provider.get(start);
        let matched = self.get(code_point, offset);
        let consumed = self.provider.char_len(start);
        if self.provider.end(start + consumed) {
            matched.get((self.search_str.len() - offset) as u32)
        } else {
            matched.traverse(|i| self.check(offset + i as usize, start + consumed))
        }
    }

    pub fn matches(&mut self, offset: usize, start: usize) -> bool {
        if self.partial {
            self.partial = false;
            self.reset();
        }
        self.check(offset, start)
    }

    pub fn begins(&mut self, offset: usize, start: usize) -> bool {
        if !self.partial {
            self.partial = true;
            self.reset();
        }
        self.check(offset, start)
    }

    pub fn contains(&mut self, offset: usize, start: usize) -> bool {
        if !self.partial {
            self.partial = true;
            self.reset();
        }
        let mut i = start;
        while !self.provider.end(i) {
            if self.check(offset, i) {
                return true;
            }
            i += self.provider.char_len(i);
        }
        false
    }
}
