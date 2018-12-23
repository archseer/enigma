use crate::value::Value;
use parking_lot::Mutex;
use std::collections::VecDeque;

pub struct Mailbox {
    /// Internal mailbox from which the process is safe to read.
    internal: VecDeque<*const Value>,

    /// External mailbox, to which other processes can write (while holding the lock)
    external: VecDeque<*const Value>,

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

    pub fn send_external(&mut self, message: *const Value) {
        let _lock = self.write_lock.lock();

        self.external.push_back(message);
    }

    pub fn send_internal(&mut self, message: *const Value) {
        self.internal.push_back(message);
    }

    pub fn receive(&mut self) -> Option<&*const Value> {
        if self.internal.len() > self.save {
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
