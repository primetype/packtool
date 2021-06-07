use packtool::{Packed, View};

#[derive(Packed)]
#[repr(u8)]
enum OneU8 {
    One = 1,
}

#[derive(Packed)]
#[repr(u32)]
enum TwoU8 {
    One = 42,
    Two = 0x00FF00FF,
}

#[test]
fn one_u8() {
    let tag = View::<OneU8>::try_from_slice(&[1u8]).unwrap();
    assert!(matches!(tag.into(), OneU8::One));
}

#[test]
fn two_u8() {
    let tag = View::<TwoU8>::try_from_slice(&[42, 0, 0, 0]).unwrap();
    assert!(matches!(tag.into(), TwoU8::One));

    let tag = View::<TwoU8>::try_from_slice(&[0xFF, 0, 0xFF, 0]).unwrap();
    assert!(matches!(tag.into(), TwoU8::Two));
}
