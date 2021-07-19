use packtool::{Packed, View};
use std::fmt;

// as in https://en.bitcoin.it/wiki/Protocol_documentation#Block_Headers
// and https://en.bitcoin.it/wiki/Block_hashing_algorithm

#[derive(Packed, Debug)]
pub struct Version(i32);

#[derive(Packed)]
pub struct Hash(#[packed(accessor = false)] [u8; 32]);

#[derive(Packed, Debug)]
pub struct None(u32);

#[derive(Packed, Debug)]
pub struct Header {
    #[packed(accessor = "get_version")]
    version: Version,
    prev_block: Hash,
    merkle_root: Hash,
    timestamp: u32,
    difficulty_target: u32,
    nonce: u32,
}

fn main() {
    assert_eq!(Header::SIZE, 80);

    const BLOCK: &[u8] = &[
        0x02, 0x00, 0x00, 0x00, 0x03, 0x5a, 0xb1, 0x54, 0x18, 0x35, 0x70, 0x28, 0x2c, 0xe9, 0xaf,
        0xc0, 0xb4, 0x94, 0xc9, 0xfc, 0x6a, 0x3c, 0xfe, 0xa0, 0x5a, 0xa8, 0xc1, 0xad, 0xd2, 0xec,
        0xc5, 0x64, 0x90, 0x00, 0x00, 0x00, 0x03, 0x8b, 0xa3, 0xd7, 0x8e, 0x45, 0x00, 0xa5, 0xa7,
        0x57, 0x0d, 0xbe, 0x61, 0x96, 0x03, 0x98, 0xad, 0xd4, 0x41, 0x0d, 0x27, 0x8b, 0x21, 0xcd,
        0x97, 0x08, 0xe6, 0xd9, 0x74, 0x3f, 0x37, 0x4d, 0x54, 0x4f, 0xc0, 0x55, 0x22, 0x7f, 0x10,
        0x01, 0xc2, 0x9c, 0x1e, 0xa3, 0xb0, 0x10, 0x10, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0xff,
        0xff, 0xff, 0xf3, 0x70, 0x3a, 0x08, 0x60, 0x10, 0x00, 0x42, 0x7f, 0x10, 0x01, 0xc0, 0x46,
        0xa5, 0x10, 0x10, 0x05, 0x22, 0xcf, 0xab, 0xe6, 0xd6, 0xd0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x06, 0x86, 0x92, 0x06, 0x67, 0x26, 0xf6, 0xd2, 0x07, 0x06, 0xf6,
        0xf6, 0xc7, 0x36, 0x57, 0x27, 0x66, 0x57, 0x26, 0xaa, 0xc1, 0xee, 0xee, 0xd8, 0x8f, 0xff,
        0xff, 0xff, 0xf0, 0x10, 0x0f, 0x20, 0x52, 0xa0, 0x10, 0x00, 0x00, 0x01, 0x97, 0x6a, 0x91,
        0x49, 0x12, 0xe2, 0xb2, 0x34, 0xf9, 0x41, 0xf3, 0x0b, 0x18, 0xaf, 0xbb, 0x4f, 0xa4, 0x61,
        0x71, 0x21, 0x4b, 0xf6, 0x6c, 0x88, 0x8a, 0xc0, 0x00, 0x00, 0x00, 0x0,
    ];
    let header: &[u8] = &BLOCK[..80];

    let view = View::<Header>::try_from_slice(header).expect("valid block");

    let version = Header::get_version(view).unpack();
    let timestamp = Header::timestamp(view).unpack();

    println!("{:?}", version);
    println!("{:?}", timestamp);
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut content = String::new();
        for byte in self.0.iter().copied() {
            content.push_str(&format!("{:02x}", byte));
        }

        f.debug_tuple("Hash").field(&content).finish()
    }
}