use compact_str::CompactString;
use smallvec::SmallVec;

use crate::pinin::PinInInner;
use crate::utils::IndexSet;

#[derive(Clone, Debug)]
pub struct Pinyin {
    pub id: usize,
    pub raw: CompactString,
    pub phonemes: SmallVec<[usize; 3]>,
    pub duo: bool,
    pub sequence: bool,
}

impl Pinyin {
    pub fn new(id: usize, raw: &str, inner: &mut PinInInner) -> Self {
        let mut py = Self {
            id,
            raw: CompactString::from(raw),
            phonemes: SmallVec::new(),
            duo: false,
            sequence: false,
        };
        py.reload(raw, inner);
        py
    }

    pub fn reload(&mut self, raw: &str, inner: &mut PinInInner) {
        let split = inner.config.keyboard.split(raw);
        self.phonemes.clear();
        for s in split {
            self.phonemes.push(inner.ensure_phoneme(&s));
        }
        self.duo = inner.config.keyboard.duo();
        self.sequence = inner.config.keyboard.sequence();
    }

    pub fn match_str(&self, s: &str, start: usize, partial: bool, inner: &PinInInner) -> IndexSet {
        let mut ret;
        if self.duo {
            ret = IndexSet::ZERO;
            ret = inner.phoneme(self.phonemes[0]).match_idx(s, ret, start, partial);
            ret = inner.phoneme(self.phonemes[1]).match_idx(s, ret, start, partial);
            let tone = inner.phoneme(self.phonemes[2]).match_idx(s, ret, start, partial);
            ret.merge(tone);
        } else {
            let mut active = IndexSet::ZERO;
            ret = IndexSet::NONE;
            for id in &self.phonemes {
                active = inner.phoneme(*id).match_idx(s, active, start, partial);
                if active.is_empty() {
                    break;
                }
                ret.merge(active);
            }
        }
        if self.sequence {
            if let Some(c) = s[start..].chars().next() {
                if inner.phoneme(self.phonemes[0]).match_sequence(c) {
                    ret.set(c.len_utf8() as u32);
                }
            }
        }
        ret
    }

    pub fn has_initial(s: &str) -> bool {
        !matches!(s.as_bytes().first().copied(), Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'v'))
    }
}
