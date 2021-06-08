use crate::{Error, Packed};
use std::convert::TryInto as _;

impl<const SIZE: usize> Packed for [u8; SIZE] {
    const SIZE: usize = SIZE;

    #[inline]
    fn check(_slice: &[u8]) -> Result<(), Error> {
        Ok(())
    }

    #[inline]
    fn unchecked_read_from_slice(slice: &[u8]) -> Self {
        match slice.try_into() {
            Ok(array) => array,
            Err(error) => {
                panic!("the slice does not have the appropriate size {}", error)
            }
        }
    }

    #[inline]
    fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
        slice.copy_from_slice(self)
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{Arbitrary, Gen};

    use super::*;
    use crate::View;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    struct Array<const SIZE: usize>([u8; SIZE]);

    impl<const SIZE: usize> Arbitrary for Array<SIZE> {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut array = Array([0; SIZE]);

            array.0.iter_mut().for_each(|byte| {
                *byte = u8::arbitrary(g);
            });

            array
        }
    }

    macro_rules! mk_primitive_test {
        ($f:ident, $SIZE:expr) => {
            #[quickcheck]
            fn $f(Array(v): Array<$SIZE>) -> bool {
                let mut slice = [0; $SIZE];
                v.unchecked_write_to_slice(&mut slice);

                let view = View::<[u8; $SIZE]>::try_from_slice(&slice).unwrap();
                let r = view.unpack();

                v == r
            }
        };
    }

    mk_primitive_test!(array0, 0);
    mk_primitive_test!(array1, 1);
    mk_primitive_test!(array2, 2);
    mk_primitive_test!(array3, 3);
    mk_primitive_test!(array4, 4);
    mk_primitive_test!(array5, 5);
    mk_primitive_test!(array6, 6);
    mk_primitive_test!(array7, 7);
    mk_primitive_test!(array8, 8);
    mk_primitive_test!(array9, 9);
    mk_primitive_test!(array10, 10);
    mk_primitive_test!(array11, 11);
    mk_primitive_test!(array12, 12);
    mk_primitive_test!(array13, 13);
    mk_primitive_test!(array14, 14);
    mk_primitive_test!(array15, 15);
    mk_primitive_test!(array16, 16);
    mk_primitive_test!(array17, 17);
    mk_primitive_test!(array18, 18);
    mk_primitive_test!(array19, 19);
    mk_primitive_test!(array20, 20);
    mk_primitive_test!(array21, 21);
    mk_primitive_test!(array22, 22);
    mk_primitive_test!(array23, 23);
    mk_primitive_test!(array24, 24);
    mk_primitive_test!(array25, 25);
    mk_primitive_test!(array26, 26);
    mk_primitive_test!(array27, 27);
    mk_primitive_test!(array28, 28);
    mk_primitive_test!(array29, 29);
    mk_primitive_test!(array30, 30);
    mk_primitive_test!(array31, 31);
    mk_primitive_test!(array32, 32);
    mk_primitive_test!(array33, 33);
    mk_primitive_test!(array34, 34);
    mk_primitive_test!(array35, 35);
    mk_primitive_test!(array36, 36);
    mk_primitive_test!(array37, 37);
    mk_primitive_test!(array38, 38);
    mk_primitive_test!(array39, 39);
    mk_primitive_test!(array40, 40);
    mk_primitive_test!(array41, 41);
    mk_primitive_test!(array42, 42);
    mk_primitive_test!(array43, 43);
    mk_primitive_test!(array44, 44);
    mk_primitive_test!(array45, 45);
    mk_primitive_test!(array46, 46);
    mk_primitive_test!(array47, 47);
    mk_primitive_test!(array48, 48);
    mk_primitive_test!(array49, 49);
    mk_primitive_test!(array50, 50);
    mk_primitive_test!(array51, 51);
    mk_primitive_test!(array52, 52);
    mk_primitive_test!(array53, 53);
    mk_primitive_test!(array54, 54);
    mk_primitive_test!(array55, 55);
    mk_primitive_test!(array56, 56);
    mk_primitive_test!(array57, 57);
    mk_primitive_test!(array58, 58);
    mk_primitive_test!(array59, 59);
    mk_primitive_test!(array60, 60);
    mk_primitive_test!(array61, 61);
    mk_primitive_test!(array62, 62);
    mk_primitive_test!(array63, 63);
    mk_primitive_test!(array64, 64);
    mk_primitive_test!(array65, 65);
    mk_primitive_test!(array66, 66);
    mk_primitive_test!(array67, 67);
    mk_primitive_test!(array68, 68);
    mk_primitive_test!(array69, 69);
    mk_primitive_test!(array70, 70);
    mk_primitive_test!(array71, 71);
    mk_primitive_test!(array72, 72);
    mk_primitive_test!(array73, 73);
    mk_primitive_test!(array74, 74);
    mk_primitive_test!(array75, 75);
    mk_primitive_test!(array76, 76);
    mk_primitive_test!(array77, 77);
    mk_primitive_test!(array78, 78);
    mk_primitive_test!(array79, 79);
    mk_primitive_test!(array80, 80);
    mk_primitive_test!(array81, 81);
    mk_primitive_test!(array82, 82);
    mk_primitive_test!(array83, 83);
    mk_primitive_test!(array84, 84);
    mk_primitive_test!(array85, 85);
    mk_primitive_test!(array86, 86);
    mk_primitive_test!(array87, 87);
    mk_primitive_test!(array88, 88);
    mk_primitive_test!(array89, 89);
    mk_primitive_test!(array90, 90);
    mk_primitive_test!(array91, 91);
    mk_primitive_test!(array92, 92);
    mk_primitive_test!(array93, 93);
    mk_primitive_test!(array94, 94);
    mk_primitive_test!(array95, 95);
    mk_primitive_test!(array96, 96);
    mk_primitive_test!(array97, 97);
    mk_primitive_test!(array98, 98);
    mk_primitive_test!(array99, 99);
    mk_primitive_test!(array100, 100);
    mk_primitive_test!(array101, 101);
    mk_primitive_test!(array102, 102);
    mk_primitive_test!(array103, 103);
    mk_primitive_test!(array104, 104);
    mk_primitive_test!(array105, 105);
    mk_primitive_test!(array106, 106);
    mk_primitive_test!(array107, 107);
    mk_primitive_test!(array108, 108);
    mk_primitive_test!(array109, 109);
    mk_primitive_test!(array110, 110);
    mk_primitive_test!(array111, 111);
    mk_primitive_test!(array112, 112);
    mk_primitive_test!(array113, 113);
    mk_primitive_test!(array114, 114);
    mk_primitive_test!(array115, 115);
    mk_primitive_test!(array116, 116);
    mk_primitive_test!(array117, 117);
    mk_primitive_test!(array118, 118);
    mk_primitive_test!(array119, 119);
    mk_primitive_test!(array120, 120);
    mk_primitive_test!(array121, 121);
    mk_primitive_test!(array122, 122);
    mk_primitive_test!(array123, 123);
    mk_primitive_test!(array124, 124);
    mk_primitive_test!(array125, 125);
    mk_primitive_test!(array126, 126);
    mk_primitive_test!(array127, 127);
    mk_primitive_test!(array128, 128);
}
