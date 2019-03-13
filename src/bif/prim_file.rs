use crate::atom;
use crate::bif;
use crate::bitstring::Binary;
use crate::exception::{Exception, Reason};
use crate::immix::Heap;
use crate::process::RcProcess;
use crate::value::{self, Cons, Term, TryFrom};
use crate::vm;
use std::fs;

fn error_to_tuple(heap: &Heap, error: std::io::Error) -> Term {
    use std::io::ErrorKind;
    let kind = match error.kind() {
        ErrorKind::NotFound => atom!(ENOENT),
        _ => unimplemented!("error_to_tuple for {:?}", error),
    };
    tup2!(heap, atom!(ERROR), kind)
}

pub fn get_cwd_nif_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;

    match std::env::current_dir() {
        Ok(path) => {
            let path = path.to_str().unwrap();
            let bin = Binary::from(path.as_bytes());

            Ok(tup2!(heap, atom!(OK), Term::binary(heap, bin)))
        }
        _ => Err(Exception::new(Reason::EXC_INTERNAL_ERROR)),
    }
    // TODO: make a function that converts io::Error to a tuple
}

/// Reads an entire file into \c result, stopping after \c size bytes or EOF. It will read until
/// EOF if size is 0.
pub fn read_file_nif_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // arg[0] = filename
    let heap = &process.context_mut().heap;

    // TODO bitstrings or non zero offsets can fail ...
    let cons = Cons::try_from(&args[0])?;
    let path = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    println!("Trying to read file {:?}", path);
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => return Ok(error_to_tuple(heap, err)),
    };

    Ok(tup2!(
        heap,
        atom!(OK),
        Term::binary(heap, Binary::from(bytes))
    ))
}

// TODO: maybe we should pass around as OsString which is null terminated dunno
pub fn internal_native2name_1(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // we already validated the name into unicode in the previous command
    bif::erlang::binary_to_list_1(vm, process, args)
    // Ok(args[0])
}

pub fn internal_name2native_1(
    _vm: &vm::Machine,
    _process: &RcProcess,
    args: &[Term],
) -> bif::Result {
    // we already validated the name into unicode in the previous command
    Ok(args[0])
}

#[cfg(unix)]
fn filetype_to_atom(file_type: fs::FileType) -> Term {
    use std::os::unix::fs::FileTypeExt;

    // TODO: does FIFO count as a device?
    if file_type.is_block_device() || file_type.is_char_device() {
        return atom!(DEVICE);
    }
    if file_type.is_dir() {
        return atom!(DIRECTORY);
    }
    if file_type.is_file() {
        return atom!(REGULAR);
    }
    if file_type.is_symlink() {
        return atom!(SYMLINK);
    }

    return atom!(OTHER);
}

#[cfg(not(unix))]
fn filetype_to_atom(file_type: fs::FileType) -> Term {
    // TODO: does FIFO count as a device?
    if file_type.is_block_device() || file_type.is_char_device() {
        return atom!(DEVICE);
    }
    if file_type.is_dir() {
        return atom!(DIRECTORY);
    }
    if file_type.is_file() {
        return atom!(REGULAR);
    }
    if file_type.is_symlink() {
        return atom!(SYMLINK);
    }

    return atom!(OTHER);
}

#[cfg(unix)]
const FILE_ACCESS_READ: u32 = 0o400;
#[cfg(unix)]
const FILE_ACCESS_WRITE: u32 = 0o200;
#[cfg(unix)]
const FILE_ACCESS_READ_WRITE: u32 = FILE_ACCESS_READ | FILE_ACCESS_WRITE;

#[cfg(unix)]
fn access_to_atom(mode: u32) -> Term {
    if (mode & FILE_ACCESS_READ != 0) && !(mode & FILE_ACCESS_WRITE != 0) {
        return atom!(READ);
    } else if (mode & FILE_ACCESS_WRITE != 0) && !(mode & FILE_ACCESS_READ != 0) {
        return atom!(WRITE);
    } else if mode & FILE_ACCESS_READ_WRITE != 0 {
        return atom!(READ_WRITE);
    }

    atom!(NONE)
}

/// The smallest value that can be converted freely between universal, local, and POSIX time, as
/// required by read_file_info/2. Corresponds to {{1902,1,1},{0,0,0}}
const FILE_MIN_FILETIME: i64 = -2_145_916_800;

