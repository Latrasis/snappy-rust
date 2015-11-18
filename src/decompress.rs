extern crate byteorder;
extern crate crc;

use std::result::*;
use std::io;
use std::io::{BufReader, ErrorKind, Read, Seek, SeekFrom, Error};
use self::byteorder::{ByteOrder, ReadBytesExt, LittleEndian};
use self::crc::{crc32, Hasher32};

use definitions::*;
use compress::{max_compressed_len};

// The Max Encoded Length of the Max Chunk of 65536 bytes
const MAX_BUFFER_SIZE: usize = 76_490;

pub struct Decompressor<R: Read> {
	inner: BufReader<R>,
	decoded: [u8; MAX_UNCOMPRESSED_CHUNK_LEN as usize],
	buf: [u8; MAX_BUFFER_SIZE + (CHECK_SUM_SIZE as usize)],
	// decoded[i:j] contains decoded bytes that have not yet been passed on.
	i: usize,
	j: usize,
	read_header: bool,
}

impl <R: Read> Decompressor<R> {

	pub fn new(inner: R) -> Decompressor<R> {
		Decompressor {
			inner: BufReader::new(inner),
			decoded: [0; MAX_UNCOMPRESSED_CHUNK_LEN as usize],
			buf: [0; MAX_BUFFER_SIZE + (CHECK_SUM_SIZE as usize)],
			i: 0,
			j: 0,
			read_header: false,
		}
	}
}

impl <R: Read> Read for Decompressor<R> {
	// Implement Read
	// Source (Inner) Buffer into Destination Buffer, returning how many bytes were read.
	fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
		loop {

			if self.i < self.j {
				let mut s = 0; 
				for (d, l) in dst.iter_mut().zip(self.decoded.split_at(self.i).1.split_at(self.j).0.iter()) {
			        *d = *l;
			        s += 1;
			    }
			    self.i += s;
			    return Ok(s)
			}	

			// Grab Chunk Header
			if self.inner.read(self.buf.split_at_mut(4).0).unwrap() != 4 {
				return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"))
			}

			// Read Chunk Type
			let chunk_type = self.buf[0];
			if !self.read_header {
				if chunk_type != CHUNK_TYPE_STREAM_IDENTIFIER {
					return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"))
				}
				self.read_header = true;
			}

			// Read Chunk Length
			let chunk_len = self.buf[1] as usize | ((self.buf[2] as usize) << 8) | ((self.buf[3] as usize) << 16);
			if chunk_len > self.buf.len() {
				return Err(Error::new(ErrorKind::InvalidInput, "snappy: unsupported input"))
			}

			// The chunk types are specified at
			// https://github.com/google/snappy/blob/master/framing_format.txt
			match chunk_type {

				// Section 4.2. Compressed data (chunk type 0x00).
				CHUNK_TYPE_COMPRESSED_DATA => {
					if chunk_len < (CHECK_SUM_SIZE as usize) {
						return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"))
					}
					// Setup Buffer Slice with Chunk Length
					let buf: &mut [u8] = self.buf.split_at_mut(chunk_len as usize).0;
					
					// Read into Buffer
					if self.inner.read(buf).unwrap() != chunk_len {
						return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"))
					}

					// Read Checksum
					let check_sum = buf[0] as u32 | ((buf[1] as u32) << 8) | ((buf[2] as u32) << 16) | ((buf[3] as u32) << 24);
					
					// TODO: 
					// Figure Out Borrowing and Modifying Buffer Slice instead of Copying.
					// buf = buf.split_at_mut(CHECK_SUM_SIZE as usize).1;
					let mut buf1 = buf.split_at_mut(CHECK_SUM_SIZE as usize).1;

					// Check Decompressed Length
					let n = decompressed_len(buf1.as_ref()) as usize;
					if n > self.decoded.len() {
						return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"))
					}

					// Decompress
					try!(decompress(self.decoded.as_mut(), buf1));

					// Check Checksum
					if crc32::checksum_ieee(self.decoded.split_at(n).0) != check_sum {
						return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"))
					}

					self.i = 0;
					self.j = n;
					continue;
				},
				// Section 4.3. Uncompressed data (chunk type 0x01).
				CHUNK_TYPE_UNCOMPRESSED_DATA => {
					if chunk_len < (CHECK_SUM_SIZE as usize) {
						return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"))
					}

					// Read Checksum using self.buf slice
					let check_buf: &mut [u8] = self.buf.split_at_mut(CHECK_SUM_SIZE as usize).0;
					
					// Read into Checksum Buffer
					if self.inner.read(check_buf).unwrap() != chunk_len {
						return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"))
					}
					// Read Checksum
					let check_sum = check_buf[0] as u32 | ((check_buf[1] as u32) << 8) | ((check_buf[2] as u32) << 16) | ((check_buf[3] as u32) << 24);
					
					// Read Directly into self.decoded instead of self.buf
					let n = chunk_len - (CHECK_SUM_SIZE as usize);
					// Read into self.decoded Buffer
					if self.inner.read(self.decoded.split_at_mut(n).0).unwrap() != chunk_len {
						return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"))
					}

					// Check Checksum
					if crc32::checksum_ieee(self.decoded.split_at(n).0) != check_sum {
						return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"))
					}

					self.i = 0;
					self.j = n;
					continue;
				},
				// Section 4.1. Stream identifier (chunk type 0xff).
				CHUNK_TYPE_STREAM_IDENTIFIER => {},

				// Section 4.5. Reserved unskippable chunks (chunk types 0x02-0x7f).
				_ => {
					return Err(Error::new(ErrorKind::InvalidInput, "snappy: unsupported input"))
				}

			}




			unimplemented!()

		}
	}
}


