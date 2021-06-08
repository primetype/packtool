use crate::{Packed, View};
use std::{any, borrow::Borrow, fmt, hash, marker};

/// a owned slice of memory containing the [`Packed`]
pub struct Packet<T> {
    boxed: Box<[u8]>,
    marker: marker::PhantomData<T>,
}

impl<T> Packet<T>
where
    T: Packed,
{
    #[inline]
    pub(crate) fn new(boxed: Box<[u8]>) -> Self {
        Self {
            boxed,
            marker: marker::PhantomData,
        }
    }

    /// get a [`View`] of the [`Packet`].
    ///
    #[inline]
    pub fn view(&self) -> View<'_, T> {
        View::new(self.boxed.as_ref())
    }

    /// pack any object that implements [`Packed`] into an owned
    /// slice of memory: [`Packet`].
    pub fn pack<P>(packed: P) -> Self
    where
        P: AsRef<T>,
    {
        let mut boxed = vec![0; <T as Packed>::SIZE];

        packed.as_ref().unchecked_write_to_slice(&mut boxed);

        Self::new(boxed.into_boxed_slice())
    }

    /// reconstruct the object `T` from the given [`Packet`]
    ///
    /// this function will involve some hoops and loops and may
    /// involve some heap allocation.
    #[inline]
    #[must_use = "this will clone data from the slice, it is often expensive"]
    pub fn unpack(&self) -> T {
        self.view().unpack()
    }
}

impl<T> Borrow<[u8]> for Packet<T> {
    fn borrow(&self) -> &[u8] {
        self.boxed.borrow()
    }
}

impl<T> Clone for Packet<T> {
    fn clone(&self) -> Self {
        Self {
            boxed: self.boxed.clone(),
            marker: self.marker,
        }
    }
}

impl<T> AsRef<[u8]> for Packet<T> {
    fn as_ref(&self) -> &[u8] {
        self.boxed.as_ref()
    }
}

impl<T, U> PartialEq<Packet<U>> for Packet<T> {
    fn eq(&self, other: &Packet<U>) -> bool {
        self.boxed.eq(&other.boxed)
    }
}

impl<T> Eq for Packet<T> {}

impl<T, U> PartialOrd<Packet<U>> for Packet<T> {
    fn partial_cmp(&self, other: &Packet<U>) -> Option<std::cmp::Ordering> {
        self.boxed.partial_cmp(&other.boxed)
    }
}

impl<T> Ord for Packet<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.boxed.cmp(&other.boxed)
    }
}

impl<T> hash::Hash for Packet<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.boxed.hash(state);
        self.marker.hash(state);
    }
}

impl<T> fmt::Debug for Packet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ty = any::type_name::<T>();
        f.debug_struct(&format!("Packet<{}>", ty))
            .field("slice", &self.boxed)
            .field("marker", &self.marker)
            .finish()
    }
}
