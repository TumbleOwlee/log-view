use arraydeque::behavior::Wrapping;
use arraydeque::ArrayDeque;

use crate::source::{Source, TryRead};

pub struct SourceBuffer<A: Send> {
    buffer: ArrayDeque<A, 1024, Wrapping>,
    handle: Source<A>,
}

impl<'a, A: 'a + Send + Clone> SourceBuffer<A> {
    pub fn new(handle: Source<A>) -> Self {
        Self {
            buffer: ArrayDeque::new(),
            handle,
        }
    }

    pub fn update(&'a mut self) -> Option<A> {
        if let Some(c) = self.handle.try_read() {
            self.buffer.push_back(c.clone());
            Some(c)
        } else {
            None
        }
    }

    pub fn iter(&'a self) -> arraydeque::Iter<A> {
        self.buffer.iter()
    }
}
