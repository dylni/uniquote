use core::fmt;
use core::fmt::Display;
use core::mem;
use core::result;

/// The error type returned by [`Quote::escape`].
///
/// This type is used similarly to [`fmt::Error`] in the standard library.
///
/// [`fmt::Error`]: https://doc.rust-lang.org/std/fmt/struct.Error.html
/// [`Quote::escape`]: trait.Quote.html#method.escape
#[derive(Debug, Eq, PartialEq)]
pub struct Error(pub(super) fmt::Error);

impl Display for Error {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for Error {}

/// The type returned by [`Quote::escape`].
///
/// This type is used similarly to [`fmt::Result`] in the standard library.
///
/// [`fmt::Result`]: https://doc.rust-lang.org/std/fmt/type.Result.html
/// [`Quote::escape`]: trait.Quote.html#method.escape
pub type Result = result::Result<(), Error>;

/// The type passed between calls to [`Quote::escape`].
///
/// No methods are defined, to ensure that all strings are quoted uniformly. To
/// escape a string, pass this struct to the [`Quote::escape`] implementation
/// of another type.
///
/// Although this type is annotated with `#[repr(transparent)]`, the inner
/// representation is not stable. Transmuting between this type and any other
/// causes immediate [undefined behavior].
///
/// [`Quote::escape`]: trait.Quote.html#method.escape
/// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
#[repr(transparent)]
pub struct Formatter<'a>(pub(super) fmt::Formatter<'a>);

impl<'a> Formatter<'a> {
    pub(super) fn from_inner_mut<'b>(
        formatter: &'b mut fmt::Formatter<'a>,
    ) -> &'b mut Self {
        // SAFETY: This struct is designed to make this operation safe.
        #[allow(clippy::transmute_ptr_to_ptr)]
        unsafe {
            mem::transmute(formatter)
        }
    }
}
