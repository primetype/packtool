/*!
`packtool` is a packing library. Useful to define how serializing
and deserializing data from a type level definition.

# Example

## Unit types

unit types can be packed. What this means is that the object
is known to have the same constant value. That way it is possible
to define values that are expected to be found and to be the same.

All [`Packed`] unit structures must have a `#[packed(value = ...)]`
attribute. The value can be set to any literal except: `bool`, `float`.

```
use packtool::Packed;

/// a unit that is always the utf8 string `"my protocol"`
/// and takes 11 bytes in the packed structure
#[derive(Packed)]
#[packed(value = "my protocol")]
pub struct ProtocolPrefix;

#[derive(Packed)]
#[packed(value = b"raw bytes")]
pub enum EnumUnit {}

/// a unit that is always `4` and takes 1 byte long
#[derive(Packed)]
#[packed(value = 0b0000_0100u8)]
pub struct OtherUnit();

/// a unit that is always `0xcafe` and takes 4 bytes
/// in the packed structure
#[derive(Packed)]
#[packed(value = 0xcafeu32)]
pub struct LastButNotLeast {}

# assert_eq!(ProtocolPrefix::SIZE, 11);
# assert_eq!(EnumUnit::SIZE, 9);
# assert_eq!(OtherUnit::SIZE, 1);
# assert_eq!(LastButNotLeast::SIZE, 4);
```

Here we are expecting the `ProtocolPrefix` to always have the
same value in the packed representation. When serializing the
`ProtocolPrefix`, the `value` will be set with these 11
characters.

## combining packed objects

It is possible to compose packed objects in named or tuple structures.

```
use packtool::Packed;

#[derive(Packed)]
#[packed(value = "packcoin")]
pub struct Tag;

/// 1 byte that will be used to store a version number
#[derive(Packed)]
#[repr(u8)]
pub enum Version {
    V1 = 1,
    V2 = 2,
}

/// 8 bytes that will be used to store a block number
#[derive(Packed)]
pub struct BlockNumber(u32, u32);

/// 9 bytes packed header
#[derive(Packed)]
pub struct Header {
    tag: Tag,
    version: Version,
    block_number: BlockNumber
}

# assert_eq!(Version::SIZE, 1);
# assert_eq!(BlockNumber::SIZE, 8);
# assert_eq!(Header::SIZE, 17);
```

## Enumeration

Only enumerations without fields are allowed for now.

```
use packtool::Packed;

#[derive(Packed)]
#[repr(u8)]
pub enum Version {
    V1 = 1,
    V2 = 2,
}

# assert_eq!(Version::SIZE, 1);
```

the `repr(...)` is necessary in order to set a size to the enum.

```compile_fail
use packtool::Packed;

#[derive(Packed)]
pub enum Color {
    Red = 1,
    Green = 2,
    Blue = -1
}
```
*/

mod array;
mod tuple;
mod view;

use std::convert::TryInto;

pub use self::view::View;
pub use anyhow::{ensure, Context, Result};
pub use packtool_macro::Packed;

pub trait Packed: Sized {
    /// the static size of a packed object in a byte array
    ///
    /// this is not necessarily the [`::std::mem::size_of::<Self>()`]
    /// but the size it takes to have this object on a slice of memory.
    const SIZE: usize;

    /// assuming the given slice if valid, perform a conversion
    /// from the slice to the object.
    ///
    /// it should be assumed the `checks` have been performed
    /// appropriately since we are passing in the [`View`]
    /// and not the raw slice.
    fn unchecked_read_from_slice(view: View<'_, Self>) -> Self {
        todo!()
    }

    /// check the validity of the given slice to hold the appropriate value
    ///
    /// the length of the slice is already handled by the [`View::try_from_slice`]
    /// method so no need to do that again in here.
    ///
    fn check(slice: &[u8]) -> Result<()>;
}

impl Packed for i8 {
    const SIZE: usize = 1;
    #[inline]
    fn unchecked_read_from_slice(slice: View<'_, Self>) -> Self {
        slice.as_ref()[0] as i8
    }

    #[inline]
    fn check(_slice: &[u8]) -> Result<()> {
        // no need to check the size of the slice, it's already handled
        // by the [`View::try_from_slice`]
        Ok(())
    }
}
impl Packed for u8 {
    const SIZE: usize = 1;
    #[inline]
    fn unchecked_read_from_slice(slice: View<'_, Self>) -> Self {
        slice.as_ref()[0]
    }

    #[inline]
    fn check(_slice: &[u8]) -> Result<()> {
        // no need to check the size of the slice, it's already handled
        // by the [`View::try_from_slice`]
        Ok(())
    }
}

macro_rules! primitive_pack {
    ($t:ty) => {
        impl Packed for $t {
            const SIZE: usize = ::std::mem::size_of::<$t>();

            #[inline]
            fn check(_slice: &[u8]) -> Result<()> {
                // no need to check the size of the slice, it's already handled
                // by the [`View::try_from_slice`]
                Ok(())
            }

            #[inline]
            fn unchecked_read_from_slice(slice: View<'_, Self>) -> Self {
                #[cfg(debug_assertions)]
                {
                    match slice.as_ref().try_into() {
                        Ok(bytes) => <$t>::from_le_bytes(bytes),
                        Err(error) => {
                            panic!(
                                "Failed read {ty} from slice: {error}",
                                ty = ::core::any::type_name::<$t>(),
                                error = error,
                            )
                        }
                    }
                }
                #[cfg(not(debug_assertions))]
                {
                    if let Ok(bytes) = slice.as_ref().try_into() {
                        <$t>::from_le_bytes(bytes)
                    } else {
                        unsafe { ::core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    };
}

primitive_pack!(u16);
primitive_pack!(u32);
primitive_pack!(u64);
primitive_pack!(u128);
primitive_pack!(usize);
primitive_pack!(i16);
primitive_pack!(i32);
primitive_pack!(i64);
primitive_pack!(i128);
primitive_pack!(isize);
