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
use packtool::{Packed, View};
# use packtool::Error;

/// a unit that is always the utf8 string `"my protocol"`
/// and takes 11 bytes in the packed structure
#[derive(Packed)]
#[packed(value = "my protocol")]
pub struct ProtocolPrefix;

/// a unit that is always `4` and takes 1 byte long
#[derive(Packed)]
#[packed(value = 0b0000_0100u8)]
pub struct OtherUnit();

/// a unit that is always `0xcafe` and takes 4 bytes
/// in the packed structure
#[derive(Packed)]
#[packed(value = 0xcafeu32)]
pub struct LastButNotLeast {}

# fn test() -> Result<(), Error> {
const SLICE: &[u8] = b"my protocol";
let view: View<'_, ProtocolPrefix> = View::try_from_slice(SLICE)?;

# Ok(()) }
# test().unwrap();

# assert_eq!(ProtocolPrefix::SIZE, 11);
# assert_eq!(OtherUnit::SIZE, 1);
# assert_eq!(LastButNotLeast::SIZE, 4);
```

Here we are expecting the `ProtocolPrefix` to always have the
same value in the packed representation. When serializing the
`ProtocolPrefix`, the `value` will be set with these 11
characters.

## Enumeration

Only enumerations without fields are allowed for now.

```
use packtool::{Packed, View};
# use packtool::Error;

#[derive(Packed)]
#[repr(u8)]
pub enum Version {
    V1 = 1,
    V2 = 2,
}

# fn test() -> Result<(), Error> {
# const SLICE: &[u8] = &[1];
let view: View<'_, Version> = View::try_from_slice(SLICE)?;

assert!(matches!(view.unpack(), Version::V1));

# Ok(()) }
# test().unwrap();
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

Enumerations with data-carrying variants are not supported yet and
are rejected at compile time:

```compile_fail
use packtool::Packed;

#[derive(Packed)]
pub enum ThisOrThat {
    This,
    That(u32),
}
```

A single catch-all `#[packed(fallback)]` variant is the one supported
data-carrying shape: an unknown discriminant decodes into it instead of
erroring, making the packed enum forward-compatible. The fallback is a
single-field tuple variant with no discriminant whose field type is the
`#[repr(...)]` integer, and it carries exactly the repr width on the wire:

```
use packtool::{Packed, View};

#[derive(Packed, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Selector {
    AdminRoot = 0x0001,
    #[packed(fallback)]
    Other(u16),
}

# fn test() -> Result<(), packtool::Error> {
let view: View<'_, Selector> = View::try_from_slice(&[0x99, 0x99])?;
assert_eq!(view.unpack(), Selector::Other(0x9999));
# Ok(()) }
# test().unwrap();
# assert_eq!(Selector::SIZE, 2);
```

The fallback exists for **decode** of genuinely-unknown values only. Because a
known variant and `Fallback(known_value)` encode to the same wire bytes,
hand-constructing the fallback with a value that equals a known discriminant
does not round-trip: decoding those bytes always yields the known unit variant.
For the `Selector` above, `Packet::pack(&Selector::Other(0x0001)).unpack()`
decodes back to `Selector::AdminRoot`, not `Selector::Other(0x0001)`. This never
arises from decoding — decode only ever produces the fallback for
genuinely-unknown values, which always round-trip — so it is a misuse caveat,
not a correctness issue:

```
use packtool::{Packed, Packet};

#[derive(Packed, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Selector {
    AdminRoot = 0x0001,
    #[packed(fallback)]
    Other(u16),
}

// a genuinely-unknown value round-trips through the fallback:
assert_eq!(
    Packet::pack(&Selector::Other(0x9999)).unpack(),
    Selector::Other(0x9999),
);
// but a fallback holding a known discriminant canonicalises on decode:
assert_eq!(
    Packet::pack(&Selector::Other(0x0001)).unpack(),
    Selector::AdminRoot,
);
```

A fallback whose field type does not match the `#[repr(...)]` is rejected:

```compile_fail
use packtool::Packed;

#[derive(Packed)]
#[repr(u16)]
pub enum Selector {
    AdminRoot = 0x0001,
    #[packed(fallback)]
    Other(u32),
}
```

More than one `#[packed(fallback)]` variant is rejected:

```compile_fail
use packtool::Packed;

#[derive(Packed)]
#[repr(u16)]
pub enum Selector {
    AdminRoot = 0x0001,
    #[packed(fallback)]
    Other(u16),
    #[packed(fallback)]
    Another(u16),
}
```

A fallback variant carrying more than one field is rejected:

```compile_fail
use packtool::Packed;

#[derive(Packed)]
#[repr(u16)]
pub enum Selector {
    AdminRoot = 0x0001,
    #[packed(fallback)]
    Other(u16, u16),
}
```

A fallback variant that declares a discriminant is rejected:

```compile_fail
use packtool::Packed;

#[derive(Packed)]
#[repr(u16)]
pub enum Selector {
    AdminRoot = 0x0001,
    #[packed(fallback)]
    Other(u16) = 5,
}
```

Unions cannot be packed and are rejected at compile time:

```compile_fail
use packtool::Packed;

#[derive(Packed)]
pub union Choice {
    a: u32,
    b: f32,
}
```

