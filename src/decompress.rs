
use std::io;
use std::io::{BufWriter, ErrorKind, Write, Seek, SeekFrom, Result, Error};
use std::result;
use self::byteorder::{ByteOrder, WriteBytesExt, LittleEndian};


// decompressed_len returns the length of the decoded block.
pub fn decompressed_len(src: &[u8]) -> Result<usize> {
	src.read_u64::<LittleEndian>()
}

// Decompress reads the decoded form of src into dst and returns the length
// read.
// Returns an error if dst was not large enough to hold the entire decoded
// block.
fn decompress(dst: &[u8], src: &[u8]) -> io::Result<usize> {
	// TODO: Handle Error
	let dLen = decompressed_len(src).unwrap();
	// TODO FIX!!
	// For now, always assume 8 bytes used for src header length
	let s: usize = 8;

	if dst.len() < dLen {
        return Err(Error::new(ErrorKind::InvalidInput, "snappy: destination buffer is too short"));
    }

    let (d, offset, length): (isize, isize, isize);

    while s < src.len() {
    	match src[s] & 0x03 {
    		TAG_LITERAL => {
    			let mut x = src[s] >> 2;

    			match x {
	    			0..59 => s += 1,
    				60 => {
    					s += 2;
    					if s > src.len() {
    						return Err(ErrorKind::InvalidData)
    					};
    					x = src[s-1] as usize;
    				},
    				61 => {
    					s += 3;
    					if s > src.len() {
    						return Err(ErrorKind::InvalidData)
    					};
    					x = (src[s-2] as usize) | ((src[s-1]<<8) as usize);
    				},
    				62 => {
    					s += 4;
    					if s > src.len() {
    						return Err(ErrorKind::InvalidData)
    					};
    					x = (src[s-3] as usize) | ((src[s-2]<<8) as usize) | ((src[s-1]<<16) as usize);
    				},
    				63 => {
    					s += 5;
    					if s > src.len() {
    						return Err(ErrorKind::InvalidData)
    					};
    					x = (src[s-4] as usize) | ((src[s-3]<<8) as usize) | ((src[s-2]<<16) as usize) | ((src[s-1]<<24) as usize);
    				},
    				_
    			}
    			length = (x+1) as isize;

    			if length <= 0 {
			        return Err(Error::new(ErrorKind::InvalidInput, "snappy: unsupported literal length"));
    			}
    			if length > dst.len() - d || length > src.len() - s {
    				return Err(ErrorKind::InvalidData)
    			}

    			// Copy src[s: s+length] to dst[d:0]
    			for (id, is) in dst.split_at_mut(d).1.iter_mut().zip(src.split_at(s).1.split_at(s+length).0.iter()) {
			        *id = *is;
			    }
			    d += length;
			    s += length;
			    continue;
    		}
    	}
    }
}

