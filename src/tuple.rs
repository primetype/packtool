/*!
Module only to define the default packed implementation for all
the kind of tuples we will want to support
*/

use crate::{Packed, View};
use anyhow::Result;

macro_rules! tuple_impls {
    ($(
        $Tuple:ident {
            $(($idx:tt) -> $T:ident)+
        }
    )+) => {
        $(
        impl<$($T:Packed),+>  Packed for ($($T,)+)
        {
            const SIZE: usize = 0 $(
                + <$T as Packed>::SIZE
            )+;

            fn check(slice: &[u8]) -> Result<()> {
                $(
                    <$T as Packed>::check(
                        &slice.as_ref()[
                            ($idx * $T::SIZE)
                            ..
                            (($idx + 1) * $T::SIZE)
                        ]
                    )?;
                )+

                Ok(())
            }

            fn unchecked_read_from_slice(slice: View<'_, Self>) -> Self {
                (
                    $(
                        <$T as Packed>::unchecked_read_from_slice(
                            View::new(&slice.as_ref()[
                                ($idx * $T::SIZE)
                                ..
                                (($idx + 1) * $T::SIZE)
                            ])
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
        (0) -> A
    }
    Tuple2 {
        (0) -> A
        (1) -> B
    }
    Tuple3 {
        (0) -> A
        (1) -> B
        (2) -> C
    }
    Tuple4 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
    }
    Tuple5 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
    }
    Tuple6 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
    }
    Tuple7 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
    }
    Tuple8 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
    }
    Tuple9 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
    }
    Tuple10 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
    }
    Tuple11 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
    }
    Tuple12 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
        (11) -> L
    }
}
