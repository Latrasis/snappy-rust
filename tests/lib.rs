extern crate snappy;

#[test]
/// Snappy: Create Compressor
fn should_create_compressor() {

	use std::io::{Write};
	use snappy::{Compressor, max_compressed_len};

	let mut sample: Vec<u8> = vec![];
	let n = Compressor::new(&mut sample).write(b"123456789").unwrap();
		
	// Assert Valid Written Size
	assert!(n == 9);
	// Result should be no more than max_compressed_len(src.len())
	assert!(sample.len() <= max_compressed_len(n));
}




