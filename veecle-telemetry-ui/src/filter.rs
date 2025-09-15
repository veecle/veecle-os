use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Deref;

use crate::store::{Level, LogRef, SpanRef, Store};

#[derive(Default, Debug)]
pub struct StringFilter {
    string: String,
}

impl Deref for StringFilter {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl StringFilter {
    pub fn set(&mut self, string: String) {
        self.string = string;
    }

    pub fn matches(&self, value: &str) -> bool {
        self.string.is_empty() || value.to_lowercase().contains(&self.string.to_lowercase())
    }
}

#[derive(Debug)]
pub struct SetFilter<T> {
    set: HashSet<T, std::hash::RandomState>,
}

impl<T> Default for SetFilter<T> {
    fn default() -> Self {
        Self {
            set: HashSet::default(),
        }
    }
}

impl<T> SetFilter<T>
where
    T: Eq + Hash,
{
    pub fn set(&mut self, set: HashSet<T>) {
        self.set = set;
    }

    pub fn matches(&self, value: &T) -> bool {
        self.set.is_empty() || self.set.contains(value)
    }
}

impl<T> Deref for SetFilter<T> {
    type Target = HashSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

#[derive(Default, Debug)]
pub struct Filters {
    pub level: SetFilter<Level>,
    pub target: StringFilter,
    pub file: StringFilter,

    pub message: StringFilter,

    pub actor: SetFilter<String>,
}

impl Filters {
    pub fn clear(&mut self) {
        *self = Default::default();
    }
}

impl Filters {
    pub fn filter_logs<'a>(&'a self, store: &'a Store) -> impl Iterator<Item = LogRef<'a>> {
        store.logs().filter(|log| {
            self.level.matches(&log.metadata.level)
                && self.target.matches(&log.metadata.target)
                && self
                    .file
                    .matches(log.metadata.file.as_deref().unwrap_or_default())
                && self.actor.matches(&log.actor)
                && self.message.matches(&log.body)
        })
    }

    pub fn filter_root_spans<'a>(&'a self, store: &'a Store) -> impl Iterator<Item = SpanRef<'a>> {
        store
            .root_spans()
            .filter(|span| self.actor.is_empty() || self.actor.matches(&span.actor))
    }
}
