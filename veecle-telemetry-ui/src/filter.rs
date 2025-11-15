use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Deref;

use veecle_telemetry::protocol::ThreadId;

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
    pub thread: SetFilter<ThreadId>,
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
                && self.thread.matches(&log.thread_id)
        })
    }

    /// Check if a span matches current filters
    pub fn span_matches(&self, span: &SpanRef) -> bool {
        self.target.matches(&span.metadata.target)
            && self
                .file
                .matches(span.metadata.file.as_deref().unwrap_or_default())
            && self.actor.matches(&span.actor)
            && self.thread.matches(&span.thread_id)
    }

    /// Check if any filters are activate
    pub fn has_active_filters(&self) -> bool {
        !self.target.is_empty()
            || !self.file.is_empty()
            || !self.actor.is_empty()
            || !self.level.is_empty()
            || !self.message.is_empty()
            || !self.thread.is_empty()
    }
}
