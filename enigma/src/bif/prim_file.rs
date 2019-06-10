use crate::atom;
use crate::bif;
use crate::bitstring::Binary;
use crate::exception::{Exception, Reason};
use crate::immix::Heap;
use crate::process::RcProcess;
use crate::value::{self, CastFrom, CastFromMut, Cons, Term, Variant};
use crate::vm;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{Read, Write};

impl CastFrom<Term> for std::fs::File {
    type Error = value::WrongBoxError;

    #[inline]
    fn cast_from(value: &Term) -> Result<&Self, value::WrongBoxError> {
        if let value::Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == value::BOXED_FILE {
                    return Ok(&(*(ptr as *const value::Boxed<Self>)).value);
                }
            }
        }
        Err(value::WrongBoxError)
    }
}

impl CastFromMut<Term> for std::fs::File {
    type Error = value::WrongBoxError;

    #[inline]
    fn cast_from_mut(value: &Term) -> Result<&mut Self, value::WrongBoxError> {
        if let value::Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == value::BOXED_FILE {
                    return Ok(&mut (*(ptr as *const value::Boxed<Self>
                        as *mut value::Boxed<Self>))
                        .value);
                }
            }
        }
        Err(value::WrongBoxError)
    }
}

fn error_to_tuple(heap: &Heap, error: std::io::Error) -> Term {
    use std::io::ErrorKind;
    let kind = match error.kind() {
        ErrorKind::NotFound => atom!(ENOENT),
        ErrorKind::Other => {
            let errno = error.raw_os_error().unwrap();
            match errno {
                20 => atom!(ENOTDIR),
                _ => unimplemented!("error_to_tuple for {:?}", error),
            }
        }
        _ => unimplemented!("error_to_tuple for {:?}", error),
    };
    tup2!(heap, atom!(ERROR), kind)
}
pub fn get_device_cwd_nif_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    unimplemented!()
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

pub fn set_cwd_nif_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    let cons = Cons::cast_from(&args[0])?;
    let path = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    match std::env::set_current_dir(path) {
        Ok(()) => Ok(atom!(OK)),
        Err(err) => Ok(error_to_tuple(heap, err)),
    }
}

/// Reads an entire file into \c result, stopping after \c size bytes or EOF. It will read until
/// EOF if size is 0.
pub fn read_file_nif_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // arg[0] = filename
    let heap = &process.context_mut().heap;

    // TODO bitstrings or non zero offsets can fail ...
    let cons = Cons::cast_from(&args[0])?;
    let path = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

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

