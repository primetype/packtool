/*!
Module only to define the default packed implementation for all
the kind of tuples we will want to support
*/

use crate::{Error, Packed};

macro_rules! range {
    ($($Pred:ident)* , $Type:ident) => {
        ( $(<$Pred as Packed>::SIZE + )* 0)
        ..
        ( $(<$Pred as Packed>::SIZE + )* <$Type as Packed>::SIZE)
    };
}

macro_rules! tuple_impls {
    ($(
        $Tuple:ident {
            $(($idx:tt) -> $T:ident [$($Pred:ident),*] )+
        }
    )+) => {
        $(
        impl<$($T:Packed),+>  Packed for ($($T,)+)
        {
            const SIZE: usize = 0 $(
                + <$T as Packed>::SIZE
            )+;

            fn check(slice: &[u8]) -> Result<(), Error> {
                $(
                    <$T as Packed>::check(
                        &slice.as_ref()[
                            range!($($Pred)* , $T)
                        ]
                    )?;
                )+

                Ok(())
            }

            #[inline]
            fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
                $(
                    <$T as Packed>::unchecked_write_to_slice(
                        &self.$idx,
                        &mut slice[
                            range!($($Pred)* , $T)
                        ]
                    )
                );+
            }

            fn unchecked_read_from_slice(slice: &[u8]) -> Self {
                (
                    $(
                        <$T as Packed>::unchecked_read_from_slice(
                            &slice[
                                range!($($Pred)* , $T)
                            ]
                        )
                        ,
                    )+
                )
            }
        })+
    };
}

tuple_impls! {
    Tuple1 {
        (0) -> A []
    }
    Tuple2 {
        (0) -> A []
        (1) -> B [A]
    }
    Tuple3 {
        (0) -> A []
        (1) -> B [A]
        (2) -> C [A, B]
    }
    Tuple4 {
        (0) -> A []
        (1) -> B [A]
        (2) -> C [A, B]
        (3) -> D [A, B, C]
    }
    Tuple5 {
        (0) -> A []
        (1) -> B [A]
        (2) -> C [A, B]
        (3) -> D [A, B, C]
        (4) -> E [A, B, C, D]
    }
    Tuple6 {
        (0) -> A []
        (1) -> B [A]
        (2) -> C [A, B]
        (3) -> D [A, B, C]
        (4) -> E [A, B, C, D]
        (5) -> F [A, B, C, D, E]
    }
    Tuple7 {
        (0) -> A []
        (1) -> B [A]
        (2) -> C [A, B]
        (3) -> D [A, B, C]
        (4) -> E [A, B, C, D]
        (5) -> F [A, B, C, D, E]
        (6) -> G [A, B, C, D, E, F]
    }
    Tuple8 {
        (0) -> A []
        (1) -> B [A]
        (2) -> C [A, B]
        (3) -> D [A, B, C]
        (4) -> E [A, B, C, D]
        (5) -> F [A, B, C, D, E]
        (6) -> G [A, B, C, D, E, F]
        (7) -> H [A, B, C, D, E, F, G]
    }
    Tuple9 {
        (0) -> A []
        (1) -> B [A]
        (2) -> C [A, B]
        (3) -> D [A, B, C]
        (4) -> E [A, B, C, D]
        (5) -> F [A, B, C, D, E]
        (6) -> G [A, B, C, D, E, F]
        (7) -> H [A, B, C, D, E, F, G]
        (8) -> I [A, B, C, D, E, F, G, H]
    }
    Tuple10 {
        (0) -> A []
        (1) -> B [A]
        (2) -> C [A, B]
        (3) -> D [A, B, C]
        (4) -> E [A, B, C, D]
        (5) -> F [A, B, C, D, E]
        (6) -> G [A, B, C, D, E, F]
        (7) -> H [A, B, C, D, E, F, G]
        (8) -> I [A, B, C, D, E, F, G, H]
        (9) -> J [A, B, C, D, E, F, G, H, I]
    }
    Tuple11 {
        (0) -> A  []
        (1) -> B  [A]
        (2) -> C  [A, B]
        (3) -> D  [A, B, C]
        (4) -> E  [A, B, C, D]
        (5) -> F  [A, B, C, D, E]
        (6) -> G  [A, B, C, D, E, F]
        (7) -> H  [A, B, C, D, E, F, G]
        (8) -> I  [A, B, C, D, E, F, G, H]
        (9) -> J  [A, B, C, D, E, F, G, H, I]
        (10) -> K [A, B, C, D, E, F, G, H, I, J]
    }
    Tuple12 {
        (0) -> A  []
        (1) -> B  [A]
        (2) -> C  [A, B]
        (3) -> D  [A, B, C]
        (4) -> E  [A, B, C, D]
        (5) -> F  [A, B, C, D, E]
        (6) -> G  [A, B, C, D, E, F]
        (7) -> H  [A, B, C, D, E, F, G]
        (8) -> I  [A, B, C, D, E, F, G, H]
        (9) -> J  [A, B, C, D, E, F, G, H, I]
        (10) -> K [A, B, C, D, E, F, G, H, I, J]
        (11) -> L [A, B, C, D, E, F, G, H, I, J, K]
    }
}

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

    mk_primitive_test!(tuple1, (u8,), 1);
    mk_primitive_test!(tuple2, (u8, u8), 2);
    mk_primitive_test!(tuple3, (u8, u32, u8), 6);
    mk_primitive_test!(tuple4, (u8, u16, u8, u32), 8);
    mk_primitive_test!(tuple5, (u8, u8, u16, u8, u32), 9);
    mk_primitive_test!(tuple6, (u8, u64, u8, u128, u8, u8), 28);
    mk_primitive_test!(tuple7, (u8, u64, u8, u128, u8, u8, i16), 30);
    mk_primitive_test!(tuple8, (u8, u64, i32, u8, u128, u8, u8, i16), 34);
}
