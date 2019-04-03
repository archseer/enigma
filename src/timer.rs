//use std::ptr;
use crate::process::Ref;
use hashbrown::HashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

#[derive(Debug)]
pub struct TimerTable {
    /// Direct mapping string to atom index
    timers: RwLock<HashMap<String, u32>>,
}

/// Stores atom lookup tables.
impl TimerTable {
    pub fn new() -> TimerTable {
        TimerTable {
            timers: HashMap::new(),
        }
    }

    pub fn register(&mut self, reference: process::Ref, timer: T) -> &T {
        // create a timer that will run for <delay>
        // need to either send a timeout message after that
        // or send a message to another process
        // --> can be abstracted by taking a pid + message, for timeout just send to self and wrap
        // in a tuple.
        // the timer needs to unregister itself on completion
        //
        // this is kind of annoying, I'd prefer to not keep a global store since tokio already
        // manages things for us.
        //instead, let's box a tokio::timer?
        self.timers.entry(atom).or_insert(process)
    }

    pub fn unregister(&mut self, atom: u32) -> Option<T> {
        self.timers.remove(&atom)
    }
}
