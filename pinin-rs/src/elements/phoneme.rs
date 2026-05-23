use compact_str::CompactString;
use smallvec::SmallVec;
use std::collections::HashSet;

use crate::pinin::PinInInner;
use crate::utils::IndexSet;

#[derive(Clone, Debug, Default)]
pub struct Phoneme {
    pub strs: SmallVec<[CompactString; 4]>,
}

impl Phoneme {
    pub fn reload(&mut self, key: &str, p: &PinInInner) {
        let mut set: HashSet<String> = HashSet::new();
        set.insert(key.to_string());
        if p.config.f_ch2c && key.starts_with('c') {
            set.insert("c".into());
            set.insert("ch".into());
        }
        if p.config.f_sh2s && key.starts_with('s') {
            set.insert("s".into());
            set.insert("sh".into());
        }
        if p.config.f_zh2z && key.starts_with('z') {
            set.insert("z".into());
            set.insert("zh".into());
        }
        if p.config.f_u2v && key.starts_with('v') {
            set.insert(format!("u{}", &key[1..]));
        }
        let ang = p.config.f_ang2an && key.ends_with("ang");
        let eng = p.config.f_eng2en && key.ends_with("eng");
        let ing = p.config.f_ing2in && key.ends_with("ing");
        if ang || eng || ing {
            set.insert(key[..key.len() - 1].to_string());
        }
        let an = p.config.f_ang2an && key.ends_with("an");
        let en = p.config.f_eng2en && key.ends_with("en");
        let in_ = p.config.f_ing2in && key.ends_with("in");
        if an || en || in_ {
            set.insert(format!("{key}g"));
        }
        let mut strs = SmallVec::new();
        for item in set {
            strs.push(CompactString::from(p.config.keyboard.keys(&item)));
        }
        self.strs = strs;
    }

    pub fn is_empty(&self) -> bool {
        self.strs.len() == 1 && self.strs[0].is_empty()
    }

    pub fn match_sequence(&self, c: char) -> bool {
        self.strs.iter().any(|s| s.chars().next() == Some(c))
    }

    fn str_cmp(a: &str, b: &str, a_start: usize) -> usize {
        let aa = &a.as_bytes()[a_start..];
        let bb = b.as_bytes();
        let len = aa.len().min(bb.len());
        for i in 0..len {
            if aa[i] != bb[i] {
                return i;
            }
        }
        len
    }

    pub fn match_str(&self, source: &str, start: usize, partial: bool) -> IndexSet {
        let mut ret = IndexSet::NONE;
        if self.is_empty() {
            return ret;
        }
        for s in &self.strs {
            let size = Self::str_cmp(source, s.as_str(), start);
            if partial && start + size == source.len() {
                ret.set(size as u32);
            } else if size == s.len() {
                ret.set(size as u32);
            }
        }
        ret
    }

    pub fn match_idx(&self, source: &str, idx: IndexSet, start: usize, partial: bool) -> IndexSet {
        if self.is_empty() {
            return idx;
        }
        let mut ret = IndexSet::NONE;
        idx.foreach(|i| {
            let mut is = self.match_str(source, start + i as usize, partial);
            is.offset(i);
            ret.merge(is);
        });
        ret
    }
}
