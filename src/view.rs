use std::{any, fmt, hash, marker};

use anyhow::Result;

use crate::Packed;

/// view of a slice in memory as a packed structure of type `T`
pub struct View<'a, T: ?Sized> {
    slice: &'a [u8],
    marker: marker::PhantomData<T>,
}

impl<'a, T> View<'a, T>
where
    T: Packed,
{
    /// create the [`View`] from the slice
    /// without performing any checks
    #[inline]
    pub(crate) fn new(slice: &'a [u8]) -> Self {
        Self {
            slice,
            marker: marker::PhantomData,
        }
    }

    /// reconstruct the object `T` from the given [`View<'_, T>`]
    ///
    /// this function will involve some hoops and loops and may
    /// involve some heap allocation.
    #[inline]
    pub fn into(self) -> T {
        T::unchecked_read_from_slice(self)
    }

    /// create a [`View`] from the given slice.
    ///
    /// this function will perform all the necessary checks
    /// in order to make sure there's no invalid data.
    ///
    /// Once the [`View`] is created, it is possible to use it
    /// safely across the board.
    pub fn try_from_slice(slice: &'a [u8]) -> Result<Self> {
        anyhow::ensure!(
            slice.len() == T::SIZE,
            "invalid length for {ty}, expecting {expected} bytes",
            ty = ::std::any::type_name::<T>(),
            expected = T::SIZE
        );

        T::check(slice)?;
        Ok(View::new(slice))
    }
}

impl<'a, T> Clone for View<'a, T> {
    fn clone(&self) -> Self {
        Self {
            slice: self.slice,
            marker: self.marker,
        }
    }
}

impl<'a, T> Copy for View<'a, T> {}

impl<'a, T> AsRef<[u8]> for View<'a, T> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a, 'b, T, U> PartialEq<View<'b, U>> for View<'a, T> {
    fn eq(&self, other: &View<'b, U>) -> bool {
        self.slice.eq(other.slice)
    }
}

impl<'a, T> Eq for View<'a, T> {}

impl<'a, 'b, T, U> PartialOrd<View<'b, U>> for View<'a, T> {
    fn partial_cmp(&self, other: &View<'b, U>) -> Option<std::cmp::Ordering> {
        self.slice.partial_cmp(other.slice)
    }
}

impl<'a, T> Ord for View<'a, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.slice.cmp(other.slice)
    }
}

impl<'a, T> hash::Hash for View<'a, T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.slice.hash(state);
        self.marker.hash(state);
    }
}

impl<'a, T> fmt::Debug for View<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ty = any::type_name::<T>();
        f.debug_struct(&format!("View<'a, {}>", ty))
            .field("slice", &self.slice)
            .field("marker", &self.marker)
            .finish()
    }
}
