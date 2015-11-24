
// Definitions
mod definitions {

	//
	// Each encoded block begins with the varint-encoded length of the decoded data,
	// followed by a sequence of chunks. Chunks begin and end on byte boundaries.
	// The
	// first byte of each chunk is broken into its 2 least and 6 most significant
	// bits
	// called l and m: l ranges in [0, 4) and m ranges in [0, 64). l is the chunk
	// tag.
	// Zero means a literal tag. All other values mean a copy tag.
	//
	// For literal tags:
	// - If m < 60, the next 1 + m bytes are literal bytes.
	// - Otherwise, let n be the little-endian unsigned integer denoted by the next
	// m - 59 bytes. The next 1 + n bytes after that are literal bytes.
	//
	// For copy tags, length bytes are copied from offset bytes ago, in the style of
	// Lempel-Ziv compression algorithms. In particular:
	// - For l == 1, the offset ranges in [0, 1<<11) and the length in [4, 12).
	// The length is 4 + the low 3 bits of m. The high 3 bits of m form bits 8-10
	// of the offset. The next byte is bits 0-7 of the offset.
	// - For l == 2, the offset ranges in [0, 1<<16) and the length in [1, 65).
	// The length is 1 + m. The offset is the little-endian unsigned integer
	// denoted by the next 2 bytes.
	// - For l == 3, this tag is a legacy format that is no longer supported.
	//
    pub const TAG_LITERAL: u8 = 0x00;
    pub const TAG_COPY_1: u8 = 0x01;
    pub const TAG_COPY_2: u8 = 0x02;
    pub const TAG_COPY_4: u8 = 0x03;

    pub const CHECK_SUM_SIZE: u8 = 4;
    pub const CHUNK_HEADER_SIZE: u8 = 4;
    pub const MAGIC_BODY : [u8; 6] = *b"sNaPpY";
    pub const MAGIC_CHUNK : [u8; 10] = [0xff,0x06,0x00,0x00,0x73,0x4e,0x61,0x50,0x70,0x59];
    
    // https://github.com/google/snappy/blob/master/framing_format.txt says
	// that "the uncompressed data in a chunk must be no longer than 65536 bytes".
    pub const MAX_UNCOMPRESSED_CHUNK_LEN : u32 = 65536;

    pub const CHUNK_TYPE_COMPRESSED_DATA: u8 = 0x00;
    pub const CHUNK_TYPE_UNCOMPRESSED_DATA: u8 = 0x01;
    pub const CHUNK_TYPE_PADDING: u8 = 0xfe;
    pub const CHUNK_TYPE_STREAM_IDENTIFIER: u8 = 0xff;
}

// Snappy Compressor
mod compress;
pub use self::compress::{Compressor, compress, max_compressed_len};

// Snappy Decompressor
mod decompress;
pub use self::decompress::{Decompressor, decompress, decompressed_len};

