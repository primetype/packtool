use packtool::{Packed, View};

#[derive(Packed)]
#[packed(value = "string")]
pub struct TagString;
const STRING_SLICE: &[u8] = b"string";
const BAD_STRING_SLICE: &[u8] = b"almost";

#[derive(Packed)]
#[packed(value = b"bytes")]
pub struct TagBytes;
const BYTES_SLICE: &[u8] = b"bytes";
const BAD_BYTES_SLICE: &[u8] = b"about";

#[derive(Packed)]
#[packed(value = 'a')]
pub struct TagChar;
const CHAR_SLICE: &[u8] = b"a";
const BAD_CHAR_SLICE: &[u8] = b"A";

#[derive(Packed)]
#[packed(value = 0b0010_1010u8)]
pub struct TagU8;
const U8_SLICE: &[u8] = &[42];
const BAD_U8_SLICE: &[u8] = &[41];

#[derive(Packed)]
#[packed(value = 0x0011u16)]
pub struct TagU16;
const U16_SLICE: &[u8] = &[0x11, 0x00];
const BAD_U16_SLICE: &[u8] = &[0xAB, 0x00];

#[derive(Packed)]
#[packed(value = 0x0011_2233u32)]
pub struct TagU32;
const U32_SLICE: &[u8] = &[0x33, 0x22, 0x11, 0x00];
const BAD_U32_SLICE: &[u8] = &[0x33, 0x20, 0x11, 0x00];

#[derive(Packed)]
#[packed(value = 0x0011_2233_4455_6677u64)]
pub struct TagU64;
const U64_SLICE: &[u8] = &[0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00];
const BAD_U64_SLICE: &[u8] = &[0x77, 0xFF, 0xFF, 0x44, 0x44, 0x22, 0x11, 0x01];

#[derive(Packed)]
#[packed(value = 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFFu128)]
pub struct TagU128;
const U128_SLICE: &[u8] = &[
    0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00,
];
const BAD_U128_SLICE: &[u8] = &[0; 16];

#[derive(Packed)]
#[packed(value = -1i8)]
pub struct TagI8;
const I8_SLICE: &[u8] = &[0xFF];
const BAD_I8_SLICE: &[u8] = &[0x01];

#[derive(Packed)]
#[packed(value = 0x0011u16)]
pub struct TagI16;
const I16_SLICE: &[u8] = &[0x11, 0x00];
const BAD_I16_SLICE: &[u8] = &[0xAB, 0x00];

#[derive(Packed)]
#[packed(value = 0x0011_2233u32)]
pub struct TagI32;
const I32_SLICE: &[u8] = &[0x33, 0x22, 0x11, 0x00];
const BAD_I32_SLICE: &[u8] = &[0x33, 0x20, 0x11, 0x00];

#[derive(Packed)]
#[packed(value = 0x0011_2233_4455_6677u64)]
pub struct TagI64;
const I64_SLICE: &[u8] = &[0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00];
const BAD_I64_SLICE: &[u8] = &[0x77, 0xFF, 0xFF, 0x44, 0x44, 0x22, 0x11, 0x01];

#[derive(Packed)]
#[packed(value = 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFFu128)]
pub struct TagI128;
const I128_SLICE: &[u8] = &[
    0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00,
];
const BAD_I128_SLICE: &[u8] = &[0; 16];

const INVALID_SLICE: &[u8] = b"invalid slice of random length";

macro_rules! internal_mk_test {
    ($Type:ty, $SLICE:ident) => {{
        let _tag = View::<$Type>::try_from_slice($SLICE).unwrap();
    }};
    ($Type:ty, ! $SLICE:ident) => {{
        let _err = View::<$Type>::try_from_slice($SLICE).unwrap_err();
    }};
}

macro_rules! mk_test {
    (
        $name:ident<$Type:ty>(
            $( [ $($unit:tt)+ ] ),*
        )
    ) => {
        #[test]
        fn $name() {
            $(
                internal_mk_test!(
                    $Type,
                    $($unit)+
                );
            )*
        }
    };
}

mk_test!(string<TagString>(
    [STRING_SLICE],
    [!INVALID_SLICE],
    [!BAD_STRING_SLICE]
));
mk_test!(bytes<TagBytes>(
    [BYTES_SLICE],
    [!INVALID_SLICE],
    [!BAD_BYTES_SLICE]
));
mk_test!(char<TagChar>(
    [CHAR_SLICE],
    [!INVALID_SLICE],
    [!BAD_CHAR_SLICE]
));

mk_test!(u8<TagU8>(
    [U8_SLICE],
    [!INVALID_SLICE],
    [!BAD_U8_SLICE]
));
mk_test!(u16<TagU16>(
    [U16_SLICE],
    [!INVALID_SLICE],
    [!BAD_U16_SLICE]
));
mk_test!(u32<TagU32>(
    [U32_SLICE],
    [!INVALID_SLICE],
    [!BAD_U32_SLICE]
));
mk_test!(u64<TagU64>(
    [U64_SLICE],
    [!INVALID_SLICE],
    [!BAD_U64_SLICE]
));
mk_test!(u128<TagU128>(
    [U128_SLICE],
    [!INVALID_SLICE],
    [!BAD_U128_SLICE]
));

mk_test!(i8<TagI8>(
    [I8_SLICE],
    [!INVALID_SLICE],
    [!BAD_I8_SLICE]
));
mk_test!(i16<TagI16>(
    [I16_SLICE],
    [!INVALID_SLICE],
    [!BAD_I16_SLICE]
));
mk_test!(i32<TagI32>(
    [I32_SLICE],
    [!INVALID_SLICE],
    [!BAD_I32_SLICE]
));
mk_test!(i64<TagI64>(
    [I64_SLICE],
    [!INVALID_SLICE],
    [!BAD_I64_SLICE]
));
mk_test!(i128<TagI128>(
    [I128_SLICE],
    [!INVALID_SLICE],
    [!BAD_I128_SLICE]
));
