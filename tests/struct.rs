use packtool::{Packed, View};

#[derive(Debug, PartialEq, Eq, Packed)]
#[packed(value = b"tuple")]
struct TagTuple();

#[derive(Debug, PartialEq, Eq, Packed)]
#[packed(value = b"struct")]
struct TagStruct {}

#[derive(Debug, PartialEq, Eq, Packed)]
struct Tuple1(u8);

#[derive(Debug, PartialEq, Eq, Packed)]
struct Tuple2(u32, u16);

#[derive(Debug, PartialEq, Eq, Packed)]
struct Tuple3(TagStruct, TagStruct, TagTuple);

#[derive(Debug, PartialEq, Eq, Packed)]
struct Struct2 {
    tag: TagStruct,
    value: u16,
}

macro_rules! internal_mk_test {
    ($Type:ty => ($cstr:expr, $SLICE:expr)) => {{
        let view = View::<$Type>::try_from_slice($SLICE).unwrap();
        let object: $Type = view.unpack();
        assert_eq!(object, $cstr);

        let mut slice = [0u8; <$Type as Packed>::SIZE];
        ($cstr).unchecked_write_to_slice(&mut slice);
        assert_eq!($SLICE, &slice);
    }};
    ($Type:ty => ( $SLICE:expr )) => {{
        let _err = View::<$Type>::try_from_slice($SLICE).unwrap_err();
    }};
}

#[test]
fn tuple1() {
    internal_mk_test!(Tuple1 => ( Tuple1(0), &[0u8]));
    internal_mk_test!(Tuple1 => ( Tuple1(42), &[42u8]));

    internal_mk_test!(Tuple1 => ( &[0u8; 2]));
}

#[test]
fn tuple2() {
    internal_mk_test!(Tuple2 => ( Tuple2(0xFF, 0), &[0xFFu8, 0, 0, 0, 0, 0]));
}

#[test]
fn tuple3() {
    internal_mk_test!(Tuple3 => (
        Tuple3(TagStruct {}, TagStruct {}, TagTuple ()),
        b"structstructtuple")
    );
}

#[test]
fn struct2() {
    internal_mk_test!(Struct2 => (
        Struct2  {
            tag: TagStruct {},
            value: 42,
        },
        b"struct\x2a\x00")
    );
}