// decompressed_len returns the length of the decoded block.
pub fn decompressed_len(src: &[u8]) -> u64 {
	LittleEndian::read_u64(src)
}

// Decompress reads the decoded form of src into dst and returns the length
// read.
// Returns an error if dst was not large enough to hold the entire decoded
// block.
fn decompress(dst: &mut [u8], src: &mut [u8]) -> io::Result<usize> {
	// TODO: Handle Error
	let dLen = decompressed_len(src) as usize;
	// TODO FIX!!
	// For now, always assume 8 bytes used for src header length
	let mut s: usize = 8;

	if dst.len() < dLen {
        return Err(Error::new(ErrorKind::InvalidInput, "snappy: destination buffer is too short"));
    }

    let (mut d, mut offset, mut length): (usize, usize, usize) = (0,0,0);

    while s < src.len() {
    	match src[s] & 0x03 {

			// Parse a Literal Chunk
    		TAG_LITERAL => {
    			
    			let mut x = (src[s] >> 2) as usize;
    			match x {
	    			0...59 => s += 1,
    				60 => {
    					s += 2;
    					if s > src.len() {
					        return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"));
    					};
    					x = src[s-1] as usize;
    				},
    				61 => {
    					s += 3;
    					if s > src.len() {
					        return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"));
    					};
    					x = (src[s-2] as usize) | ((src[s-1] as usize) << 8);
    				},
    				62 => {
    					s += 4;
    					if s > src.len() {
					        return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"));
    					};
    					x = (src[s-3] as usize) | ((src[s-2] as usize) << 8) | ((src[s-1] as usize) << 16);
    				},
    				63 => {
    					s += 5;
    					if s > src.len() {
					        return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"));
    					};
    					x = (src[s-4] as usize) | ((src[s-3] as usize) << 8) | ((src[s-2] as usize) << 16) | ((src[s-1] as usize) << 24);
    				},
    				_ => {}
    			}
    			length = x.wrapping_add(1);

    			if length as isize <= 0 {
			        return Err(Error::new(ErrorKind::InvalidInput, "snappy: unsupported literal length"));
    			}
    			if length > dst.len() - d || length > src.len() - s {
			        return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"));
    			}

    			// Copy src[s: s+length] to dst[d:0]
    			for (id, is) in dst.split_at_mut(d).1.iter_mut().zip(src.split_at(s).1.split_at(s+length).0.iter()) {
			        *id = *is;
			    }
			    d += length;
			    s += length;
			    continue;
    		},

    		// Parse a Copy1 Chunk
    		TAG_COPY_1 => {
    			s+= 2;
    			if s > src.len() {
			        return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"));
				};
				length = 4 + (src[s-2] as usize)>>2&0x7;
				offset = ((src[s-2] as usize)&0xe0<<3) | (src[s-1] as usize);
    		},

    		// Parse a Copy2 Chunk
    		TAG_COPY_2 => {
    			s+= 3;
    			if s > src.len() {
			        return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"));
				};
				length = 1 + (src[s-3] as usize)>>2;
				offset = (src[s-2] as usize) | ((src[s-1] as usize)<<8);
    		},

    		// Parse a Copy4
    		TAG_COPY_4 => {
		        return Err(Error::new(ErrorKind::InvalidInput, "snappy: unsupported COPY_4 tag"));
    		},

    		_ => unreachable!()
    	};

    	let end = d + length;
    	if offset > d || end > dst.len() {
	        return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"));
	    }
    	while d < end {
    		d += 1;
    		dst[d] = dst[d-offset];
    	}
    }
    if d != dLen {
        return Err(Error::new(ErrorKind::InvalidInput, "snappy: corrupt input"));
    }
    Ok(d)
}

