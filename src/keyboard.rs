#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Keyboard {
    Quanpin,
    Daqian,
    Xiaohe,
    Ziranma,
    Sougou,
    Guobiao,
    Microsoft,
    Pinyinpp,
    Ziguang,
}

pub const QUANPIN: Keyboard = Keyboard::Quanpin;
pub const DAQIAN: Keyboard = Keyboard::Daqian;
pub const XIAOHE: Keyboard = Keyboard::Xiaohe;
pub const ZIRANMA: Keyboard = Keyboard::Ziranma;
pub const SOUGOU: Keyboard = Keyboard::Sougou;
pub const GUOBIAO: Keyboard = Keyboard::Guobiao;
pub const MICROSOFT: Keyboard = Keyboard::Microsoft;
pub const PINYINPP: Keyboard = Keyboard::Pinyinpp;
pub const ZIGUANG: Keyboard = Keyboard::Ziguang;

const PHONETIC_LOCAL: &[(&str, &str)] = &[
    ("yi", "i"), ("you", "iu"), ("yin", "in"), ("ye", "ie"), ("ying", "ing"),
    ("wu", "u"), ("wen", "un"), ("yu", "v"), ("yue", "ve"), ("yuan", "van"),
    ("yun", "vn"), ("ju", "jv"), ("jue", "jve"), ("juan", "jvan"), ("jun", "jvn"),
    ("qu", "qv"), ("que", "qve"), ("quan", "qvan"), ("qun", "qvn"), ("xu", "xv"),
    ("xue", "xve"), ("xuan", "xvan"), ("xun", "xvn"), ("shi", "sh"), ("si", "s"),
    ("chi", "ch"), ("ci", "c"), ("zhi", "zh"), ("zi", "z"), ("ri", "r"),
];

const DAQIAN_KEYS: &[(&str, &str)] = &[
    ("", ""), ("0", ""), ("1", " "), ("2", "6"), ("3", "3"), ("4", "4"),
    ("a", "8"), ("ai", "9"), ("an", "0"), ("ang", ";"), ("ao", "l"), ("b", "1"),
    ("c", "h"), ("ch", "t"), ("d", "2"), ("e", "k"), ("ei", "o"), ("en", "p"),
    ("eng", "/"), ("er", "-"), ("f", "z"), ("g", "e"), ("h", "c"), ("i", "u"),
    ("ia", "u8"), ("ian", "u0"), ("iang", "u;"), ("iao", "ul"), ("ie", "u,"),
    ("in", "up"), ("ing", "u/"), ("iong", "m/"), ("iu", "u."), ("j", "r"),
    ("k", "d"), ("l", "x"), ("m", "a"), ("n", "s"), ("o", "i"), ("ong", "j/"),
    ("ou", "."), ("p", "q"), ("q", "f"), ("r", "b"), ("s", "n"), ("sh", "g"),
    ("t", "w"), ("u", "j"), ("ua", "j8"), ("uai", "j9"), ("uan", "j0"),
    ("uang", "j;"), ("uen", "mp"), ("ueng", "j/"), ("ui", "jo"), ("un", "jp"),
    ("uo", "ji"), ("v", "m"), ("van", "m0"), ("vang", "m;"), ("ve", "m,"),
    ("vn", "mp"), ("w", "j"), ("x", "v"), ("y", "u"), ("z", "y"), ("zh", "5"),
];

const XIAOHE_KEYS: &[(&str, &str)] = &[
    ("ai", "d"), ("an", "j"), ("ang", "h"), ("ao", "c"), ("ch", "i"),
    ("ei", "w"), ("en", "f"), ("eng", "g"), ("ia", "x"), ("ian", "m"),
    ("iang", "l"), ("iao", "n"), ("ie", "p"), ("in", "b"), ("ing", "k"),
    ("iong", "s"), ("iu", "q"), ("ong", "s"), ("ou", "z"), ("sh", "u"),
    ("ua", "x"), ("uai", "k"), ("uan", "r"), ("uang", "l"), ("ui", "v"),
    ("un", "y"), ("uo", "o"), ("ve", "t"), ("ue", "t"), ("vn", "y"), ("zh", "v"),
];

