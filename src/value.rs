#[derive(Debug, PartialEq)]
pub enum Value {
    None(), // also known as nil
    Integer(i64),
    Atom(usize),
    Catch(),
    // external vals? except Pid can also be internal
    Pid(),
    Port(),
    Ref(),
    // continuation pointer?
    Cons {
        head: Box<Value>,
        tail: Box<Value>,
    }, // two values
    /// Boxed values
    Tuple(),
    Float(f64),
    /// Strings use an Arc so they can be sent to other processes without
    /// requiring a full copy of the data.
    //Binary(ArcWithoutWeak<ImmutableString>),

    /// An interned string is a string allocated on the permanent space. For
    /// every unique interned string there is only one object allocated.
    //InternedBinary(ArcWithoutWeak<ImmutableString>),
    // BigInt(Box<BigInt>),
    Closure(),
    // Import(), Export(),
}

// TODO: maybe box binaries further:
// // contains size, followed in memory by the data bytes
// ProcBin { nbytes: Word } ,
// // contains reference to heapbin
// RefBin,
// // stores data on a separate heap somewhere else with refcount
// HeapBin { nbytes: Word, refc: Word },

impl Value {
    pub fn is_atom(&self) -> bool {
        match *self {
            Value::Atom(_) => true,
            _ => false,
        }
    }
}
