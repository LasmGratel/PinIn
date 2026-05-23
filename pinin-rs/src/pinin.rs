use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use compact_str::CompactString;
use smallvec::SmallVec;

use crate::dict_loader::{DefaultLoader, DictLoader};
use crate::elements::{CharData, Phoneme, Pinyin};
use crate::keyboard::{Keyboard, QUANPIN};
use crate::utils::{Accelerator, PinyinFormat};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub keyboard: Keyboard,
    pub f_zh2z: bool,
    pub f_sh2s: bool,
    pub f_ch2c: bool,
    pub f_ang2an: bool,
    pub f_ing2in: bool,
    pub f_eng2en: bool,
    pub f_u2v: bool,
    pub accelerate: bool,
    pub format: PinyinFormat,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keyboard: QUANPIN,
            f_zh2z: false,
            f_sh2s: false,
            f_ch2c: false,
            f_ang2an: false,
            f_ing2in: false,
            f_eng2en: false,
            f_u2v: false,
            accelerate: false,
            format: PinyinFormat::NUMBER,
        }
    }
}

#[derive(Clone)]
pub struct PinIn(pub(crate) Rc<RefCell<PinInInner>>);

#[derive(Clone, Debug)]
pub(crate) struct PhonemeEntry {
    pub key: CompactString,
    pub data: Phoneme,
}

#[derive(Clone, Debug)]
pub(crate) struct PinyinEntry {
    pub key: CompactString,
    pub data: Pinyin,
}

pub struct PinInInner {
    pub config: Config,
    pub modification: u64,
    chars: Vec<Option<usize>>,
    extra_chars: HashMap<u32, usize>,
    char_data: Vec<CharData>,
    pub(crate) phonemes: HashMap<String, usize>,
    pub(crate) phoneme_data: Vec<PhonemeEntry>,
    pub(crate) pinyins: HashMap<String, usize>,
    pub(crate) pinyin_data: Vec<PinyinEntry>,
}

impl PinInInner {
    fn new() -> Self {
        Self {
            config: Config::default(),
            modification: 0,
            chars: vec![None; 65536],
            extra_chars: HashMap::new(),
            char_data: Vec::new(),
            phonemes: HashMap::new(),
            phoneme_data: Vec::new(),
            pinyins: HashMap::new(),
            pinyin_data: Vec::new(),
        }
    }

    pub(crate) fn phoneme(&self, id: usize) -> &Phoneme {
        &self.phoneme_data[id].data
    }

    pub(crate) fn pinyin(&self, id: usize) -> &Pinyin {
        &self.pinyin_data[id].data
    }

    pub(crate) fn ensure_phoneme(&mut self, s: &str) -> usize {
        if let Some(&id) = self.phonemes.get(s) {
            return id;
        }
        let mut data = Phoneme::default();
        data.reload(s, self);
        let id = self.phoneme_data.len();
        self.phoneme_data.push(PhonemeEntry { key: CompactString::from(s), data });
        self.phonemes.insert(s.to_string(), id);
        id
    }

    pub(crate) fn ensure_pinyin(&mut self, s: &str) -> usize {
        if let Some(&id) = self.pinyins.get(s) {
            return id;
        }
        let id = self.pinyin_data.len();
        let data = Pinyin::new(id, s, self);
        self.pinyin_data.push(PinyinEntry { key: CompactString::from(s), data });
        self.pinyins.insert(s.to_string(), id);
        id
    }

    fn set_char(&mut self, cp: u32, ss: &[&str]) {
        let mut pinyins = SmallVec::new();
        for &record in ss {
            pinyins.push(self.ensure_pinyin(record));
        }
        let idx = self.char_data.len();
        self.char_data.push(CharData::new(cp, pinyins));
        if cp <= 0xffff {
            self.chars[cp as usize] = Some(idx);
            self.extra_chars.remove(&cp);
        } else {
            self.extra_chars.insert(cp, idx);
        }
    }

    pub(crate) fn get_char_data(&self, cp: u32) -> CharData {
        let idx = if cp <= 0xffff {
            self.chars[cp as usize]
        } else {
            self.extra_chars.get(&cp).copied()
        };
        idx.map(|i| self.char_data[i].clone()).unwrap_or_else(|| CharData::new(cp, SmallVec::new()))
    }

    fn apply_config(&mut self, config: Config) {
        if self.config == config {
            return;
        }
        self.config = config;
        let phoneme_keys: Vec<String> = self.phoneme_data.iter().map(|e| e.key.to_string()).collect();
        for (i, key) in phoneme_keys.iter().enumerate() {
            let mut data = Phoneme::default();
            data.reload(key, self);
            self.phoneme_data[i].data = data;
        }
        let pinyin_keys: Vec<String> = self.pinyin_data.iter().map(|e| e.key.to_string()).collect();
        for (i, key) in pinyin_keys.iter().enumerate() {
            self.pinyin_data[i].data = Pinyin::new(i, key, self);
        }
        self.modification += 1;
    }
}

impl PinIn {
    pub fn new() -> Self {
        Self::with_loader(&DefaultLoader)
    }

