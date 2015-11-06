use std::io;
use std::io::{ BufWriter, ErrorKind, Write, Seek, SeekFrom, Result};
use std::result;

use definitions::*;

// We limit how far copy back-references can go, the same as the C++ code.
const maxOffset: u16 = 1 << 15;

pub struct Compressor<W: Write> {
	inner: BufWriter<W>,
	pos: u64,
	wroteHeader: bool,
}

impl <W: Write> Compressor<W> {

	pub fn new(inner: W) -> Compressor<W> {
		Compressor {
			inner: BufWriter::new(inner),
			pos: 0,
			wroteHeader: false,
		}
	}
}

impl <W: Write> Write for Compressor<W> {
    // Implement Write
    // Source Buffer -> Destination (Inner) Buffer
    pub fn write(&mut self, src: &[u8]) -> Result<usize> {
    	unimplemented!()
    }
    // Implement Flush
    pub fn flush(&mut self) -> Result<()> {
    	unimplemented!()
    }
}

// If Compressor is Given a Cursor or Seekable Writer
// This Gives the BufWriter the seek method
impl <W: Write + Seek> Seek for Compressor<W> {
	pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
		self.inner.seek(pos).and_then(|res: u64| {
			self.pos = res;
			Ok(res)
		})
	}
}

pub fn compress(src: &[u8]) -> Vec<u8> {
	let dst : Vec<u8> = Vec::with_capacity(Max_Encoded_Len(src.len()));

	// Start Block with varint-encoded length of decompressed bytes
	for i in 0..7 {
		// dst.push((((src.len() as u64) >> 8*i) << 8*i) as u8);
	}

	// Return early if src is short
	if src.len() <= 4 {
		if src.len() != 0 {
			// TODO
			unimplemented!();
		}
	}

	unimplemented!()

}

// emitLiteral writes a literal chunk and returns the number of bytes written.
fn emit_literal(dst: &mut [u8], lit: &mut [u8]) -> Result<usize> {

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
	} else if n < 1<<24 {
		dst[0] = 62<<2 | TAG_LITERAL;
		dst[1] = n as u8;
		dst[2] = (n >> 8) as u8;
		dst[3] = (n >> 16) as u8;
		i = 4;
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
fn emit_copy(dst: &mut [u8], offset: usize, mut length: usize) -> usize {

	let mut i: usize = 0;

	while length > 0 {
		let mut x = length - 4;
		if 0 <=x && x < 1<<3 && offset < 1<<11 {
			dst[i] = ((offset>>8) as u8)&0x07<<5 | (x as u8)<<2 | TAG_COPY_1;
			dst[i+1] = offset as u8;
			i += 2;
			break;
		}
		x = length;
		if x > 1<<6 {
			x = 1 << 6;
		}
		dst[i] = ((x as u8) -1)<<2 | TAG_COPY_2;
		dst[i+1] = offset as u8;
		dst[i+2] = (offset >> 8) as u8;
		i += 3;
		length -= x;
	}
	i
}

// Max_Encoded_Len returns the maximum length of a snappy block, given its
// uncompressed length.
pub fn Max_Encoded_Len(srcLen : usize) ->  usize {
	32 + srcLen + srcLen/6
}

// Encode returns the encoded form of src. The returned slice may be a sub-
// slice of dst if dst was large enough to hold the entire encoded block.
// Otherwise, a newly allocated slice will be returned.
// It is valid to pass a nil dst.
// pub fn Encode(dst: &'a mut [u8], src: &mut [u8]) -> Result<&'a mut [u8], &'static str> {
	
// 	// Ok(dst.split_at_mut(i).1)
// }