pub fn ipread_s32bu_p32bu_nif_3(
    _vm: &vm::Machine,
    process: &RcProcess,
    args: &[Term],
) -> bif::Result {
    unimplemented!()
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

pub fn native_name_encoding_0(
    _vm: &vm::Machine,
    _process: &RcProcess,
    _args: &[Term],
) -> bif::Result {
    // TODO:
    Ok(atom!(UTF8))
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

    tup!(
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
    )
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

    let follow_links = match args[1].to_int() {
        Some(i) => i > 0,
        None => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let cons = Cons::cast_from(&args[0])?;
    // TODO: maybe do these casts in the native2name/name2native
    let path = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    let meta = if follow_links {
        std::fs::metadata(path)
    } else {
        std::fs::symlink_metadata(path)
    };

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
    let cons = Cons::cast_from(&args[0])?;
    let path = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    let res = match std::fs::read_dir(path) {
        Ok(entries) => Cons::from_iter(
            entries
                .map(|entry| {
                    Term::binary(
                        heap,
                        Binary::from(entry.unwrap().file_name().to_str().unwrap().as_bytes()),
                    )
                    // bitstring!(heap, entry.unwrap().file_name().to_str().unwrap()
                })
                .collect::<Vec<Term>>()
                .into_iter(),
            heap,
        ),
        Err(err) => return Ok(error_to_tuple(heap, err)),
    };

    Ok(tup2!(heap, atom!(OK), res))
}

pub fn open_nif_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    use std::fs::OpenOptions;
    let heap = &process.context_mut().heap;

    // println!("opening.. {} opts: {}\r", args[0], args[1]);

    let mut opts = OpenOptions::new();
    for value in Cons::cast_from(&args[1])?.iter() {
        match value.into_variant() {
            Variant::Atom(atom::READ) => opts.read(true),
            Variant::Atom(atom::WRITE) => opts.write(true).create(true),
            Variant::Atom(atom::EXCLUSIVE) => unimplemented!(),
            Variant::Atom(atom::APPEND) => opts.append(true),
            Variant::Atom(atom::SYNC) => unimplemented!(),
            Variant::Atom(atom::SKIP_TYPE_CHECK) => unimplemented!(),
            // Modes like 'raw', 'ram', 'delayed_writes' etc are handled further up the chain.
            _ => &mut opts,
        };
    }

    // FIXME:
    // if (modes & (EFILE_MODE_APPEND | EFILE_MODE_EXCLUSIVE)) {
    //     /* 'append' and 'exclusive' are documented as "open for writing." */
    //     modes |= EFILE_MODE_WRITE;
    // } else if !(modes & EFILE_MODE_READ_WRITE) {
    //     /* Defaulting to read if !(W|R) is undocumented, but specifically
    //      * tested against in file_SUITE. */
    //     modes |= EFILE_MODE_READ;
    // }
    let cons = Cons::cast_from(&args[0])?;
    let path = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    let file = match opts.open(path) {
        Ok(file) => file,
        Err(err) => {
            // println!("pid={} attempted open on {}, failed.", process.pid, args[0]);
            return Ok(error_to_tuple(heap, err));
        }
    };
    Ok(tup2!(heap, atom!(OK), Term::file(heap, file)))
}

pub fn close_nif_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let file = File::cast_from_mut(&args[0])?;
    unsafe { std::ptr::drop_in_place(file) }
    // FIXME: ^
    Ok(atom!(OK))
}

pub fn read_nif_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    let file = File::cast_from_mut(&args[0])?;

    let size = match args[1].into_variant() {
        Variant::Integer(i) => i as usize,
        // TODO: bigint
        _ => unimplemented!(),
        // _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let mut buffer = vec![0; size];

    // eprintln!("read_nif_2: fd: {:?} size: {}", file, size);

    match file.read(&mut buffer) {
        Ok(0) => Ok(atom!(EOF)),
        Ok(n) => {
            buffer.truncate(n);
            Ok(tup2!(
                heap,
                atom!(OK),
                Term::binary(heap, Binary::from(buffer))
            ))
        }
        Err(err) => Ok(error_to_tuple(heap, err)),
    }
}

pub fn write_nif_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    let file = File::cast_from_mut(&args[0])?;

    let bytes = crate::bif::erlang::list_to_iodata(args[1])?;
    match file.write_all(&bytes) {
        Ok(()) => Ok(atom!(OK)),
        Err(err) => Ok(error_to_tuple(heap, err)),
    }
}

