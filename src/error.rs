use std::{any::type_name, borrow::Cow, error, fmt};
use thiserror::Error;

/// helper method to create an [`Error`] is the assumption
/// fails.
///
/// The type associated is helping to create useful message
/// as to what was expecting on what.
///
/// # Example
///
/// ```
/// use packtool::{Error, ensure};
///
/// # fn test() -> Result<(), Error> {
/// ensure!(
///   u8,
///   0 == 1,
///   "math needs to hold here",
/// );
/// # Ok(()) }
/// # let error = test().unwrap_err();
///
/// assert_eq!(
///   error.to_string(),
///   "Assumption `0 == 1` failed for u8: math needs to hold here",
/// )
/// ```
#[macro_export]
macro_rules! ensure {
    ($Type:ty, $assumption:expr, $fmt:expr, $($arg:tt)*) => {
        if !$assumption {
            return ::std::result::Result::Err(
                $crate::Error::Assumption {
                    ty: ::std::any::type_name::<$Type>(),
                    assumption: ::std::stringify!($assumption),
                    message: ::std::format!($fmt, $($arg)*),
                }
            );
        }
    };
}

/// error associated to unpacking or creating [`View`] of [`Packed`] types.
#[derive(Debug, Error)]
pub enum Error {
    /// error that is returned if an invalid size is detected
    ///
    /// this will happen when calling [`View::try_from_slice`]
    /// for example.
    #[error("Invalid size for {ty}: expected {expected} bytes but received {received} bytes")]
    InvalidSize {
        /// the stringified type associated to this error
        ty: &'static str,
        /// the size of the slice that does not match the expected size
        received: usize,
        /// the expected size
        expected: usize,
    },

    /// error that is created when an assumption is false
    ///
    /// this is used by [`ensure`] macro
    #[error("Assumption `{assumption}` failed for {ty}: {message}")]
    Assumption {
        /// the stringified type associated to this error
        ty: &'static str,
        /// the stringified assumption
        assumption: &'static str,
        /// a custom message associated to the assumption
        message: String,
    },

    /// error when trying to check an Enumeration against a slice
    ///
    #[error(
        "Invalid discriminant for {ty}, received {found:?} while expecting one of: [ {options}]"
    )]
    InvalidDiscriminant {
        /// the stringified type associated to this error
        ty: &'static str,
        /// the available values (if any)
        options: &'static str,
        /// the actually found value
        found: Box<dyn fmt::Debug + Send + Sync>,
    },

    #[error("Field {field} of {ty} is not valid")]
    InvalidField {
        /// the stringified type associated to this error
        ty: &'static str,
        field: &'static str,
    },

    #[error("Tuple entry {ty}.{index} is not valid")]
    InvalidTuple {
        /// the stringified type associated to this error
        ty: &'static str,
        index: usize,
    },

    #[error("{0}")]
    Message(Cow<'static, str>),

    #[error(transparent)]
    Custom(#[from] Box<dyn error::Error + Send + Sync>),

    #[error("{ty}: {error}")]
    Context {
        /// the stringified type associated to this error
        ty: &'static str,
        error: Box<Self>,
        /// the root cause of the error
        ///
        /// recursively call the underlying error to find
        /// more granulated details of a given error
        #[source]
        cause: Box<Self>,
    },
}

impl From<&'static str> for Error {
    fn from(error: &'static str) -> Self {
        Error::Message(Cow::Borrowed(error))
    }
}

impl Error {
    /// convenient function to create an [`Error::InvalidSize`]
    /// with the type_name of `T` being captured
    #[inline]
    pub fn invalid_size<T: ?Sized>(received: usize, expected: usize) -> Self {
        Self::InvalidSize {
            ty: type_name::<T>(),
            received,
            expected,
        }
    }

    #[inline]
    pub fn invalid_discriminant<T, V>(found: V, options: &'static str) -> Self
    where
        T: ?Sized,
        V: fmt::Debug + Send + Sync + 'static,
    {
        Self::InvalidDiscriminant {
            ty: type_name::<T>(),
            found: Box::new(found),
            options,
        }
    }

    #[inline]
    pub fn invalid_field<T>(field: &'static str) -> Self
    where
        T: ?Sized,
    {
        Self::InvalidField {
            ty: type_name::<T>(),
            field,
        }
    }

    #[inline]
    pub fn invalid_tuple<T>(index: usize) -> Self
    where
        T: ?Sized,
    {
        Self::InvalidTuple {
            ty: type_name::<T>(),
            index,
        }
    }
}

mod private {
    pub trait Sealed {}

    impl<T, E> Sealed for Result<T, E> where E: ::std::error::Error + 'static {}
}

pub trait Context<T, E>: private::Sealed {
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: Into<Error>;
}

impl<T, E> Context<T, E> for Result<T, E>
where
    E: error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: Into<Error>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(cause) => Err(Error::Context {
                ty: type_name::<T>(),
                error: Box::new(context.into()),
                cause: Box::new(Error::Custom(Box::new(cause))),
            }),
        }
    }
}

/*
pub trait Context {
    #[inline]
    fn context<E>(result: Self, error: Error) -> Self
    where
        E: error::Error + 'static;
}

impl<T, E> Context for Result<T, E>
where
    E: error::Error + 'static,
{
    #[inline]
    pub fn context<T, E>(result: Result<T, E>, error: impl Into<Self>) -> Result<T, Self>
    where
        E: error::Error + 'static,
    {
        result.map_err(|cause| Self::Context {
            ty: type_name::<T>(),
            error: Box::new(error.into()),
            cause: Box::new(Self::Custom(Box::new(cause))),
        })
    }

    #[inline]
    pub fn with_context<F>(result: Self, f: F) -> Result<T, Error>
    where
        F: Fn() -> Self,
    {
        result.map_err(|cause| {
            let error = f();
            Self::Context {
                ty: type_name::<T>(),
                error: Box::new(error),
                cause: Box::new(cause),
            }
        })
    }
}
*/
