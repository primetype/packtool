use crate::{Error, Packed};
use std::convert::TryInto;

impl Packed for i8 {
    const SIZE: usize = 1;
    #[inline]
    fn unchecked_read_from_slice(slice: &[u8]) -> Self {
        slice[0] as i8
    }

    #[inline]
    fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
        slice[0] = *self as u8;
    }

    #[inline]
    fn check(_slice: &[u8]) -> Result<(), Error> {
        // no need to check the size of the slice, it's already handled
        // by the [`View::try_from_slice`]
        Ok(())
    }
}
impl Packed for u8 {
    const SIZE: usize = 1;
    #[inline]
    fn unchecked_read_from_slice(slice: &[u8]) -> Self {
        slice[0]
    }

    #[inline]
    fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
        slice[0] = *self;
    }

    #[inline]
    fn check(_slice: &[u8]) -> Result<(), Error> {
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
            fn check(_slice: &[u8]) -> Result<(), Error> {
                // no need to check the size of the slice, it's already handled
                // by the [`View::try_from_slice`]
                Ok(())
            }

            #[inline]
            fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
                slice.copy_from_slice(&self.to_le_bytes())
            }

            #[inline]
            fn unchecked_read_from_slice(slice: &[u8]) -> Self {
                #[cfg(debug_assertions)]
                {
                    match slice.try_into() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::View;

    macro_rules! mk_primitive_test {
        ($f:ident, $Type:ty, $size:expr) => {
            #[quickcheck]
            fn $f(v: $Type) -> bool {
                assert_eq!(<$Type as Packed>::SIZE, $size);

                let mut slice = [0; $size];
                v.unchecked_write_to_slice(&mut slice);

                let view = View::<$Type>::try_from_slice(&slice).unwrap();
                let r = view.unpack();

                v == r
            }
        };
    }

    mk_primitive_test!(u8, u8, 1);
    mk_primitive_test!(u16, u16, 2);
    mk_primitive_test!(u32, u32, 4);
    mk_primitive_test!(u64, u64, 8);
    mk_primitive_test!(u128, u128, 16);

    mk_primitive_test!(i8, i8, 1);
    mk_primitive_test!(i16, i16, 2);
    mk_primitive_test!(i32, i32, 4);
    mk_primitive_test!(i64, i64, 8);
    mk_primitive_test!(i128, i128, 16);
}
