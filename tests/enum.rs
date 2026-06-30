use packtool::{Packed, View};

#[derive(Packed, Debug, PartialEq, Eq)]
#[repr(u8)]
enum OneU8 {
    One = 1,
}

#[derive(Packed, Debug, PartialEq, Eq)]
#[repr(u32)]
enum TwoU8 {
    One = 42,
    Two = 0x00FF00FF,
}

/*
#[derive(Packed, Debug, PartialEq, Eq)]
enum ThisOrThat {
    #[packed(value = 0)]
    This,
    That(u32),
}
 */

macro_rules! internal_mk_test {
    ($Type:ty => ($cstr:expr_2021, $SLICE:expr_2021)) => {{
        let view = View::<$Type>::try_from_slice($SLICE).unwrap();
        let variant: $Type = view.unpack();
        assert_eq!(variant, $cstr);

        let mut slice = [0; <$Type as Packed>::SIZE];
        ($cstr).unchecked_write_to_slice(&mut slice);
        assert_eq!($SLICE, &slice);
    }};
    ($Type:ty => ( $error:literal $SLICE:expr_2021 )) => {{
        let err = View::<$Type>::try_from_slice($SLICE).unwrap_err();

        assert_eq!(err.to_string(), $error);
    }};
}

#[test]
fn one_u8() {
    internal_mk_test!(OneU8 => ( OneU8::One, &[1u8]));

    internal_mk_test!(OneU8 => ( "Invalid discriminant for enum::OneU8, received 0 while expecting one of: [ 1, ]" &[0u8] ) );
    internal_mk_test!(OneU8 => ( "Invalid size for enum::OneU8: expected 1 bytes but received 2 bytes" &[0u8, 1] ) );
}

#[test]
fn two_u8() {
    internal_mk_test!(TwoU8 => (TwoU8::One, &[42, 0, 0, 0]));
    internal_mk_test!(TwoU8 => (TwoU8::Two, &[0xFF, 0, 0xFF, 0]));

    internal_mk_test!(TwoU8 => ( "Invalid size for enum::TwoU8: expected 4 bytes but received 1 bytes" &[0] ) );
    internal_mk_test!(TwoU8 => ( "Invalid discriminant for enum::TwoU8, received 0 while expecting one of: [ 42, 16711935, ]" &[0, 0, 0, 0 ] ) );
    internal_mk_test!(TwoU8 => ( "Invalid size for enum::TwoU8: expected 4 bytes but received 6 bytes" &[0, 0, 0, 0, 0, 0 ] ) );
}

// A forward-compatible enum: any unknown `u16` discriminant decodes into the
// catch-all `Other` variant carrying the raw integer.
#[derive(Packed, Debug, PartialEq, Eq)]
#[repr(u16)]
enum Selector {
    AdminRoot = 0x0001,
    #[packed(fallback)]
    Other(u16),
}

#[test]
fn fallback_size_is_repr_width() {
    // a data-carrying variant inflates `size_of::<Selector>()`, but the wire
    // SIZE must be the repr width (2 bytes for u16).
    assert_eq!(<Selector as Packed>::SIZE, 2);
}

#[test]
fn fallback_known_discriminant_decodes_to_unit_variant() {
    // 0x0001 (LE) is the known `AdminRoot` discriminant.
    internal_mk_test!(Selector => (Selector::AdminRoot, &[0x01, 0x00]));
}

#[test]
fn fallback_unknown_discriminant_decodes_to_fallback() {
    // 0x9999 is NOT a known discriminant: it decodes into `Other(0x9999)`
    // instead of erroring, and round-trips back to the same bytes.
    internal_mk_test!(Selector => (Selector::Other(0x9999), &[0x99, 0x99]));

    // 0x0000 is also unknown and carries through verbatim.
    internal_mk_test!(Selector => (Selector::Other(0x0000), &[0x00, 0x00]));
}

#[test]
fn fallback_never_rejects_a_value() {
    // every 2-byte value is valid for a fallback enum: `check` is always Ok.
    for raw in [0x0000u16, 0x0001, 0x00FF, 0x9999, 0xFFFF] {
        let bytes = raw.to_le_bytes();
        let view = View::<Selector>::try_from_slice(&bytes)
            .expect("a fallback enum accepts every repr value");
        let decoded = view.unpack();
        let expected = if raw == 0x0001 {
            Selector::AdminRoot
        } else {
            Selector::Other(raw)
        };
        assert_eq!(decoded, expected);
    }

    // a wrong length is still rejected by the size check.
    internal_mk_test!(Selector => ( "Invalid size for enum::Selector: expected 2 bytes but received 1 bytes" &[0] ) );
}

// An enum WITHOUT a fallback still rejects an unknown discriminant — the
// historical behaviour is preserved.
#[derive(Packed, Debug, PartialEq, Eq)]
#[repr(u16)]
enum NoFallback {
    AdminRoot = 0x0001,
}

#[test]
fn without_fallback_unknown_discriminant_is_rejected() {
    internal_mk_test!(NoFallback => (NoFallback::AdminRoot, &[0x01, 0x00]));
    internal_mk_test!(NoFallback => ( "Invalid discriminant for enum::NoFallback, received 39321 while expecting one of: [ 1, ]" &[0x99, 0x99] ));
}
