use parking_lot::Mutex;
use std::collections::VecDeque;

use crate::exception::Exception;
use crate::value::Term;

pub enum Signal {
    Exit(Exception),
    Message(Term),
}

#[derive(Default)]
pub struct Mailbox {
    /// Internal mailbox from which the process is safe to read.
    /// It only holds messages, other signals are processed as we read the external queue.
    internal: VecDeque<Term>,

    /// External mailbox, to which other processes can write (while holding the lock)
    /// It holds a mixture of different signals and messages.
    external: VecDeque<Signal>,

    /// Used for synchronizing writes to the external part.
    write_lock: Mutex<()>,

    /// Save pointer to track position to the current offset when scanning through the mailbox.
    save: usize,
}

impl Mailbox {
    pub fn new() -> Self {
        Mailbox {
            internal: VecDeque::new(),
            external: VecDeque::new(),
            write_lock: Mutex::new(()),
            save: 0,
        }
    }

    pub fn send_external(&mut self, message: Signal) {
        let _lock = self.write_lock.lock();

        self.external.push_back(message);
    }

    pub fn send_internal(&mut self, message: Term) {
        self.internal.push_back(message);
    }

    pub fn receive(&mut self) -> Option<&Signal> {
        if self.internal.len() >= self.save {
            let _lock = self.write_lock.lock();

            self.internal.append(&mut self.external);
        }

        self.internal.get(self.save)
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
        self.internal.remove(self.save);
    }

    pub fn has_messages(&self) -> bool {
        if !self.internal.is_empty() {
            return true;
        }

        let _lock = self.write_lock.lock();

        !self.external.is_empty()
    }
}
