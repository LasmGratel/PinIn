use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::hint::black_box;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Duration;

// Adjust these imports to match your actual crate layout
use pinin::{
    searchers::{CachedSearcher, Logic, Searcher, SimpleSearcher, TreeSearcher},
    PinIn,
};

const SEARCH_TOKENS: &[&str] = &["boli", "yangmao", "hongse"];

fn load_test_data(source: &str) -> Vec<String> {
    let path = Path::new("benches").join(format!("{}.txt", source));
    let file = File::open(&path)
        .unwrap_or_else(|e| panic!("Failed to open test data '{}': {}", path.display(), e));
    BufReader::new(file)
        .lines()
        .map(|l| l.expect("Failed to read line"))
        .filter(|l| !l.is_empty())
        .collect()
}

fn build_tree_searcher(data: &[String], logic: Logic) -> TreeSearcher<usize> {
    let mut searcher = TreeSearcher::new(logic, PinIn::new());
    for (i, s) in data.iter().enumerate() {
        searcher.put(s, i);
    }
    searcher
}

fn build_cached_searcher(data: &[String], logic: Logic) -> CachedSearcher<usize> {
    let mut searcher = CachedSearcher::new(logic, PinIn::new());
    for (i, s) in data.iter().enumerate() {
        searcher.put(s, i);
    }
    searcher
}

fn build_simple_searcher(data: &[String], logic: Logic) -> SimpleSearcher<usize> {
    let mut searcher = SimpleSearcher::new(logic, PinIn::new());
    for (i, s) in data.iter().enumerate() {
        searcher.put(s, i);
    }
    searcher
}

fn run_search<S: Searcher<usize>>(searcher: &mut S) -> Vec<Vec<usize>> {
    SEARCH_TOKENS
        .iter()
        .map(|token| searcher.search(token).into_iter().copied().collect())
        .collect()
}

fn bench_searchers(c: &mut Criterion) {
    let datasets = [
        ("small", load_test_data("small")),
        ("large", load_test_data("large")),
    ];
    let logics = [Logic::Begin, Logic::Contain, Logic::Equal];

    for (dataset_name, data) in &datasets {
        for logic in &logics {
            let param = format!("{}/{:?}", dataset_name, logic);

            // --- TreeSearcher ---
            let mut searcher = build_tree_searcher(data, *logic);
            c.bench_function("tree_searcher", |b| {
                b.iter(|| black_box(run_search(&mut searcher)))
            });

            // --- CachedSearcher ---
            let mut searcher = build_cached_searcher(data, *logic);
            c.bench_function("cached_searcher", |b| {
                b.iter(|| black_box(run_search(&mut searcher)))
            });

            // --- SimpleSearcher ---
            let mut searcher = build_simple_searcher(data, *logic);
            c.bench_function("simple_searcher", |b| {
                b.iter(|| black_box(run_search(&mut searcher)))
            });
        }
    }
}

criterion_group! {
    name = pinin_benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2))
        .sample_size(10);
    targets = bench_searchers
}
criterion_main!(pinin_benches);
