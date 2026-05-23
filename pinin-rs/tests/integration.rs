use ::pinin::{CachedSearcher, DAQIAN, DefaultLoader, DictLoader, Logic, PinIn, PinyinFormat, Searcher, SimpleSearcher, TreeSearcher, XIAOHE, ZIRANMA};

fn ints(v: Vec<&i32>) -> Vec<i32> {
    v.into_iter().copied().collect()
}

#[test]
fn quanpin() {
    let p = PinIn::new();
    assert!(p.contains("测试文本", "ceshiwenben"));
    assert!(p.contains("测试文本", "ceshiwenbe"));
    assert!(p.contains("测试文本", "ceshiwben"));
    assert!(p.contains("测试文本", "ce4shi4wb"));
    assert!(!p.contains("测试文本", "ce2shi4wb"));
    assert!(p.contains("合金炉", "hejinlu"));
    assert!(p.contains("洗矿场", "xikuangchang"));
    assert!(p.contains("流体", "liuti"));
    assert!(p.contains("轰20", "hong2"));
    assert!(p.contains("hong2", "hong2"));
    assert!(!p.begins("测", "ce4a"));
    assert!(!p.begins("", "a"));
    assert!(p.contains("石头", "stou"));
    assert!(p.contains("安全", "aquan"));
    assert!(p.contains("昂扬", "ayang"));
    assert!(!p.contains("昂扬", "anyang"));
    assert!(p.contains("昂扬", "angyang"));
}

#[test]
fn daqian() {
    let p = PinIn::new().config().keyboard(DAQIAN).commit();
    assert!(p.contains("测试文本", "hk4g4jp61p3"));
    assert!(p.contains("测试文本", "hkgjp1"));
    assert!(p.contains("錫", "vu6"));
    assert!(p.contains("鑽石", "yj0"));
    assert!(p.contains("物質", "j456"));
    assert!(p.contains("腳手架", "rul3g.3ru84"));
    assert!(p.contains("鵝", "k6"));
    assert!(p.contains("葉", "u,4"));
    assert!(p.contains("共同", "ej/wj/"));
}

#[test]
fn xiaohe_and_ziranma() {
    let p = PinIn::new().config().keyboard(XIAOHE).commit();
    assert!(p.contains("测试文本", "ceuiwfbf"));
    assert!(p.contains("测试文本", "ceuiwf2"));
    assert!(!p.contains("测试文本", "ceuiw2"));
    assert!(p.contains("合金炉", "hej"));
    assert!(p.contains("洗矿场", "xikl4"));
    assert!(p.contains("月球", "ytqq"));

    let p = PinIn::new().config().keyboard(ZIRANMA).commit();
    assert!(p.contains("测试文本", "ceuiwfbf"));
    assert!(p.contains("测试文本", "ceuiwf2"));
    assert!(!p.contains("测试文本", "ceuiw2"));
    assert!(p.contains("合金炉", "hej"));
    assert!(p.contains("洗矿场", "xikd4"));
    assert!(p.contains("月球", "ytqq"));
    assert!(p.contains("安全", "anqr"));
}

#[test]
fn tree_and_context() {
    let mut tree = TreeSearcher::new(Logic::Contain, PinIn::new());
    tree.put("测试文本", 1);
    tree.put("测试切分", 5);
    tree.put("测试切分文本", 6);
    tree.put("合金炉", 2);
    tree.put("洗矿场", 3);
    tree.put("流体", 4);
    tree.put("轰20", 7);
    tree.put("hong2", 8);
    assert_eq!(ints(tree.search("ceshiwenben")), vec![1]);
    assert_eq!(ints(tree.search("ceshiwenbe")), vec![1]);
    assert_eq!(ints(tree.search("ceshiwben")), vec![1]);
    assert_eq!(ints(tree.search("ce4shi4wb")), vec![1]);
    assert!(tree.search("ce2shi4wb").is_empty());
    assert_eq!(ints(tree.search("hejinlu")), vec![2]);
    assert_eq!(ints(tree.search("xikuangchang")), vec![3]);
    assert_eq!(ints(tree.search("liuti")), vec![4]);
    assert_eq!(ints(tree.search("ceshi")), vec![1, 5, 6]);
    assert_eq!(ints(tree.search("ceshiqiefen")), vec![5, 6]);
    assert_eq!(ints(tree.search("ceshiqiefenw")), vec![6]);
    assert_eq!(ints(tree.search("hong2")), vec![7, 8]);

    let p = PinIn::new();
    let mut tree = TreeSearcher::new(Logic::Contain, p.clone());
    tree.put("测试文本", 0);
    tree.put("测试文字", 3);
    assert_eq!(ints(tree.search("ce4shi4wb")), vec![0]);
    assert_eq!(ints(tree.search("ce4shw")), vec![0, 3]);
    assert_eq!(ints(tree.search("ce4sw")), vec![0, 3]);
    assert!(tree.search("ce4siw").is_empty());
    p.config().f_sh2s(true).commit();
    assert_eq!(ints(tree.search("ce4siw")), vec![0, 3]);
    p.config().f_sh2s(false).keyboard(DAQIAN).commit();
    assert_eq!(ints(tree.search("hk4g4jp61p3")), vec![0]);
    assert!(tree.search("ce4shi4wb").is_empty());
}

