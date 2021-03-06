extern crate snappy_rust;

#[macro_use]
extern crate log;

use std::io::{Write, Read, Cursor, Result};
use snappy_rust::{Compressor,Decompressor, compress, decompress, max_compressed_len};


fn roundtrip(data: &[u8]) -> bool {

	// Write into Buffer
	let mut comp = Cursor::new(Vec::<u8>::with_capacity(100));
	let c = Compressor::new(&mut comp)
		.write_all(data)
		.unwrap_or_else(|err| {
			panic!("Error at Compress: {:?}", err);
		});

	// Reset Position
	comp.set_position(0);

	// Read into Buffer
	let mut decomp : Vec<u8> = vec![0; data.len()];
	let d = Decompressor::new(&mut comp)
		.read(&mut decomp)
		.unwrap_or_else(|err| {
			panic!("Error at Decompress: {:?}", err);
		});

	// Check Result
	decomp == data
}

#[test]
/// Snappy: Roundtrip Uncompressible Data
fn should_do_uncompressible() {
	assert!(roundtrip(b"123456789abcdefg"));
	assert!(roundtrip(b"The quick red fox jumped over the lazy dog"));
}

#[test]
/// Snappy: Roundtrip Compressible Data
fn should_do_compressible() {
	
	// TODO
	// Byteorder Crashes with this:
	assert!(roundtrip(b"1111111100000000"));
}

#[test]
/// Snappy: Test Empty
fn should_do_empty() {
	let a = [];
	roundtrip(&a);
}


#[test]
/// Snappy: Test Files
fn should_do_files(){

	// Testfiles are copied directly from:
	// https://raw.githubusercontent.com/google/snappy/master/snappy_unittest.cc
	let test_files: Vec<(&str, &[u8])> = vec![
		("alice29.txt" ,include_bytes!("data/alice29.txt")),
		("asyoulik.txt", include_bytes!("data/asyoulik.txt")),
		("lcet10.txt", include_bytes!("data/lcet10.txt")),
		("plrabn12.txt", include_bytes!("data/plrabn12.txt"))
	];

	for (label, data) in test_files {
		assert!(roundtrip(data), "Mismatch at File: {:?}", label);
	}
}