    pub fn with_loader(loader: &dyn DictLoader) -> Self {
        let mut inner = PinInInner::new();
        loader.load(&mut |c, ss| inner.set_char(c as u32, ss));
        loader.load_code_points(&mut |cp, ss| inner.set_char(cp, ss));
        Self(Rc::new(RefCell::new(inner)))
    }

    pub fn config(&self) -> ConfigBuilder {
        ConfigBuilder { pinin: self.clone(), config: self.0.borrow().config }
    }

    pub fn modification(&self) -> u64 {
        self.0.borrow().modification
    }

    pub fn get_char<C: Into<u32>>(&self, cp: C) -> CharData {
        self.0.borrow().get_char_data(cp.into())
    }

    pub fn get_phoneme(&self, s: &str) -> usize {
        self.0.borrow_mut().ensure_phoneme(s)
    }

    pub fn get_pinyin(&self, s: &str) -> usize {
        self.0.borrow_mut().ensure_pinyin(s)
    }

    pub fn format(&self, pinyin_id: usize) -> String {
        let inner = self.0.borrow();
        inner.config.format.format_raw(inner.pinyin(pinyin_id).raw.as_str())
    }

    pub fn contains(&self, s1: &str, s2: &str) -> bool {
        if self.0.borrow().config.accelerate {
            let mut acc = Accelerator::new(self.clone());
            acc.set_provider_str(s1);
            acc.search(s2);
            acc.contains(0, 0)
        } else {
            let inner = self.0.borrow();
            Matcher::contains(s1, s2, &inner)
        }
    }

    pub fn begins(&self, s1: &str, s2: &str) -> bool {
        if self.0.borrow().config.accelerate {
            let mut acc = Accelerator::new(self.clone());
            acc.set_provider_str(s1);
            acc.search(s2);
            acc.begins(0, 0)
        } else {
            let inner = self.0.borrow();
            Matcher::begins(s1, s2, &inner)
        }
    }

    pub fn matches(&self, s1: &str, s2: &str) -> bool {
        if self.0.borrow().config.accelerate {
            let mut acc = Accelerator::new(self.clone());
            acc.set_provider_str(s1);
            acc.search(s2);
            acc.matches(0, 0)
        } else {
            let inner = self.0.borrow();
            Matcher::matches(s1, s2, &inner)
        }
    }
}

impl Default for PinIn {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConfigBuilder {
    pinin: PinIn,
    config: Config,
}

impl ConfigBuilder {
    pub fn keyboard(mut self, keyboard: Keyboard) -> Self { self.config.keyboard = keyboard; self }
    pub fn f_zh2z(mut self, v: bool) -> Self { self.config.f_zh2z = v; self }
    pub fn f_sh2s(mut self, v: bool) -> Self { self.config.f_sh2s = v; self }
    pub fn f_ch2c(mut self, v: bool) -> Self { self.config.f_ch2c = v; self }
    pub fn f_ang2an(mut self, v: bool) -> Self { self.config.f_ang2an = v; self }
    pub fn f_ing2in(mut self, v: bool) -> Self { self.config.f_ing2in = v; self }
    pub fn f_eng2en(mut self, v: bool) -> Self { self.config.f_eng2en = v; self }
    pub fn f_u2v(mut self, v: bool) -> Self { self.config.f_u2v = v; self }
    pub fn accelerate(mut self, v: bool) -> Self { self.config.accelerate = v; self }
    pub fn format(mut self, v: PinyinFormat) -> Self { self.config.format = v; self }
    pub fn commit(self) -> PinIn {
        self.pinin.0.borrow_mut().apply_config(self.config);
        self.pinin
    }
}

pub struct Matcher;

impl Matcher {
    pub fn begins(s1: &str, s2: &str, p: &PinInInner) -> bool {
        if s1.is_empty() { s1.starts_with(s2) } else { Self::check(s1, 0, s2, 0, p, true) }
    }

    pub fn contains(s1: &str, s2: &str, p: &PinInInner) -> bool {
        if s1.is_empty() {
            return s1.contains(s2);
        }
        for (i, _) in s1.char_indices() {
            if Self::check(s1, i, s2, 0, p, true) {
                return true;
            }
        }
        false
    }

    pub fn matches(s1: &str, s2: &str, p: &PinInInner) -> bool {
        if s1.is_empty() { s1 == s2 } else { Self::check(s1, 0, s2, 0, p, false) }
    }

    fn check(s1: &str, start1: usize, s2: &str, start2: usize, p: &PinInInner, partial: bool) -> bool {
        if start2 == s2.len() {
            return partial || start1 == s1.len();
        }
        if start1 >= s1.len() {
            return false;
        }
        let ch = s1[start1..].chars().next().unwrap();
        let consumed = ch.len_utf8();
        let matched = p.get_char_data(ch as u32).match_str(s2, start2, partial, p);
        let next = start1 + consumed;
        if next >= s1.len() {
            matched.get((s2.len() - start2) as u32)
        } else {
            matched.traverse(|i| Self::check(s1, next, s2, start2 + i as usize, p, partial))
        }
    }
}
