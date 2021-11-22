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
    ($Type:ty => ($cstr:expr, $SLICE:expr)) => {{
        let view = View::<$Type>::try_from_slice($SLICE).unwrap();
        let variant: $Type = view.unpack();
        assert_eq!(variant, $cstr);

        let mut slice = [0; <$Type as Packed>::SIZE];
        ($cstr).unchecked_write_to_slice(&mut slice);
        assert_eq!($SLICE, &slice);
    }};
    ($Type:ty => ( $error:literal $SLICE:expr )) => {{
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
