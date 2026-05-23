use std::collections::HashMap;

use crate::searchers::{Logic, Searcher, SimpleSearcher};
use crate::PinIn;

pub struct CachedSearcher<T> {
    inner: SimpleSearcher<T>,
    all: Vec<usize>,
    scale: f32,
    len_cached: usize,
    max_cached: usize,
    total: usize,
    stats: Stats<String>,
    cache: HashMap<String, Vec<usize>>,
}

impl<T> CachedSearcher<T> {
    pub fn new(logic: Logic, context: PinIn) -> Self {
        Self::with_scale(logic, context, 1.0)
    }

    pub fn with_scale(logic: Logic, context: PinIn, scale: f32) -> Self {
        Self {
            inner: SimpleSearcher::new(logic, context),
            all: Vec::new(),
            scale,
            len_cached: 0,
            max_cached: 0,
            total: 0,
            stats: Stats::default(),
            cache: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.inner.reset();
        self.stats.reset();
        self.len_cached = 0;
        self.max_cached = 0;
        self.cache.clear();
    }

    fn filter(&mut self, name: &str) -> Vec<usize> {
        if name.is_empty() {
            return self.all.clone();
        }
        self.stats.count(name.to_string());
        if let Some(v) = self.cache.get(name) {
            return v.clone();
        }
        let base = self.filter(&name[..name.len() - 1]);
        if self.cache.len() >= self.max_cached {
            if let Some(least) = self.stats.least(self.cache.keys().cloned().collect(), name.to_string()) {
                if least != name {
                    self.cache.remove(&least);
                } else {
                    return base;
                }
            }
        }
        let filter = self.inner.logic.filter_logic();
        let context = self.inner.context.clone();
        let ret: Vec<usize> = base.into_iter().filter(|&i| filter.test(&context, &self.inner.names[i], name)).collect();
        self.cache.insert(name.to_string(), ret.clone());
        ret
    }
}

impl<T> Searcher<T> for CachedSearcher<T> {
    fn put(&mut self, name: &str, identifier: T) {
        self.reset();
        self.total += name.chars().count();
        self.all.push(self.all.len());
        self.inner.put(name, identifier);
    }

    fn search<'a>(&'a mut self, name: &str) -> Vec<&'a T> {
        self.inner.renew();
        if self.all.is_empty() {
            return Vec::new();
        }
        if self.max_cached == 0 {
            let total_search = if self.inner.logic == Logic::Contain { self.total.max(1) as f32 } else { self.all.len().max(1) as f32 };
            self.max_cached = (self.scale * (2.0 * total_search.log2().ceil() + 16.0)).ceil() as usize;
            self.max_cached = self.max_cached.max(1);
        }
        if self.len_cached == 0 {
            self.len_cached = (self.max_cached as f32).log(8.0).ceil() as usize;
        }
        let prefix = &name[..name.len().min(self.len_cached)];
        let is = self.filter(prefix);
        let context = self.inner.context.clone();
        let tested: Vec<usize> = if self.inner.logic == Logic::Equal || name.len() > self.len_cached {
            is.into_iter().filter(|&i| self.inner.logic.test(&context, &self.inner.names[i], name)).collect()
        } else {
            is
        };
        tested.into_iter().map(|i| &self.inner.objects[i]).collect()
    }

    fn context(&self) -> PinIn {
        self.inner.context()
    }
}

#[derive(Default)]
struct Stats<T> {
    data: HashMap<T, usize>,
}

impl<T: std::hash::Hash + Eq + Clone> Stats<T> {
    fn count(&mut self, key: T) {
        let cnt = self.data.get(&key).copied().unwrap_or(0).saturating_add(1);
        self.data.insert(key, cnt);
        if cnt == usize::MAX {
            for v in self.data.values_mut() {
                *v /= 2;
            }
        }
    }

    fn least(&self, keys: Vec<T>, extra: T) -> Option<T> {
        let mut ret = extra.clone();
        let mut cnt = self.data.get(&extra).copied().unwrap_or(0);
        for key in keys {
            let value = self.data.get(&key).copied().unwrap_or(0);
            if value < cnt {
                ret = key;
                cnt = value;
            }
        }
        Some(ret)
    }

    fn reset(&mut self) {
        self.data.clear();
    }
}
