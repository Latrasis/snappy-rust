extern crate byteorder;
extern crate crc;

use std::io;
use std::io::{BufWriter, ErrorKind, Write, Seek, SeekFrom, Result, Error};
use self::byteorder::{ByteOrder, WriteBytesExt, LittleEndian};
use self::crc::{crc32, Hasher32};

use definitions::*;


// We limit how far copy back-references can go, the same as the C++ code.
const MAX_OFFSET: usize = 1 << 15;

// The Max Encoded Length of the Max Chunk of 65536 bytes
const MAX_BUFFER_SIZE: usize = 76_490;

pub struct Compressor<W: Write> {
    inner: BufWriter<W>,
    pos: u64,
    buf_body: [u8; MAX_BUFFER_SIZE],
    buf_header: [u8; (CHECK_SUM_SIZE + CHUNK_HEADER_SIZE) as usize],
    wrote_header: bool,
}

impl <W: Write> Compressor<W> {

    pub fn new(inner: W) -> Compressor<W> {
        Compressor {
            inner: BufWriter::new(inner),
            pos: 0,
            buf_body: [0; MAX_BUFFER_SIZE],
            buf_header: [0; (CHECK_SUM_SIZE + CHUNK_HEADER_SIZE) as usize],
            wrote_header: false,
        }
    }
}

impl <W: Write> Write for Compressor<W> {
    // Implement Write
    // Source Buffer -> Destination (Inner) Buffer
    fn write(&mut self, src: &[u8]) -> Result<usize> {

        let mut written: usize = 0;

        if !self.wrote_header {
            // Write Stream Literal
            try!(self.inner.write(&MAGIC_CHUNK));
            self.wrote_header = true;
        }

        // Split source into chunks of 65536 bytes each.
        for src_chunk in src.chunks(MAX_UNCOMPRESSED_CHUNK_LEN as usize) {

            // TODO 
            // Handle Written Slice Length (Ignore Previous Garbage)
            let chunk_body: &[u8];
            let chunk_type: u8;

            // Create Checksum
            let checksum: u32 = crc32::checksum_ieee(src_chunk);

            // Compress the buffer, discarding the result if the improvement
            // isn't at least 12.5%.
            let n = try!(compress(&mut self.buf_body, src_chunk));
            if n >= src_chunk.len() * (7 / 8) {
                chunk_type = CHUNK_TYPE_UNCOMPRESSED_DATA;
                chunk_body = src_chunk;
            } else {
                chunk_type = CHUNK_TYPE_COMPRESSED_DATA;
                chunk_body = self.buf_body.split_at(n).0;
            }

            let chunk_len = chunk_body.len() + 4;

            // Write Chunk Type
            self.buf_header[0] = chunk_type;
            // Write Chunk Length
            self.buf_header[1] = (chunk_len >> 0) as u8;
            self.buf_header[2] = (chunk_len >> 8) as u8;
            self.buf_header[3] = (chunk_len >> 16) as u8;
            // Write Chunk Checksum
            self.buf_header[4] = (checksum >> 0) as u8;
            self.buf_header[5] = (checksum >> 8) as u8;
            self.buf_header[6] = (checksum >> 16) as u8;
            self.buf_header[7] = (checksum >> 24) as u8;

            // Write Chunk Header and Handle Error
            try!(self.inner.write_all(&self.buf_header));
            // Write Chunk Body and Handle Error
            try!(self.inner.write_all(chunk_body));

            // If all goes well, count written length as uncompressed length
            written += src_chunk.len();
        }

        Ok(written)
    }

    // Flushes Inner buffer and resets Compressor
    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

// If Compressor is Given a Cursor or Seekable Writer
// This Gives the BufWriter the seek method
impl <W: Write + Seek> Seek for Compressor<W> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.inner.seek(pos).and_then(|res: u64| {
            self.pos = res;
            Ok(res)
        })
    }
}

// Compress writes the encoded form of src into dst and return the length
// written.
// Returns an error if dst was not large enough to hold the entire encoded
// block.

