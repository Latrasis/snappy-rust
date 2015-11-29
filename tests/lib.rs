extern crate snappy_rust;

#[macro_use]
extern crate log;

use std::io::{Write, Read, Cursor, Result};
use snappy_rust::{Compressor,Decompressor, compress, decompress, max_compressed_len};


fn roundtrip(data: &[u8]) {

	// Write into Buffer
	let mut comp = Cursor::new(Vec::<u8>::with_capacity(100));
	let c = Compressor::new(&mut comp).write_all(data).unwrap();

	// Reset Position
	comp.set_position(0);

	// Read into Buffer
	let mut decomp : Vec<u8> = vec![0; data.len()];
	let d = Decompressor::new(&mut comp).read(&mut decomp).unwrap();

	// Check Result
	assert!(decomp == data);
}


// #[test]
// /// Snappy: Roundtrip Uncompressible Data
// fn should_roundtrip_uncompressible() {
// 	roundtrip(b"123456789abcdefg");
// }

#[test]
/// Snappy: Roundtrip Compressible Data
fn should_roundtrip_compressible() {
	
	// TODO
	// Byteorder Crashes with this:
	roundtrip(b"1111111100000000");
}





