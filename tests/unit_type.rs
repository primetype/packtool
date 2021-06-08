use packtool::{Packed, View};

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 1u8)]
struct StructTupleUnit();
const STRUCT_TUPLE_UNIT_SLICE: &[u8] = &[1];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 1u8)]
struct StructBraceUnit {}
const STRUCT_BRACE_UNIT_SLICE: &[u8] = &[1];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = "string")]
pub struct TagString;
const STRING_SLICE: &[u8] = b"string";
const BAD_STRING_SLICE: &[u8] = b"almost";

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = b"bytes")]
pub struct TagBytes;
const BYTES_SLICE: &[u8] = b"bytes";
const BAD_BYTES_SLICE: &[u8] = b"about";

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 'a')]
pub struct TagChar;
const CHAR_SLICE: &[u8] = b"a";
const BAD_CHAR_SLICE: &[u8] = b"A";

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 0b0010_1010u8)]
pub struct TagU8;
const U8_SLICE: &[u8] = &[42];
const BAD_U8_SLICE: &[u8] = &[41];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 0x0011u16)]
pub struct TagU16;
const U16_SLICE: &[u8] = &[0x11, 0x00];
const BAD_U16_SLICE: &[u8] = &[0xAB, 0x00];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 0x0011_2233u32)]
pub struct TagU32;
const U32_SLICE: &[u8] = &[0x33, 0x22, 0x11, 0x00];
const BAD_U32_SLICE: &[u8] = &[0x33, 0x20, 0x11, 0x00];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 0x0011_2233_4455_6677u64)]
pub struct TagU64;
const U64_SLICE: &[u8] = &[0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00];
const BAD_U64_SLICE: &[u8] = &[0x77, 0xFF, 0xFF, 0x44, 0x44, 0x22, 0x11, 0x01];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFFu128)]
pub struct TagU128;
const U128_SLICE: &[u8] = &[
    0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00,
];
const BAD_U128_SLICE: &[u8] = &[0; 16];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = -1i8)]
pub struct TagI8;
const I8_SLICE: &[u8] = &[0xFF];
const BAD_I8_SLICE: &[u8] = &[0x01];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 0x0011i16)]
pub struct TagI16;
const I16_SLICE: &[u8] = &[0x11, 0x00];
const BAD_I16_SLICE: &[u8] = &[0xAB, 0x00];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 0x0011_2233i32)]
pub struct TagI32;
const I32_SLICE: &[u8] = &[0x33, 0x22, 0x11, 0x00];
const BAD_I32_SLICE: &[u8] = &[0x33, 0x20, 0x11, 0x00];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 0x0011_2233_4455_6677i64)]
pub struct TagI64;
const I64_SLICE: &[u8] = &[0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00];
const BAD_I64_SLICE: &[u8] = &[0x77, 0xFF, 0xFF, 0x44, 0x44, 0x22, 0x11, 0x01];

#[derive(Packed, PartialEq, Eq, Debug)]
#[packed(value = 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFFi128)]
pub struct TagI128;
const I128_SLICE: &[u8] = &[
    0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00,
];
const BAD_I128_SLICE: &[u8] = &[0; 16];

const INVALID_SLICE: &[u8] = b"invalid slice of random length";

macro_rules! internal_mk_test {
    ($cstr:expr, $Type:ty, $SLICE:ident) => {{
        let tag = View::<$Type>::try_from_slice($SLICE).unwrap();
        let value: $Type = tag.unpack();
        assert_eq!(value, $cstr);

        let mut slice = [0; <$Type as Packed>::SIZE];
        ($cstr).unchecked_write_to_slice(&mut slice);

        assert_eq!(slice, $SLICE);
    }};
    ($cstr:expr, $Type:ty, ! $SLICE:ident $error:literal) => {{
        let err = View::<$Type>::try_from_slice($SLICE).unwrap_err();

        assert_eq!(err.to_string(), $error);
    }};
}

macro_rules! mk_test {
    (
        $cstr:expr,
        $name:ident<$Type:ty>(
            $( [ $($unit:tt)+ ] ),*
        )
    ) => {
        #[test]
        fn $name() {
            $(
                internal_mk_test!(
                    $cstr,
                    $Type,
                    $($unit)+
                );
            )*
        }
    };
}

