use hashbrown::HashMap;

// pub type RcRegistry = Arc<Mutex<Registry<RcProcess>>>;

/// A registry for handling processes with registered (atom) names.
#[derive(Default)]
pub struct Registry<T: Clone> {
    processes: HashMap<u32, T>,
}
impl<T: Clone> Registry<T> {
    pub fn new() -> Self {
        Registry {
            processes: HashMap::new(),
        }
    }
    // pub fn with_rc() -> RcRegistry {
    //     Arc::new(Mutex::new(Self::new()))
    // }

    pub fn register(&mut self, atom: u32, process: T) -> &T {
        self.processes.entry(atom).or_insert(process)
    }

    pub fn unregister(&mut self, atom: u32) -> Option<T> {
        self.processes.remove(&atom)
    }

    pub fn whereis(&self, atom: u32) -> Option<&T> {
        self.processes.get(&atom)
    }
}

/*
 * Register a process or port (can't be registered twice).
 * Returns 0 if name, process or port is already registered.
 *
 * When smp support is enabled:
 *   * Assumes that main lock is locked (and only main lock)
 *     on c_p.
 *
 */
// int erts_register_name(Process *c_p, Eterm name, Eterm id)
// {
//     int res = 0;
//     Process *proc = NULL;
//     Port *port = NULL;
//     RegProc r, *rp;
//     ERTS_CHK_HAVE_ONLY_MAIN_PROC_LOCK(c_p);

//     if (is_not_atom(name) || name == am_undefined)
// 	return res;

//     if (c_p->common.id == id) /* A very common case I think... */
// 	proc = c_p;
//     else {
// 	if (is_not_internal_pid(id) && is_not_internal_port(id))
// 	    return res;
// 	erts_proc_unlock(c_p, ERTS_PROC_LOCK_MAIN);
// 	if (is_internal_port(id)) {
// 	    port = erts_id2port(id);
// 	    if (!port)
// 		goto done;
// 	}
//     }

//     {
// 	ErtsProcLocks proc_locks = proc ? ERTS_PROC_LOCK_MAIN : 0;
// 	reg_safe_write_lock(proc, &proc_locks);

// 	if (proc && !proc_locks)
// 	    erts_proc_lock(c_p, ERTS_PROC_LOCK_MAIN);
//     }

//     if (is_internal_pid(id)) {
// 	if (!proc)
// 	    proc = erts_pid2proc(NULL, 0, id, ERTS_PROC_LOCK_MAIN);
// 	r.p = proc;
// 	if (!proc)
// 	    goto done;
// 	if (proc->common.u.alive.reg)
// 	    goto done;
// 	r.pt = NULL;
//     }
//     else {
// 	ASSERT(!INVALID_PORT(port, id));
// 	ERTS_LC_ASSERT(erts_lc_is_port_locked(port));
// 	r.pt = port;
// 	if (r.pt->common.u.alive.reg)
// 	    goto done;
// 	r.p = NULL;
//     }

//     r.name = name;

//     rp = (RegProc*) hash_put(&process_reg, (void*) &r);
//     if (proc && rp->p == proc) {
// 	if (IS_TRACED_FL(proc, F_TRACE_PROCS)) {
// 	    trace_proc(proc, ERTS_PROC_LOCK_MAIN,
//                        proc, am_register, name);
// 	}
// 	proc->common.u.alive.reg = rp;
//     }
//     else if (port && rp->pt == port) {
//     	if (IS_TRACED_FL(port, F_TRACE_PORTS)) {
// 		trace_port(port, am_register, name);
// 	}
// 	port->common.u.alive.reg = rp;
//     }

//     if ((rp->p && rp->p->common.id == id)
// 	|| (rp->pt && rp->pt->common.id == id)) {
// 	res = 1;
//     }

//  done:
//     reg_write_unlock();
//     if (port)
// 	erts_port_release(port);
//     if (c_p != proc) {
// 	if (proc)
// 	    erts_proc_unlock(proc, ERTS_PROC_LOCK_MAIN);
// 	erts_proc_lock(c_p, ERTS_PROC_LOCK_MAIN);
//     }
//     return res;
// }
