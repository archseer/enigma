use std::collections::VecDeque;

use crate::value::Term;

#[derive(Debug, Default)]
pub struct Mailbox {
    queue: VecDeque<Term>,

    /// Save pointer to track position to the current offset when scanning through the mailbox.
    save: usize,
}

impl Mailbox {
    pub fn new() -> Self {
        Mailbox {
            queue: VecDeque::new(),
            save: 0,
        }
    }

    pub fn send(&mut self, message: Term) {
        self.queue.push_back(message);
    }

    pub fn receive(&mut self) -> Option<&Term> {
        self.queue.get(self.save)
    }

    pub fn advance(&mut self) {
        self.save += 1;
    }

    pub fn reset(&mut self) {
        self.save += 0;
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
}