#[cfg(unix)]
fn meta_to_tuple(heap: &Heap, meta: std::fs::Metadata) -> Term {
    use std::os::unix::fs::MetadataExt;

    let tup = tup!(
        heap,
        atom!(FILE_INFO),
        Term::uint64(heap, meta.size()),
        filetype_to_atom(meta.file_type()),
        access_to_atom(meta.mode()),
        Term::int64(heap, std::cmp::max(FILE_MIN_FILETIME, meta.atime())),
        Term::int64(heap, std::cmp::max(FILE_MIN_FILETIME, meta.mtime())),
        Term::int64(heap, std::cmp::max(FILE_MIN_FILETIME, meta.ctime())),
        Term::uint(heap, meta.mode()),
        Term::uint64(heap, meta.nlink()),
        Term::uint64(heap, meta.dev()),
        Term::uint64(heap, meta.rdev()),
        Term::uint64(heap, meta.ino()),
        Term::uint(heap, meta.uid()),
        Term::uint(heap, meta.gid()),
    );
    println!("file_info: {}", tup);
    tup
}

#[cfg(not(unix))]
fn meta_to_tuple(heap: &Heap, meta: std::fs::Metadata) -> Term {
    let zero = Term::int(0);

    // TODO:

    let mode = if meta.permissions().readonly() {
        READ
    } else {
        READ | WRITE
    };

    // if(!(attributes & FILE_ATTRIBUTE_READONLY)) {
    //     result->access = EFILE_ACCESS_READ | EFILE_ACCESS_WRITE;
    //     result->mode |= _S_IREAD | _S_IWRITE;
    // } else {
    //     result->access = EFILE_ACCESS_READ;
    //     result->mode |= _S_IREAD;
    // }

    /* Propagate user mode-bits to group/other fields */
    // result->mode |= (result->mode & 0700) >> 3;
    // result->mode |= (result->mode & 0700) >> 6;

    tup!(
        heap,
        atom!(FILE_INFO),
        Term::uint64(heap, meta.size()),
        filetype_to_atom(meta.file_type()),
        access_to_atom(meta.permissions()),
        Term::int64(
            heap,
            std::cmp::max(FILE_MIN_FILETIME, meta.accessed().unwrap())
        ),
        Term::int64(
            heap,
            std::cmp::max(FILE_MIN_FILETIME, meta.modified().unwrap())
        ),
        Term::int64(
            heap,
            std::cmp::max(FILE_MIN_FILETIME, meta.created().unwrap())
        ),
        Term::uint(heap, mode),
        Term::uint(heap, meta.links),
        Term::uint(heap, meta.major_device),
        zero,
        zero,
        zero,
        zero,
    )
}

pub fn read_info_nif_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;

    assert!(args.len() == 2);

    println!("file stuff");

    let follow_links = match args[1].to_int() {
        Some(i) => i > 0,
        None => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let cons = Cons::try_from(&args[0])?;
    // TODO: maybe do these casts in the native2name/name2native
    let path = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    println!("path stuff");

    let meta = if follow_links {
        std::fs::metadata(path)
    } else {
        std::fs::symlink_metadata(path)
    };

    println!("meta {:?}", meta);

    // TODO map/and then?
    let info = match meta {
        Ok(meta) => meta,
        Err(err) => return Ok(error_to_tuple(heap, err)),
    };

    Ok(meta_to_tuple(heap, info))
}

pub fn list_dir_nif_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // arg[0] = filename
    let heap = &process.context_mut().heap;

    // TODO: needs to work with binary and list based strings
    // TODO bitstrings or non zero offsets can fail ...
    let cons = Cons::try_from(&args[0])?;
    let path = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    println!("Trying to read dir {:?}", path);
    let res = match std::fs::read_dir(path) {
        Ok(entries) => Cons::from_iter(
            entries
                .map(|entry| {
                    Term::binary(
                        heap,
                        Binary::from(entry.unwrap().path().to_str().unwrap().as_bytes()),
                    )
                    // bitstring!(heap, entry.unwrap().path().to_str().unwrap()
                })
                .collect::<Vec<Term>>()
                .into_iter(),
            heap,
        ),
        Err(err) => return Ok(error_to_tuple(heap, err)),
    };

    Ok(tup2!(heap, atom!(OK), res))
}

#[cfg(test)]
mod tests {
    use super::*;

}