// (Future) Include a Legacy Compress??
pub fn compress(dst: &mut [u8], src: &[u8]) -> io::Result<usize> {

    if dst.len() < max_compressed_len(src.len()) {
        return Err(Error::new(ErrorKind::InvalidInput, "snappy: destination buffer is too short"));
    }

    // Start Block with varint-encoded length of decompressed bytes
    LittleEndian::write_u64(dst, src.len() as u64);

    let mut d: usize = 4;

    // Return early if src is short
    if src.len() <= 4 {
        if src.len() != 0 {
            // TODO Handle Error
            d += emit_literal(dst.split_at_mut(d).1, src).unwrap();
        }
        return Ok(d);
    }

    // Initialize the hash table. Its size ranges from 1<<8 to 1<<14 inclusive.
    const MAX_TABLE_SIZE: usize = 1 << 14;
    let mut shift: u32 = 24;
    let mut table_size: usize = 1 << 8;

    while table_size < MAX_TABLE_SIZE && table_size < src.len() {
        shift -= 1;
        table_size *= 2;
    }

    // Create table
    let mut table: Vec<i32> = vec![0; MAX_TABLE_SIZE];

    // Iterate over the source bytes
    let mut s: usize = 0;
    let mut t: i32;
    let mut lit: usize = 0;

    // (Future) Iterate in chunks of 4?
    while s + 3 < src.len() {

        // Grab 4 bytes
        let b: (u8, u8, u8, u8) = (src[s], src[s + 1], src[s + 2], src[s + 3]);
        
        // Create u32 for Hashing
        let h: u32 = (b.0 as u32) | ((b.1 as u32) << 8) | ((b.2 as u32) << 16) | ((b.3 as u32) << 24);

        // Update the hash table
        let ref mut p: i32 = table[(h.wrapping_mul(0x1e35a7bd) >> shift) as usize];

        // We need to to store values in [-1, inf) in table. To save
        // some initialization time, (re)use the table's zero value
        // and shift the values against this zero: add 1 on writes,
        // subtract 1 on reads.

        // If Hash Exists t = 0; if not t = -1;
        t = *p - 1;

        // Set Position of new Hash -> i32
        *p = (s + 1) as i32;

        // If t is invalid or src[s:s+4] differs from src[t:t+4], accumulate a literal
        // byte.
        if t < 0 || 
            s.wrapping_sub(t as usize) >= MAX_OFFSET || 
            b.0 != src[t as usize] || 
            b.1 != src[(t + 1) as usize] || 
            b.2 != src[(t + 2) as usize] || 
            b.3 != src[(t + 3) as usize] {
            s += 1;
            continue;
        }

        // Otherwise, we have a match. First, emit any pending literal bytes.
        // Panics on d > dst.len();

        if lit != s {
            d += try!(emit_literal(dst.split_at_mut(d).1,

                // Handle Split_at mid < src check 
                {
                    if src.len() > s {
                        src.split_at(s).0.split_at(lit).1
                    } else {
                        src.split_at(lit).1
                    }
                }));
        }

        // Extend the match to be as long as possible
        let s0 = s;
        s += 4;
        t += 4;
        while s < src.len() && src[s] == src[t as usize] {
            s += 1;
            t += 1;
        }

        // Emit the copied bytes.
        d += emit_copy(dst.split_at_mut(d).1, s.wrapping_sub(t as usize), s - s0);
        lit = s;
    }

    // Emit any final pending literal bytes and return.
    if lit != src.len() {
        d += emit_literal(dst.split_at_mut(d).1, src.split_at(lit).1).unwrap();
    }

    Ok(d)
}

// emitLiteral writes a literal chunk and returns the number of bytes written.
fn emit_literal(dst: &mut [u8], lit: &[u8]) -> io::Result<usize> {

    let i: usize;
    let n: u64 = (lit.len() - 1) as u64;

    if n < 60 {
        dst[0] = (n as u8) << 2 | TAG_LITERAL;
        i = 1;
    } else if n < 1 << 8 {
        dst[0] = 60 << 2 | TAG_LITERAL;
        dst[1] = n as u8;
        i = 2;
    } else if n < 1 << 16 {
        dst[0] = 61 << 2 | TAG_LITERAL;
        dst[1] = n as u8;
        dst[2] = (n >> 8) as u8;
        i = 3;
    } else if n < 1 << 24 {
        dst[0] = 62 << 2 | TAG_LITERAL;
        dst[1] = n as u8;
        dst[2] = (n >> 8) as u8;
        dst[3] = (n >> 16) as u8;
        i = 4;
    } else if n < 1 << 32 {
        dst[0] = 63 << 2 | TAG_LITERAL;
        dst[1] = n as u8;
        dst[2] = (n >> 8) as u8;
        dst[3] = (n >> 16) as u8;
        dst[4] = (n >> 24) as u8;
        i = 5;
    } else {
        return Err(Error::new(ErrorKind::InvalidInput, "snappy: source buffer is too long"));
    }

    let mut s = 0;
    for (d, l) in dst.split_at_mut(i).1.iter_mut().zip(lit.iter()) {
        *d = *l;
        s += 1;
    }

    if s == lit.len() {
        Ok(i + lit.len())
    } else {
        return Err(Error::new(ErrorKind::InvalidInput, "snappy: destination buffer is too short"));
    }
}

// emitCopy writes a copy chunk and returns the number of bytes written.
fn emit_copy(dst: &mut [u8], offset: usize, mut length: usize) -> usize {

    let mut i: usize = 0;

    while length > 0 {
        // TODO: Handle Overflow
        let mut x = length - 4;
        if 0 <= x && x < 1 << 3 && offset < 1 << 11 {
            dst[i] = ((offset >> 8) as u8) & 0x07 << 5 | (x as u8) << 2 | TAG_COPY_1;
            dst[i + 1] = offset as u8;
            i += 2;
            break;
        }
        x = length;
        if x > 1 << 6 {
            x = 1 << 6;
        }
        dst[i] = ((x as u8) - 1) << 2 | TAG_COPY_2;
        dst[i + 1] = offset as u8;
        dst[i + 2] = (offset >> 8) as u8;
        i += 3;
        length -= x;
    }
    // (Future) Return a `Result<usize>` Instead??
    i
}

// max_compressed_len returns the maximum length of a snappy block, given its
// uncompressed length.
pub fn max_compressed_len(src_len: usize) -> usize {
    32 + src_len + src_len / 6
}
