extern crate snappy_rust;

#[macro_use]
extern crate log;

use std::io::{Write, Read, Cursor};
use snappy_rust::{Compressor,Decompressor, compress, decompress, max_compressed_len};

#[test]
/// Snappy: Should Do Basic Valid Compress
fn should_compress(){

	let mut comp = Cursor::new(Vec::<u8>::with_capacity(100));

	comp.write(b"1234567890");

	info!("{:?}", comp);
}

#[test]
/// Snappy: Should Do Basic Valid Decompress
fn should_decompress(){

}

#[test]
/// Snappy: Create Compressor and Decompressor
fn should_create() {

	// Write into Buffer
	let mut comp = Cursor::new(Vec::<u8>::with_capacity(100));
	let c = Compressor::new(&mut comp).write_all(b"123456789abcdefg").unwrap();

	// Reset Position
	comp.set_position(0);

	// Read into Buffer
	let mut decomp : Vec<u8> = vec![0; 16];
	let d = Decompressor::new(&mut comp).read(&mut decomp).unwrap();

	// Check Result
	assert!(d <= decomp.len());
	assert!(decomp == b"123456789abcdefg");

}





