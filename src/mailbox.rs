use crate::value::Value;
use parking_lot::Mutex;
use std::vec::Vec;

pub struct Mailbox {
    /// Internal mailbox from which the process is safe to read.
    internal: Vec<*const Value>,

    /// External mailbox, to which other processes can write (while holding the lock)
    external: Vec<*const Value>,

    /// Used for synchronizing writes to the external part.
    write_lock: Mutex<()>,
}

impl Mailbox {
    pub fn new() -> Self {
        Mailbox {
            internal: Vec::new(),
            external: Vec::new(),
            write_lock: Mutex::new(()),
        }
    }

    pub fn send_external(&mut self, message: *const Value) {
        let _lock = self.write_lock.lock();

        self.external.push(message);
    }

    pub fn send_internal(&mut self, message: *const Value) {
        self.internal.push(message);
    }

    pub fn receive(&mut self) -> Option<*const Value> {
        if self.internal.is_empty() {
            let _lock = self.write_lock.lock();

            self.internal.append(&mut self.external);
        }

        self.internal.pop()
    }

    pub fn has_messages(&self) -> bool {
        if !self.internal.is_empty() {
            return true;
        }

        let _lock = self.write_lock.lock();

        !self.external.is_empty()
    }
}
