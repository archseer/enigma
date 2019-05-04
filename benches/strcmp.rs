#![feature(test)]

extern crate test;
use test::Bencher;
// use rand::Rng;
use libenigma::bitstring;
use libenigma::bitstring::{Binary, SubBinary};
use libenigma::servo_arc::Arc;

#[bench]
fn bitstring_cmpbits(b: &mut Bencher) {
    let binary = Arc::new(Binary::from(vec![0x0B, 0xCD, 0xE]));
    let subbinary = SubBinary::new(Arc::new(Binary::from(vec![0xAB, 0xCD, 0xEF])), 20, 4, false);

    b.iter(|| unsafe {
        assert!(
            bitstring::cmp_bits(
                binary.data.as_ptr(),
                4,
                subbinary.original.data.as_ptr(),
                subbinary.bit_offset as usize,
                subbinary.bitsize + subbinary.size
            ) == std::cmp::Ordering::Equal
        )
    })
}

#[bench]
fn bitstring_slice(b: &mut Bencher) {
    let binary = Arc::new(Binary::from(vec![0xB, 0xCD, 0xE]));
    let subbinary = SubBinary::new(Arc::new(Binary::from(vec![0xB, 0xCD, 0xE])), 20, 0, false);

    b.iter(|| unsafe {
        assert!(
            bitstring::cmp_bits(
                binary.data.as_ptr(),
                0,
                subbinary.original.data.as_ptr(),
                subbinary.bit_offset as usize,
                subbinary.bitsize + subbinary.size
            ) == std::cmp::Ordering::Equal
        )
    })
}
