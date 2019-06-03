use std::collections::VecDeque;

use crate::value::Term;

#[derive(Debug, Default)]
pub struct Mailbox {
    queue: VecDeque<Term>,

    /// Save pointer to track position to the current offset when scanning through the mailbox.
    save: usize,
    mark: Option<usize>,
}

impl Mailbox {
    pub fn new() -> Self {
        Mailbox {
            queue: VecDeque::new(),
            save: 0,
            mark: None,
        }
    }

    pub fn send(&mut self, message: Term) {
        self.queue.push_back(message);
    }

    pub fn receive(&mut self) -> Option<Term> {
        self.queue.get(self.save).copied()
    }

    // recv_mark
    pub fn mark(&mut self) {
        self.mark = Some(self.save);
    }

    // recv_set
    pub fn set(&mut self) {
        if let Some(mark) = self.mark {
            self.save = mark;
        }
    }

    pub fn advance(&mut self) {
        self.save += 1;
    }

    pub fn reset(&mut self) {
        self.save = 0;
        self.mark = None;
    }

    /// We use a reference when we receive, because of pattern matching.
    /// Once we're done with a message, we have to specifically pop
    pub fn remove(&mut self) {
        self.queue.remove(self.save);
    }

    pub fn has_messages(&self) -> bool {
        !self.queue.is_empty()

        // TODO: take sig queue in account?
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}
