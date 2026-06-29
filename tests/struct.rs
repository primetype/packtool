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
    ($Type:ty => ($cstr:expr_2021, $SLICE:expr_2021)) => {{
        let view = View::<$Type>::try_from_slice($SLICE).unwrap();
        let object: $Type = view.unpack();
        assert_eq!(object, $cstr);

        let mut slice = [0u8; <$Type as Packed>::SIZE];
        ($cstr).unchecked_write_to_slice(&mut slice);
        assert_eq!($SLICE, &slice);
    }};
    ($Type:ty => ( $SLICE:expr_2021 )) => {{
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

// --- spike: type-parameter generics on STRUCTS ---------------------------

#[derive(Debug, PartialEq, Eq, Packed)]
struct Content {
    nonce: u32,
    flag: u8,
}

#[derive(Debug, PartialEq, Eq, Packed)]
struct Log<T> {
    parent_id: [u8; 64],
    author: [u8; 33],
    content: T,
    signature: [u8; 64],
}

#[test]
fn generic_log_round_trips() {
    use packtool::Packet;

    // SIZE is the sum of the field SIZEs, computed through `<T as Packed>::SIZE`.
    assert_eq!(
        <Log<Content> as Packed>::SIZE,
        64 + 33 + <Content as Packed>::SIZE + 64
    );

    let log = Log::<Content> {
        parent_id: [1u8; 64],
        author: [2u8; 33],
        content: Content {
            nonce: 0xDEAD_BEEF,
            flag: 0x2a,
        },
        signature: [3u8; 64],
    };

    // pack -> unpack round-trips to an EQUAL value.
    let packet = Packet::pack(&log);
    assert_eq!(packet.unpack(), log);

    // and via an explicitly checked `View` over the exact bytes.
    let view = View::<Log<Content>>::try_from_slice(packet.as_ref()).unwrap();
    assert_eq!(view.unpack(), log);

    // the generated accessor works for the generic field too.
    let content = Log::<Content>::content(packet.view());
    assert_eq!(
        content.unpack(),
        Content {
            nonce: 0xDEAD_BEEF,
            flag: 0x2a,
        }
    );

    // a wrong-length slice is rejected.
    let too_short = vec![0u8; <Log<Content> as Packed>::SIZE - 1];
    let _err = View::<Log<Content>>::try_from_slice(&too_short).unwrap_err();
}

// Two type parameters: `SIZE` must be the sum of each parameter's `SIZE`
// (plus the fixed `tag` byte), proving every `T: Packed` bound composes.
#[derive(Debug, PartialEq, Eq, Packed)]
struct Pair<A, B> {
    left: A,
    tag: u8,
    right: B,
}

#[test]
fn generic_pair_two_type_params() {
    use packtool::Packet;

    assert_eq!(
        <Pair<u32, Content> as Packed>::SIZE,
        <u32 as Packed>::SIZE + 1 + <Content as Packed>::SIZE
    );

    let pair = Pair::<u32, Content> {
        left: 0x0102_0304,
        tag: 0x2a,
        right: Content {
            nonce: 0xDEAD_BEEF,
            flag: 0x7f,
        },
    };

    let packet = Packet::pack(&pair);
    assert_eq!(packet.unpack(), pair);

    let view = View::<Pair<u32, Content>>::try_from_slice(packet.as_ref()).unwrap();
    assert_eq!(view.unpack(), pair);
}

// A generic struct that ALREADY carries a `where` clause: the derive must
// inject `T: Packed` ALONGSIDE the user's `T: Copy` rather than replacing it.
// If the clauses did not compose, this would fail to compile.
#[derive(Debug, PartialEq, Eq, Packed)]
struct Guarded<T>
where
    T: Copy,
{
    value: T,
    tag: u8,
}

#[test]
fn generic_struct_with_preexisting_where_clause() {
    use packtool::Packet;

    assert_eq!(<Guarded<u16> as Packed>::SIZE, <u16 as Packed>::SIZE + 1);

    let guarded = Guarded::<u16> {
        value: 0xBEEF,
        tag: 0x11,
    };

    let packet = Packet::pack(&guarded);
    assert_eq!(packet.unpack(), guarded);
}

// Sanity: a concrete (non-generic) struct still round-trips through the
// `Packet` path unchanged — the generics machinery emits the same `impl`
// (`split_for_impl` yields nothing) for concrete types.
#[test]
fn concrete_struct_unchanged() {
    use packtool::Packet;

    let content = Content {
        nonce: 0x0011_2233,
        flag: 0x44,
    };

    let packet = Packet::pack(&content);
    assert_eq!(packet.unpack(), content);
    assert_eq!(<Content as Packed>::SIZE, 5);
}
