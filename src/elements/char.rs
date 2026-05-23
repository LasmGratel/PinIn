use smallvec::SmallVec;

use crate::pinin::PinInInner;
use crate::utils::IndexSet;

#[derive(Clone, Debug, Default)]
pub struct CharData {
    pub code_point: u32,
    pub pinyins: SmallVec<[usize; 4]>,
}

impl CharData {
    pub fn new(code_point: u32, pinyins: SmallVec<[usize; 4]>) -> Self {
        Self { code_point, pinyins }
    }

    pub fn match_str(&self, s: &str, start: usize, partial: bool, inner: &PinInInner) -> IndexSet {
        let mut ret = IndexSet::NONE;
        if let Some(ch) = s[start..].chars().next() {
            if ch as u32 == self.code_point {
                ret.set(ch.len_utf8() as u32);
            }
        }
        for py_id in &self.pinyins {
            ret.merge(inner.pinyin(*py_id).match_str(s, start, partial, inner));
        }
        ret
    }

    pub fn pinyins(&self) -> &[usize] {
        &self.pinyins
    }
}
