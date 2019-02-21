use intrusive_collections::{IntrusivePointer, KeyAdapter, RBTree, RBTreeLink};

use super::PID;
use crate::servo_arc::Arc;

// Implement for servo_arc, since arc != servo_arc
unsafe impl<T: ?Sized> IntrusivePointer<T> for Arc<T> {
    #[inline]
    fn into_raw(self) -> *const T {
        let ptr = self.as_ref() as *const T;
        std::mem::forget(self);
        ptr
    }
    #[inline]
    unsafe fn from_raw(ptr: *const T) -> Arc<T> {
        // Create a temporary fake Arc object from the given pointer and
        // calculate the address of the inner T.
        let fake_rc: Self = std::mem::transmute(ptr);
        let fake_rc_target = fake_rc.as_ref() as *const _;
        std::mem::forget(fake_rc);

        // Calculate the offset of T in ArcInner<T>
        let rc_offset = (fake_rc_target as *const u8) as isize - (ptr as *const u8) as isize;

        // Get the address of the ArcInner<T> by subtracting the offset from the
        // pointer we were originally given.
        let rc_ptr = (ptr as *const u8).offset(-rc_offset);

        // If T is an unsized type, then *const T is a fat pointer. To handle
        // this case properly we need to preserve the second word of the fat
        // pointer but overwrite the first one with our adjusted pointer.
        let mut result = std::mem::transmute(ptr);
        std::ptr::write(&mut result as *mut _ as *mut *const u8, rc_ptr);
        result
    }
}

#[derive(Default, Debug)]
pub struct Node {
    pub link: RBTreeLink,
    pub other: PID, // maybe Term<PID>
                    // TODO: other can be proc/port/dist proc
}
// original structure had a link a, link b, but we can't do that here
// so we allocate two nodes, one on each collection ¯\_(ツ)_/¯

intrusive_adapter!(pub NodeAdapter = Arc<Node>: Node { link: RBTreeLink });

impl<'a> KeyAdapter<'a> for NodeAdapter {
    type Key = PID;
    fn get_key(&self, x: &'a Node) -> PID {
        x.other
    }
}

// TODO: benchmark vs a BTreeMap<PID, ()>
// also https://github.com/orium/rpds RBTreeMap
pub type Tree = RBTree<NodeAdapter>;

pub type Link = RBTreeLink;
