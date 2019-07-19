use std::fmt;

use crate::event::Event;

pub struct Events {
    pub(crate) inner: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Iter<'a> {
    inner: &'a Events,
    pos: usize,
}

#[derive(Debug)]
pub struct IntoIter {
    inner: Events,
    pos: usize,
}

impl Events {
    pub fn with_capacity(capacity: usize) -> Events {
        Events {
            inner: Vec::with_capacity(capacity),
        }
    }

    #[deprecated(
        since = "0.6.10",
        note = "Index access removed in favor of iterator only API."
    )]
    #[doc(hidden)]
    pub fn get(&self, idx: usize) -> Option<Event> {
        self.inner.get(idx).cloned()
    }

    #[doc(hidden)]
    #[deprecated(
        since = "0.6.10",
        note = "Index access removed in favor of iterator only API."
    )]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> Iter {
        Iter {
            inner: self,
            pos: 0,
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl<'a> IntoIterator for &'a Events {
    type Item = Event;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        let ret = self.inner.inner.get(self.pos);
        self.pos += 1;
        ret.cloned()
    }
}

impl IntoIterator for Events {
    type Item = Event;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self,
            pos: 0,
        }
    }
}

impl Iterator for IntoIter {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        let ret = self.inner.inner.get(self.pos);
        self.pos += 1;
        ret.cloned()
    }
}

impl fmt::Debug for Events {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Events")
            .field("capacity", &self.capacity())
            .finish()
    }
}
