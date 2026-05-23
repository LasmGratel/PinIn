#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatKind {
    Raw,
    Number,
    Phonetic,
    Unicode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinyinFormat(pub FormatKind);

impl PinyinFormat {
    pub const RAW: Self = Self(FormatKind::Raw);
    pub const NUMBER: Self = Self(FormatKind::Number);
    pub const PHONETIC: Self = Self(FormatKind::Phonetic);
    pub const UNICODE: Self = Self(FormatKind::Unicode);

    pub fn format_raw(self, raw: &str) -> String {
        match self.0 {
            FormatKind::Raw => raw[..raw.len() - 1].to_string(),
            FormatKind::Number => raw.to_string(),
            FormatKind::Phonetic => phonetic(raw),
            FormatKind::Unicode => unicode(raw),
        }
    }
}

fn local(s: &str) -> Option<&'static str> {
    match s {
        "yi" => Some("i"), "you" => Some("iu"), "yin" => Some("in"), "ye" => Some("ie"), "ying" => Some("ing"),
        "wu" => Some("u"), "wen" => Some("un"), "yu" => Some("v"), "yue" => Some("ve"), "yuan" => Some("van"),
        "yun" => Some("vn"), "ju" => Some("jv"), "jue" => Some("jve"), "juan" => Some("jvan"), "jun" => Some("jvn"),
        "qu" => Some("qv"), "que" => Some("qve"), "quan" => Some("qvan"), "qun" => Some("qvn"), "xu" => Some("xv"),
        "xue" => Some("xve"), "xuan" => Some("xvan"), "xun" => Some("xvn"), "shi" => Some("sh"), "si" => Some("s"),
        "chi" => Some("ch"), "ci" => Some("c"), "zhi" => Some("zh"), "zi" => Some("z"), "ri" => Some("r"),
        _ => None,
    }
}

fn symbol(s: &str) -> &'static str {
    match s {
        "a" => "ㄚ", "o" => "ㄛ", "e" => "ㄜ", "er" => "ㄦ", "ai" => "ㄞ", "ei" => "ㄟ", "ao" => "ㄠ",
        "ou" => "ㄡ", "an" => "ㄢ", "en" => "ㄣ", "ang" => "ㄤ", "eng" => "ㄥ", "ong" => "ㄨㄥ",
        "i" => "ㄧ", "ia" => "ㄧㄚ", "iao" => "ㄧㄠ", "ie" => "ㄧㄝ", "iu" => "ㄧㄡ", "ian" => "ㄧㄢ",
        "in" => "ㄧㄣ", "iang" => "ㄧㄤ", "ing" => "ㄧㄥ", "iong" => "ㄩㄥ", "u" => "ㄨ", "ua" => "ㄨㄚ",
        "uo" => "ㄨㄛ", "uai" => "ㄨㄞ", "ui" => "ㄨㄟ", "uan" => "ㄨㄢ", "un" => "ㄨㄣ", "uang" => "ㄨㄤ",
        "ueng" => "ㄨㄥ", "uen" => "ㄩㄣ", "v" => "ㄩ", "ve" => "ㄩㄝ", "van" => "ㄩㄢ", "vang" => "ㄩㄤ",
        "vn" => "ㄩㄣ", "b" => "ㄅ", "p" => "ㄆ", "m" => "ㄇ", "f" => "ㄈ", "d" => "ㄉ", "t" => "ㄊ",
        "n" => "ㄋ", "l" => "ㄌ", "g" => "ㄍ", "k" => "ㄎ", "h" => "ㄏ", "j" => "ㄐ", "q" => "ㄑ",
        "x" => "ㄒ", "zh" => "ㄓ", "ch" => "ㄔ", "sh" => "ㄕ", "r" => "ㄖ", "z" => "ㄗ", "c" => "ㄘ",
        "s" => "ㄙ", "w" => "ㄨ", "y" => "ㄧ", "1" => "", "2" => "ˊ", "3" => "ˇ", "4" => "ˋ", "0" => "˙", "" => "",
        _ => "",
    }
}

fn tone_char(base: char, tone: u8) -> char {
    match (base, tone) {
        ('a', 0) => 'a', ('o', 0) => 'o', ('e', 0) => 'e', ('i', 0) => 'i', ('u', 0) => 'u', ('v', 0) => 'ü',
        ('a', 1) => 'ā', ('o', 1) => 'ō', ('e', 1) => 'ē', ('i', 1) => 'ī', ('u', 1) => 'ū', ('v', 1) => 'ǖ',
        ('a', 2) => 'á', ('o', 2) => 'ó', ('e', 2) => 'é', ('i', 2) => 'í', ('u', 2) => 'ú', ('v', 2) => 'ǘ',
        ('a', 3) => 'ǎ', ('o', 3) => 'ǒ', ('e', 3) => 'ě', ('i', 3) => 'ǐ', ('u', 3) => 'ǔ', ('v', 3) => 'ǚ',
        ('a', 4) => 'à', ('o', 4) => 'ò', ('e', 4) => 'è', ('i', 4) => 'ì', ('u', 4) => 'ù', ('v', 4) => 'ǜ',
        _ => base,
    }
}

fn has_initial(s: &str) -> bool {
    !matches!(s.as_bytes().first().copied(), Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'v'))
}

fn phonetic(raw: &str) -> String {
    let mut s = raw.to_string();
    let tone = s.pop().unwrap();
    if let Some(alt) = local(&s) {
        s = alt.to_string();
    }
    s.push(tone);
    let len = s.len();
    let split = if !has_initial(&s) {
        vec!["".to_string(), s[..len - 1].to_string(), s[len - 1..].to_string()]
    } else {
        let i = if s.len() > 2 && s.as_bytes()[1] == b'h' { 2 } else { 1 };
        vec![s[..i].to_string(), s[i..len - 1].to_string(), s[len - 1..].to_string()]
    };
    let weak = split[2] == "0";
    let mut ret = String::new();
    if weak {
        ret.push_str(symbol(&split[2]));
    }
    ret.push_str(symbol(&split[0]));
    ret.push_str(symbol(&split[1]));
    if !weak {
        ret.push_str(symbol(&split[2]));
    }
    ret
}

fn unicode(raw: &str) -> String {
    let mut prefix = String::new();
    let len = raw.len();
    let finale = if !has_initial(raw) {
        raw[..len - 1].to_string()
    } else {
        let i = if raw.len() > 2 && raw.as_bytes()[1] == b'h' { 2 } else { 1 };
        prefix.push_str(&raw[..i]);
        raw[i..len - 1].to_string()
    };
    let offset = matches!(finale.as_str(), "ui" | "iu" | "uan" | "uang" | "ian" | "iang" | "ua" | "ie" | "uo" | "iong" | "iao" | "ve" | "ia") as usize;
    if offset == 1 {
        prefix.push_str(&finale[..1]);
    }
    let tone = raw.as_bytes()[raw.len() - 1] - b'0';
    let vowel = finale.chars().nth(offset).unwrap();
    prefix.push(tone_char(vowel, tone));
    let skip = finale.chars().take(offset + 1).map(|c| c.len_utf8()).sum::<usize>();
    if finale.len() > skip {
        prefix.push_str(&finale[skip..]);
    }
    prefix
}