const ZIRANMA_KEYS: &[(&str, &str)] = &[
    ("ai", "l"), ("an", "j"), ("ang", "h"), ("ao", "k"), ("ch", "i"),
    ("ei", "z"), ("en", "f"), ("eng", "g"), ("ia", "w"), ("ian", "m"),
    ("iang", "d"), ("iao", "c"), ("ie", "x"), ("in", "n"), ("ing", "y"),
    ("iong", "s"), ("iu", "q"), ("ong", "s"), ("ou", "b"), ("sh", "u"),
    ("ua", "w"), ("uai", "y"), ("uan", "r"), ("uang", "d"), ("ui", "v"),
    ("un", "p"), ("uo", "o"), ("ve", "t"), ("ue", "t"), ("vn", "p"), ("zh", "v"),
];

const SOUGOU_KEYS: &[(&str, &str)] = &[
    ("ai", "l"), ("an", "j"), ("ang", "h"), ("ao", "k"), ("ch", "i"),
    ("ei", "z"), ("en", "f"), ("eng", "g"), ("ia", "w"), ("ian", "m"),
    ("iang", "d"), ("iao", "c"), ("ie", "x"), ("in", "n"), ("ing", ";"),
    ("iong", "s"), ("iu", "q"), ("ong", "s"), ("ou", "b"), ("sh", "u"),
    ("ua", "w"), ("uai", "y"), ("uan", "r"), ("uang", "d"), ("ui", "v"),
    ("un", "p"), ("uo", "o"), ("ve", "t"), ("ue", "t"), ("v", "y"), ("zh", "v"),
];

const GUOBIAO_KEYS: &[(&str, &str)] = &[
    ("ai", "k"), ("an", "f"), ("ang", "g"), ("ao", "c"), ("ch", "i"),
    ("ei", "b"), ("en", "r"), ("eng", "h"), ("er", "l"), ("ia", "q"),
    ("ian", "d"), ("iang", "n"), ("iao", "m"), ("ie", "t"), ("in", "l"),
    ("ing", "j"), ("iong", "s"), ("iu", "y"), ("ong", "s"), ("ou", "p"),
    ("sh", "u"), ("ua", "q"), ("uai", "y"), ("uan", "w"), ("uang", "n"),
    ("ui", "v"), ("un", "z"), ("uo", "o"), ("van", "w"), ("ve", "x"), ("vn", "z"),
    ("zh", "v"),
];

const MICROSOFT_KEYS: &[(&str, &str)] = &[
    ("ai", "l"), ("an", "j"), ("ang", "h"), ("ao", "k"), ("ch", "i"),
    ("ei", "z"), ("en", "f"), ("eng", "g"), ("er", "r"), ("ia", "w"),
    ("ian", "m"), ("iang", "d"), ("iao", "c"), ("ie", "x"), ("in", "n"),
    ("ing", ";"), ("iong", "s"), ("iu", "q"), ("ong", "s"), ("ou", "b"),
    ("sh", "u"), ("ua", "w"), ("uai", "y"), ("uan", "r"), ("uang", "d"),
    ("ui", "v"), ("un", "p"), ("uo", "o"), ("ve", "v"), ("ue", "t"), ("v", "y"),
    ("zh", "v"),
];

const PINYINPP_KEYS: &[(&str, &str)] = &[
    ("ai", "s"), ("an", "f"), ("ang", "g"), ("ao", "d"), ("ch", "u"),
    ("ei", "w"), ("en", "r"), ("eng", "t"), ("er", "q"), ("ia", "b"),
    ("ian", "j"), ("iang", "h"), ("iao", "k"), ("ie", "m"), ("in", "l"),
    ("ing", "q"), ("iong", "y"), ("iu", "n"), ("ong", "y"), ("ou", "p"),
    ("ua", "b"), ("uai", "x"), ("uan", "c"), ("uang", "h"), ("ue", "x"),
    ("ui", "v"), ("un", "z"), ("uo", "o"), ("sh", "i"), ("zh", "v"),
];