pub fn pread_nif_3(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn pwrite_nif_3(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn seek_nif_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    use std::io::SeekFrom;
    let heap = &process.context_mut().heap;
    // file, :bof->set/:cur->cur/:eof->end, 0
    let file = File::cast_from_mut(&args[0])?;
    let pos = match args[2].into_variant() {
        Variant::Integer(i) if i >= 0 => i as usize,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let seek = match args[1].into_variant() {
        Variant::Atom(atom::BOF) => SeekFrom::Start(pos as u64),
        Variant::Atom(atom::CUR) => SeekFrom::Current(pos as i64),
        Variant::Atom(atom::EOF) => SeekFrom::End(pos as i64),
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    match file.seek(seek) {
        Ok(new_pos) => Ok(tup2!(heap, atom!(OK), Term::uint64(heap, new_pos))),
        Err(err) => Ok(error_to_tuple(heap, err)),
    }
}

pub fn sync_nif_2(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn truncate_nif_1(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn allocate_nif_3(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn advise_nif_4(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

// filesystem ops

pub fn make_hard_link_nif_2(
    _vm: &vm::Machine,
    _process: &RcProcess,
    _args: &[Term],
) -> bif::Result {
    unimplemented!()
}

pub fn make_soft_link_nif_2(
    _vm: &vm::Machine,
    _process: &RcProcess,
    _args: &[Term],
) -> bif::Result {
    unimplemented!()
}

pub fn rename_nif_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    let from = Cons::cast_from(&args[0])?;
    let from = value::cons::unicode_list_to_buf(from, 2048).unwrap();

    let to = Cons::cast_from(&args[1])?;
    let to = value::cons::unicode_list_to_buf(to, 2048).unwrap();

    println!("renaming {} to {}", from, to);

    match fs::rename(from, to) {
        Ok(()) => Ok(atom!(OK)),
        Err(err) => Ok(error_to_tuple(heap, err)),
    }
}

pub fn set_permissions_nif_2(
    _vm: &vm::Machine,
    _process: &RcProcess,
    _args: &[Term],
) -> bif::Result {
    unimplemented!()
}

pub fn set_owner_nif_3(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn set_time_nif_4(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn read_link_nif_1(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn make_dir_nif_1(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn del_file_nif_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // arg[0] = filename
    let heap = &process.context_mut().heap;

    // TODO: needs to work with binary and list based strings
    // TODO bitstrings or non zero offsets can fail ...
    let cons = Cons::cast_from(&args[0])?;
    let path = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    match fs::remove_file(path) {
        Ok(()) => Ok(atom!(OK)),
        Err(err) => Ok(error_to_tuple(heap, err)),
    }
}

pub fn del_dir_nif_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    unimplemented!()
}

// internal nifs

pub fn get_handle_nif_1(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn delayed_close_nif_1(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

pub fn altname_nif_1(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    unimplemented!()
}

// gzip

pub fn compress_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    use libflate::gzip;
    let heap = &process.context_mut().heap;
    let string = crate::bif::erlang::list_to_iodata(args[0]).unwrap(); // TODO: error handling
    let mut cursor = std::io::Cursor::new(string);

    let mut encoder = gzip::Encoder::new(Vec::new()).unwrap();
    std::io::copy(&mut cursor, &mut encoder).unwrap();
    let encoded_data = encoder.finish().into_result().unwrap();

    Ok(Term::binary(heap, Binary::from(encoded_data)))
}

pub fn uncompress_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    use libflate::gzip;
    let heap = &process.context_mut().heap;
    let string = crate::bif::erlang::list_to_iodata(args[0]).unwrap(); // TODO: error handling

    // check for gzip magic header
    if string.len() < 2 || string[..2] != [0x1f, 0x8b] {
        return Ok(Term::binary(heap, Binary::from(string)));
    }

    let mut data = Vec::new();

    gzip::Decoder::new(std::io::Cursor::new(string))
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();

    Ok(Term::binary(heap, Binary::from(data)))
}

// Override zlib:compress/1, zlib:uncompress/1

pub fn zlib_compress_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    use libflate::zlib;
    let heap = &process.context_mut().heap;
    let string = crate::bif::erlang::list_to_iodata(args[0]).unwrap(); // TODO: error handling
    let mut cursor = std::io::Cursor::new(string);

    let mut encoder = zlib::Encoder::new(Vec::new()).unwrap();
    std::io::copy(&mut cursor, &mut encoder).unwrap();
    let encoded_data = encoder.finish().into_result().unwrap();

    Ok(Term::binary(heap, Binary::from(encoded_data)))
}

#[cfg(test)]
mod tests {}
