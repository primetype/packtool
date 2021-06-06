/*!
Module only to define the default packed implementation for all
the kind of tuples we will want to support
*/

use crate::{Packed, Result, View};

macro_rules! array_impls {
    ($(
        Array $n:expr => {
            $([$idx:tt])+
        }
    )+) => {
        $(
        impl<T: Packed>  Packed for [T; $n]
        {
            const SIZE: usize = <T as Packed>::SIZE * $n;

            #[inline]
            fn check(slice: &[u8]) -> Result<()> {
                $(
                    <T as Packed>::check(
                        &slice.as_ref()[
                            ($idx * T::SIZE)
                            ..
                            (($idx + 1) * T::SIZE)
                        ]
                    )?;
                )+

                Ok(())
            }

            #[inline]
            fn unchecked_read_from_slice(slice: View<'_, Self>) -> Self {
                [
                    $(
                        <T as Packed>::unchecked_read_from_slice(
                            View::new(&slice.as_ref()[
                                ($idx * T::SIZE)
                                ..
                                (($idx + 1) * T::SIZE)
                            ])
                        )
                    ),+
                ]
            }
        })+
    };
}

array_impls! {
    Array 1 => {
        [0]
    }
    Array 2 => {
        [0]
        [1]
    }
    Array 3 => {
        [0]
        [1]
        [2]
    }
    Array 4 => {
        [0]
        [1]
        [2]
        [3]
    }
    Array 5 => {
        [0]
        [1]
        [2]
        [3]
        [4]
    }
    Array 6 => {
        [0]
        [1]
        [2]
        [3]
        [4]
        [5]
    }
    Array 7 => {
        [0]
        [1]
        [2]
        [3]
        [4]
        [5]
        [6]
    }
    Array 8 => {
        [0]
        [1]
        [2]
        [3]
        [4]
        [5]
        [6]
        [7]
    }
    Array 16 => {
        [0]
        [1]
        [2]
        [3]
        [4]
        [5]
        [6]
        [7]
        [8]
        [9]
        [10]
        [11]
        [12]
        [13]
        [14]
        [15]
    }
    Array 32 => {
        [0]
        [1]
        [2]
        [3]
        [4]
        [5]
        [6]
        [7]
        [8]
        [9]
        [10]
        [11]
        [12]
        [13]
        [14]
        [15]
        [16]
        [17]
        [18]
        [19]
        [20]
        [21]
        [22]
        [23]
        [24]
        [25]
        [26]
        [27]
        [28]
        [29]
        [30]
        [31]
    }
    Array 64 => {
        [0]
        [1]
        [2]
        [3]
        [4]
        [5]
        [6]
        [7]
        [8]
        [9]
        [10]
        [11]
        [12]
        [13]
        [14]
        [15]
        [16]
        [17]
        [18]
        [19]
        [20]
        [21]
        [22]
        [23]
        [24]
        [25]
        [26]
        [27]
        [28]
        [29]
        [30]
        [31]
        [32]
        [33]
        [34]
        [35]
        [36]
        [37]
        [38]
        [39]
        [40]
        [41]
        [42]
        [43]
        [44]
        [45]
        [46]
        [47]
        [48]
        [49]
        [50]
        [51]
        [52]
        [53]
        [54]
        [55]
        [56]
        [57]
        [58]
        [59]
        [60]
        [61]
        [62]
        [63]
    }
}
