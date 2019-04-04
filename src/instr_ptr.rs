use crate::loader::{FuncInfo, LINE_INVALID_LOCATION};
use crate::module::{Module, MFA};
use crate::value::{self, Term, TryFrom};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Debug)]
pub struct InstrPtr {
    /// Module containing the instruction set.
    pub module: *const Module,
    /// Offset to the current instruction.
    pub ptr: u32,
}

unsafe impl Send for InstrPtr {}
unsafe impl Sync for InstrPtr {}

impl InstrPtr {
    pub fn new(module: *const Module, ptr: u32) -> Self {
        InstrPtr { module, ptr }
    }

    pub fn get_module<'a, 'b>(&'a self) -> &'b Module {
        unsafe { &(*self.module) }
    }

    // typedef struct {
    //     ErtsCodeMFA* mfa;		/* Pointer to: Mod, Name, Arity */
    //     Uint needed;		/* Heap space needed for entire tuple */
    //     Uint32 loc;			/* Location in source code */
    //     Eterm* fname_ptr;		/* Pointer to fname table */
    // } FunctionInfo;

    /// Find a function from the given pc and fill information in
    /// the FunctionInfo struct. If the full_info is non-zero, fill
    /// in all available information (including location in the
    /// source code).
    pub fn lookup_func_info(&self) -> Option<(MFA, Option<FuncInfo>)> {
        let module = unsafe { &(*self.module) };

        let mut vec: Vec<(&(u32, u32), &u32)> = module.funs.iter().collect();
        vec.sort_by(|(_, v1), (_, v2)| v1.cmp(v2));

        let mut low: u32 = 0;
        let mut high = (vec.len() - 1) as u32;

        while low < high {
            let mid = low + (high - low) / 2;
            if self.ptr < *vec[mid as usize].1 {
                high = mid;
            } else if self.ptr < *vec[(mid + 1) as usize].1 {
                let ((f, a), _fun_offset) = vec[mid as usize];
                let mfa = MFA(module.name, *f, *a);
                let func_info = self.lookup_loc();
                return Some((mfa, func_info));
            } else {
                low = mid + 1;
            }
        }
        None
    }

    pub fn lookup_loc(&self) -> Option<FuncInfo> {
        // TODO limit search scope in the future by searching between (current func, currentfunc+1);
        let module = unsafe { &(*self.module) };

        let mut low = 0;
        let mut high = module.lines.len() - 1;

        while high > low {
            let mid = low + (high - low) / 2;
            if self.ptr < module.lines[mid].1 {
                high = mid;
            } else if self.ptr < module.lines[mid + 1].1 {
                let res = module.lines[mid];

                if res == LINE_INVALID_LOCATION {
                    return None;
                }

                return Some(res);
            } else {
                low = mid + 1;
            }
        }
        None
    }
}

// TODO: these are kinda messy since Opt<ptr> vs ptr deboxes differently

// TODO: to be TryFrom once rust stabilizes the trait
impl TryFrom<Term> for value::Boxed<Option<InstrPtr>> {
    type Error = value::WrongBoxError;

    #[inline]
    fn try_from(value: &Term) -> Result<&Self, value::WrongBoxError> {
        if let value::Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == value::BOXED_CP {
                    return Ok(&*(ptr as *const value::Boxed<Option<InstrPtr>>));
                }
            }
        }
        Err(value::WrongBoxError)
    }
}
// TODO: to be TryFrom once rust stabilizes the trait
impl TryFrom<Term> for value::Boxed<InstrPtr> {
    type Error = value::WrongBoxError;

    #[inline]
    fn try_from(value: &Term) -> Result<&Self, value::WrongBoxError> {
        if let value::Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == value::BOXED_CATCH {
                    return Ok(&*(ptr as *const value::Boxed<InstrPtr>));
                }
            }
        }
        Err(value::WrongBoxError)
    }
}
