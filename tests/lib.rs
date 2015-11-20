extern crate snappy_rust;


use std::io::{Write, Read, Cursor};
use snappy_rust::{Compressor,Decompressor, max_compressed_len};

#[test]
/// Snappy: Create Compressor and Decompressor
fn should_create() {

	// Write into Buffer
	let mut comp = Cursor::new(Vec::<u8>::new());
	let c = Compressor::new(&mut comp).write(b"123456789").unwrap();
	
	// Read into Buffer
	let mut decomp = Vec::<u8>::new();
	let d = Decompressor::new(&mut comp).read(&mut decomp).unwrap();

	// Check Result
	assert!(decomp == b"123456789");

}





