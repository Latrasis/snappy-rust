extern crate snappy;

#[test]
/// Snappy: Create Compressor
fn should_create_compressor() {

	use std::io::{Write};
	use snappy::Compressor;

	let sample = vec![b'H'];
	let mut snapper = Compressor::new(sample);
	let n = snapper.write(b"123456789").unwrap();

	assert!(n == 9);
	
}