mk_test!(
    StructTupleUnit (),
    struct_tuple_unit<StructTupleUnit>(
    [STRUCT_TUPLE_UNIT_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::StructTupleUnit: expected 1 bytes but received 30 bytes"]
));
mk_test!(
    StructBraceUnit {},
    struct_brace_unit<StructBraceUnit>(
    [STRUCT_BRACE_UNIT_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::StructBraceUnit: expected 1 bytes but received 30 bytes"]
));

mk_test!(
    TagString,
    string<TagString>(
    [STRING_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagString: expected 6 bytes but received 30 bytes"],
    [!BAD_STRING_SLICE "Assumption `slice == \"string\".as_bytes()` failed for unit_type::TagString: Invalid string, expected string but received almost"]
));
mk_test!(
    TagBytes,
    bytes<TagBytes>(
    [BYTES_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagBytes: expected 5 bytes but received 30 bytes"],
    [!BAD_BYTES_SLICE "Assumption `slice == b\"bytes\"` failed for unit_type::TagBytes: Invalid string, expected [98, 121, 116, 101, 115] but received [97, 98, 111, 117, 116]"]
));
mk_test!(
    TagChar, char<TagChar>(
    [CHAR_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagChar: expected 1 bytes but received 30 bytes"],
    [!BAD_CHAR_SLICE "Assumption `c.chars().next() == Some(\'a\')` failed for unit_type::TagChar: Invalid UTF8 encoded char, expected a but received A"]
));

mk_test!(TagU8, u8<TagU8>(
    [U8_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagU8: expected 1 bytes but received 30 bytes"],
    [!BAD_U8_SLICE "Assumption `int == 0b0010_1010u8` failed for u8: Invalid packed integer, expected 42 but received 41"]
));
mk_test!(TagU16, u16<TagU16>(
    [U16_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagU16: expected 2 bytes but received 30 bytes"],
    [!BAD_U16_SLICE "Assumption `int == 0x0011u16` failed for u16: Invalid packed integer, expected 17 but received 171"]
));
mk_test!(TagU32,u32<TagU32>(
    [U32_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagU32: expected 4 bytes but received 30 bytes"],
    [!BAD_U32_SLICE "Assumption `int == 0x0011_2233u32` failed for u32: Invalid packed integer, expected 1122867 but received 1122355"]
));
mk_test!(TagU64,u64<TagU64>(
    [U64_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagU64: expected 8 bytes but received 30 bytes"],
    [!BAD_U64_SLICE "Assumption `int == 0x0011_2233_4455_6677u64` failed for u64: Invalid packed integer, expected 4822678189205111 but received 76880345252757367"]
));
mk_test!(TagU128,u128<TagU128>(
    [U128_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagU128: expected 16 bytes but received 30 bytes"],
    [!BAD_U128_SLICE "Assumption `int == 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFFu128` failed for u128: Invalid packed integer, expected 88962710306127702866241727433142015 but received 0"]
));

mk_test!(TagI8,i8<TagI8>(
    [I8_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagI8: expected 1 bytes but received 30 bytes"],
    [!BAD_I8_SLICE "Assumption `int == -1i8` failed for i8: Invalid packed integer, expected -1 but received 1"]
));
mk_test!(TagI16,i16<TagI16>(
    [I16_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagI16: expected 2 bytes but received 30 bytes"],
    [!BAD_I16_SLICE "Assumption `int == 0x0011i16` failed for i16: Invalid packed integer, expected 17 but received 171"]
));
mk_test!(TagI32,i32<TagI32>(
    [I32_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagI32: expected 4 bytes but received 30 bytes"],
    [!BAD_I32_SLICE "Assumption `int == 0x0011_2233i32` failed for i32: Invalid packed integer, expected 1122867 but received 1122355"]
));
mk_test!(TagI64,i64<TagI64>(
    [I64_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagI64: expected 8 bytes but received 30 bytes"],
    [!BAD_I64_SLICE "Assumption `int == 0x0011_2233_4455_6677i64` failed for i64: Invalid packed integer, expected 4822678189205111 but received 76880345252757367"]
));
mk_test!(TagI128,i128<TagI128>(
    [I128_SLICE],
    [!INVALID_SLICE "Invalid size for unit_type::TagI128: expected 16 bytes but received 30 bytes"],
    [!BAD_I128_SLICE "Assumption `int == 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFFi128` failed for i128: Invalid packed integer, expected 88962710306127702866241727433142015 but received 0"]
));
