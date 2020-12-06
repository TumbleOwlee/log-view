use arraydeque::behavior::Wrapping;
use arraydeque::{Array, ArrayDeque};

use crate::source::{Source, TryRead};

pub struct SourceBuffer<A: Array<Item = I>, I: Send> {
    buffer: ArrayDeque<A, Wrapping>,
    handle: Source<I>,
}

impl<'a, A: Array<Item = I>, I: 'a + Send + Clone> SourceBuffer<A, I> {
    pub fn new(handle: Source<A::Item>) -> Self {
        Self {
            buffer: ArrayDeque::new(),
            handle,
        }
    }

    pub fn update(&'a mut self) -> Option<A::Item> {
        if let Some(c) = self.handle.try_read() {
            self.buffer.push_back(c.clone());
            Some(c)
        } else {
            None
        }
    }

    pub fn iter(&'a self) -> arraydeque::Iter<I> {
        self.buffer.iter()
    }
}