Type-parameter generics are supported on **named** structs. The packed layout
and `SIZE` are computed through `<T as Packed>::SIZE`, and a `T: Packed` bound is
injected for every type parameter (composing with any `where` clause you write):

```
use packtool::Packed;

#[derive(Packed)]
pub struct Log<T> {
    parent_id: [u8; 64],
    content: T,
    signature: [u8; 64],
}

# assert_eq!(<Log<u32> as Packed>::SIZE, 64 + 4 + 64);
```

Generics on enums and on tuple structs are still rejected at compile time:

```compile_fail
use packtool::Packed;

#[derive(Packed)]
pub struct Wrapper<T>(T);
```

```compile_fail
use packtool::Packed;

#[derive(Packed)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
```

A generic struct with no fields is rejected too: its type parameters could not
appear in the (empty) packed layout.

```compile_fail
use packtool::Packed;

#[derive(Packed)]
pub struct Empty<T> {}
```

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

Each of the packed objects have a view accessor for each fields:

* for named fields, the name of the accessor is the name of the field
* for tuples, the name of the accessor is the index of the field preceded by an underscore (`_`): `_0`, `_1` etc.

```
# use packtool::{Packed, View, Packet};
#
# #[derive(Packed)]
# #[packed(value = "packcoin")]
# pub struct Tag;
#
# /// 1 byte that will be used to store a version number
# #[derive(Packed)]
# #[repr(u8)]
# pub enum Version {
#     V1 = 1,
#     V2 = 2,
# }
#
# /// 8 bytes that will be used to store a block number
# #[derive(Packed)]
# pub struct BlockNumber(u32, u32);
#
# /// 9 bytes packed header
# #[derive(Packed)]
# pub struct Header {
#     tag: Tag,
#     version: Version,
#     block_number: BlockNumber
# }
#
# let header = Header { tag: Tag, version: Version::V1, block_number: BlockNumber(0, 1) };
# let header = Packet::pack(&header);
# let header = header.view();
#
let tag: View<'_, Tag> = Header::tag(header);
let block_number: View<'_, BlockNumber> = Header::block_number(header);

let epoch: View<'_, u32> = BlockNumber::_0(block_number);
let slot: u32  = BlockNumber::_1(block_number).unpack();
#
# assert_eq!(slot, 1);
```

You can rename the accessor with the attribute `accessor`:

```
# use packtool::{Packed, View, Packet};
#
#[derive(Packed)]
pub struct BlockNumber(
    #[packed(accessor = "epoch")]
    u32,
    #[packed(accessor = "slot")]
    u32
);
#
# let block_number = Packet::pack(&BlockNumber(0, 1));
# let block_number = block_number.view();
let epoch = BlockNumber::epoch(block_number); // instead of _0
let slot = BlockNumber::slot(block_number).unpack(); // instead of _1
#
# assert_eq!(slot, 1);
```

It is also possible to prevent the accessor to be created. You can set
the accessor with a literal boolean to say if you want the accessor or
not. `true` will simply means the default case (use the index of the field
or use the name for the name of the accessor):

```
# use packtool::{Packed, View, Packet};
#
#[derive(Packed)]
pub struct Hash(
    #[packed(accessor = true)]
    [u8; 32]
);
#
# let hash = Packet::pack(&Hash([0; 32]));
# let hash = hash.view();
let bytes = Hash::_0(hash);
# assert_eq!(bytes.unpack(), [0; 32]);
```

However if you set it to `false` there will be no accessor created for you:

```compile_fail
# use packtool::{Packed, View, Packet};
#
#[derive(Packed)]
pub struct Hash(
    #[packed(accessor = false)]
    [u8; 32]
);
#
# let hash = Packet::pack(&Hash([0; 32]));
# let hash = hash.view();
let bytes = Hash::_0(hash);
```

*/

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod array;
mod error;
mod packet;
mod primitives;
mod tuple;
mod view;

pub use self::{
    error::{Context, Error},
    packet::Packet,
    view::View,
};
pub use packtool_macro::Packed;

/// trait to define how a fixed size Packed object is serialized
/// into a byte slice representation.
///
/// see crate documentation for more information.
pub trait Packed: Sized {
    /// the static size of a packed object in a byte array
    ///
    /// this is not necessarily the [`::std::mem::size_of::<Self>()`]
    /// but the size it takes to have this object on a slice of memory.
    const SIZE: usize;

    /// assuming the given slice if valid, perform a conversion
    /// from the slice to the object.
    fn unchecked_read_from_slice(slice: &[u8]) -> Self;

    /// assuming there is enough slice available in the
    fn unchecked_write_to_slice(&self, _slice: &mut [u8]);

    /// check the validity of the given slice to hold the appropriate value
    ///
    /// the length of the slice is already handled by the [`View::try_from_slice`]
    /// method so no need to do that again in here.
    fn check(slice: &[u8]) -> Result<(), Error>;

    /// assuming the given slice if valid, perform a conversion
    /// from the slice to the object.
    ///
    /// it should be assumed the `checks` have been performed
    /// appropriately since we are passing in the [`View`]
    /// and not the raw slice.
    #[inline]
    fn read(view: View<'_, Self>) -> Self {
        Self::unchecked_read_from_slice(view.as_ref())
    }
}