#[test]
fn full_searchers() {
    let mut searchers: Vec<Box<dyn Searcher<i32>>> = vec![
        Box::new(TreeSearcher::new(Logic::Equal, PinIn::new())),
        Box::new(SimpleSearcher::new(Logic::Equal, PinIn::new())),
        Box::new(CachedSearcher::new(Logic::Equal, PinIn::new())),
    ];
    for s in &mut searchers {
        s.put("测试文本", 1);
        s.put("测试切分", 5);
        s.put("测试切分文本", 6);
        s.put("合金炉", 2);
        s.put("洗矿场", 3);
        s.put("流体", 4);
        s.put("轰20", 7);
        s.put("hong2", 8);
        s.put("月球", 9);
        s.put("汉化", 10);
        s.put("喊话", 11);
        assert_eq!(ints(s.search("hong2")), vec![8]);
        assert_eq!(ints(s.search("hong20")), vec![7]);
        assert_eq!(ints(s.search("ceshqf")), vec![5]);
        assert!(s.search("ceshqfw").is_empty());
        assert_eq!(ints(s.search("hh")), vec![10, 11]);
        assert!(s.search("hhu").is_empty());
    }
}

#[test]
fn format() {
    let pi = PinIn::new();
    let ch = pi.get_char('圆');
    let py = ch.pinyins()[0];
    assert_eq!(pi.format(py), "yuan2");
    assert_eq!(PinyinFormat::RAW.format_raw("yuan2"), "yuan");
    assert_eq!(PinyinFormat::UNICODE.format_raw("yuan2"), "yuán");
    assert_eq!(PinyinFormat::PHONETIC.format_raw("yuan2"), "ㄩㄢˊ");
    let pi = pi.config().format(PinyinFormat::PHONETIC).commit();
    assert_eq!(pi.format(pi.get_pinyin("le0")), "˙ㄌㄜ");
}

struct CustomLoader;
impl DictLoader for CustomLoader {
    fn load(&self, feed: &mut dyn FnMut(char, &[&str])) {
        DefaultLoader.load(feed);
        feed('\u{E900}', &["lu2"]);
    }
}

struct CpLoader;
impl DictLoader for CpLoader {
    fn load(&self, feed: &mut dyn FnMut(char, &[&str])) {
        DefaultLoader.load(feed);
    }
    fn load_code_points(&self, feed: &mut dyn FnMut(u32, &[&str])) {
        DefaultLoader.load_code_points(feed);
        feed("𫟼".chars().next().unwrap() as u32, &["ta2"]);
    }
}

#[test]
fn dicts() {
    let p = PinIn::new();
    let mut searcher = TreeSearcher::new(Logic::Contain, p.clone());
    searcher.put("\u{E900}锭", 0);
    assert!(!ints(searcher.search("lu2d")).contains(&0));
    assert!(!p.contains("\u{E900}", "lu2"));

    let p = PinIn::with_loader(&CustomLoader);
    let mut searcher = TreeSearcher::new(Logic::Contain, p.clone());
    searcher.put("\u{E900}锭", 0);
    assert!(ints(searcher.search("lu2d")).contains(&0));
    assert!(p.contains("\u{E900}", "lu2"));

    let p = PinIn::new();
    let mut searcher = TreeSearcher::new(Logic::Contain, p.clone());
    searcher.put("𫟼锭", 0);
    assert!(ints(searcher.search("da2d")).contains(&0));
    assert!(p.contains("𫟼", "da2"));
    assert!(!ints(searcher.search("ta2d")).contains(&0));
    assert!(!p.contains("𫟼", "ta2"));

    let p = PinIn::with_loader(&CpLoader);
    let mut searcher = TreeSearcher::new(Logic::Contain, p.clone());
    searcher.put("𫟼锭", 0);
    assert!(ints(searcher.search("ta2d")).contains(&0));
    assert!(p.contains("𫟼", "ta2"));
}

#[test]
fn fuzzy_and_accelerate() {
    let p = PinIn::new()
        .config()
        .f_sh2s(true)
        .f_zh2z(true)
        .f_ch2c(true)
        .f_ang2an(true)
        .f_eng2en(true)
        .f_u2v(true)
        .accelerate(true)
        .commit();
    assert!(p.contains("试", "si4"));
    assert!(p.contains("张", "zan"));
    assert!(p.contains("成", "cen"));
    assert!(p.contains("女", "nu3"));
}
