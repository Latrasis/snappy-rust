use std::io;
use std::result;

use definitions::*;

// We limit how far copy back-references can go, the same as the C++ code.
const maxOffset: u16 = 1 << 15;

// emitLiteral writes a literal chunk and returns the number of bytes written.
fn emit_literal(dst: &mut [u8], lit: &mut [u8]) -> Result<usize, &'static str> {

	let mut i: usize;
	let n : u64 = (lit.len() -1) as u64;
	
	if n < 60 {
		dst[0] = (n as u8) << 2 | TAG_LITERAL;
		i = 1;
	} else if n < 1<<8 {
		dst[0] = 60<<2 | TAG_LITERAL;
		dst[1] = n as u8;
		i = 2;
	} else if n < 1<<16 {
		dst[0] = 61<<2 | TAG_LITERAL;
		dst[1] = n as u8;
		dst[2] = (n >> 8) as u8;
		i = 3;
	} else if n < 1<<32 {
		dst[0] = 63<<2 | TAG_LITERAL;
		dst[1] = n as u8;
		dst[2] = (n >> 8) as u8;
		dst[3] = (n >> 16) as u8;
		dst[4] = (n >> 24) as u8;
		i = 5;
	} else {
		return Err("snappy: source buffer is too long")
	}

	let mut s = 0;
	for (d, l) in dst.split_at_mut(i).1.iter_mut().zip(lit.iter()) {
        *d = *l;
        s += 1;
    }

    if(s == lit.len()){
		Ok(i + lit.len())
    } else {
    	Err("snappy: destination buffer is too short")
    }
}

// emitCopy writes a copy chunk and returns the number of bytes written.
fn emit_copy(dst: &mut [u8], offset: u8, length: usize) -> Result<usize, &'static str> {

	let mut i: usize = 0;



}

// MaxEncodedLen returns the maximum length of a snappy block, given its
// uncompressed length.
pub fn MaxEncodedLen(srcLen : u64) ->  u64 {
	32 + srcLen + srcLen/6
}

// Encode returns the encoded form of src. The returned slice may be a sub-
// slice of dst if dst was large enough to hold the entire encoded block.
// Otherwise, a newly allocated slice will be returned.
// It is valid to pass a nil dst.
pub fn Encode(dst: &'a mut [u8], src: &mut [u8]) -> Result<&'a mut [u8], &'static str> {
	
	unimplemented!()
	Ok(dst.split_at_mut(i).1)
}