const ZIGUANG_KEYS: &[(&str, &str)] = &[
    ("ai", "p"), ("an", "r"), ("ang", "s"), ("ao", "q"), ("ch", "a"),
    ("ei", "k"), ("en", "w"), ("eng", "t"), ("er", "j"), ("ia", "x"),
    ("ian", "f"), ("iang", "g"), ("iao", "b"), ("ie", "d"), ("in", "y"),
    ("ing", ";"), ("iong", "h"), ("iu", "j"), ("ong", "h"), ("ou", "z"),
    ("ua", "x"), ("uan", "l"), ("uai", "y"), ("uang", "g"), ("ue", "n"),
    ("un", "m"), ("uo", "o"), ("ve", "n"), ("sh", "i"), ("zh", "u"),
];

fn lookup(map: &'static [(&'static str, &'static str)], s: &str) -> Option<&'static str> {
    map.iter().find_map(|(k, v)| (*k == s).then_some(*v))
}

fn has_initial(s: &str) -> bool {
    !matches!(s.as_bytes().first().copied(), Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'v'))
}

impl Keyboard {
    fn local_map(self) -> Option<&'static [(&'static str, &'static str)]> {
        match self {
            Keyboard::Daqian => Some(PHONETIC_LOCAL),
            _ => None,
        }
    }

    fn key_map(self) -> Option<&'static [(&'static str, &'static str)]> {
        match self {
            Keyboard::Quanpin => None,
            Keyboard::Daqian => Some(DAQIAN_KEYS),
            Keyboard::Xiaohe => Some(XIAOHE_KEYS),
            Keyboard::Ziranma => Some(ZIRANMA_KEYS),
            Keyboard::Sougou => Some(SOUGOU_KEYS),
            Keyboard::Guobiao => Some(GUOBIAO_KEYS),
            Keyboard::Microsoft => Some(MICROSOFT_KEYS),
            Keyboard::Pinyinpp => Some(PINYINPP_KEYS),
            Keyboard::Ziguang => Some(ZIGUANG_KEYS),
        }
    }

    pub fn duo(self) -> bool {
        !matches!(self, Keyboard::Quanpin | Keyboard::Daqian)
    }

    pub fn sequence(self) -> bool {
        matches!(self, Keyboard::Quanpin)
    }

    pub fn keys(self, s: &str) -> String {
        self.key_map().and_then(|m| lookup(m, s)).unwrap_or(s).to_string()
    }

    pub fn split(self, s: &str) -> Vec<String> {
        let mut normalized = s.to_string();
        if let Some(local) = self.local_map() {
            let tone = normalized.chars().last().unwrap();
            let cut = &normalized[..normalized.len() - tone.len_utf8()];
            if let Some(alt) = lookup(local, cut) {
                normalized = format!("{alt}{tone}");
            }
        }
        match self {
            Keyboard::Quanpin | Keyboard::Daqian => Self::standard(&normalized),
            _ => Self::zero(&normalized),
        }
    }

    pub fn standard(s: &str) -> Vec<String> {
        let chars: Vec<char> = s.chars().collect();
        let mut ret = Vec::new();
        let mut cursor = 0usize;
        if has_initial(s) {
            cursor = if chars.len() > 2 && chars[1] == 'h' { 2 } else { 1 };
            ret.push(chars[..cursor].iter().collect());
        }
        if chars.len() != cursor + 1 {
            ret.push(chars[cursor..chars.len() - 1].iter().collect());
        }
        ret.push(chars[chars.len() - 1..].iter().collect());
        ret
    }

    pub fn zero(s: &str) -> Vec<String> {
        let mut ss = Self::standard(s);
        if ss.len() == 2 {
            let finale = ss[0].clone();
            let mut chars = finale.chars();
            let first = chars.next().unwrap().to_string();
            let rest: String = chars.collect();
            ss[0] = first;
            ss.insert(1, rest);
        }
        ss
    }
}
