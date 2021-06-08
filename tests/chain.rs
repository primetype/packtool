use packtool::{Packed, View};

#[derive(Packed, Debug, PartialEq, Eq)]
#[packed(value = b"protocol tag")]
struct Tag;

#[derive(Packed, Debug, PartialEq, Eq)]
struct Hash([u8; 32]);

#[derive(Packed, Debug, PartialEq, Eq)]
#[repr(u16)]
enum Version {
    Era1 = 0b0000_0000_0010_0001,
    Testing = 0b1111_0101_0000_1100,
    TestNet = 0b1111_0101_1011_1101,
}

#[derive(Packed, Debug, PartialEq, Eq)]
struct Value(u64);

#[derive(Packed, Debug, PartialEq, Eq)]
struct Packet {
    tag: Tag,
    version: Version,
    hash: Hash,
    value: Value,
}

const SIZE: usize
    = 12 // size of the tag
    + 2  // size of the version
    + 32 // size of the hash
    + 8  // size of the value
    ;
const SLICE: &[u8] =
        b"protocol tag\x21\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\0\0\0\0\0\0\0";
const PACKET: Packet = Packet {
    tag: Tag,
    version: Version::Era1,
    hash: Hash([0; 32]),
    value: Value(1),
};

#[test]
fn packed_size() {
    assert_eq!(<Packet as Packed>::SIZE, SIZE);
}

#[test]
fn encode_from_slice() {
    let mut slice = [0; SIZE];
    PACKET.unchecked_write_to_slice(&mut slice);

    assert_eq!(&slice, SLICE);
}

#[test]
fn decode_from_slice() {
    let view = View::<'_, Packet>::try_from_slice(SLICE).unwrap();
    let packet = view.unpack();

    assert_eq!(packet, PACKET);
}
