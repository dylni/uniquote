use core::fmt;
use core::fmt::Display;
use core::fmt::Write as _;

use super::Formatter;
use super::Result;
use super::QUOTE;

#[derive(Debug)]
pub struct QuotedDisplay<T>(T);

impl<T> Display for QuotedDisplay<&T>
where
    T: Quote + ?Sized,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_char(QUOTE)?;

        self.0
            .escape(Formatter::from_inner_mut(formatter))
            .map_err(|x| x.0)?;

        formatter.write_char(QUOTE)
    }
}

/// The trait used to quote strings.
pub trait Quote {
    /// Escapes a string using the format described in the [the module-level
    /// documentation][format], without the surrounding quotes.
    ///
    /// This method is only used to provide new implementations of this trait.
    ///
    /// # Errors
    ///
    /// Similar to [`Display::fmt`], this method should fail if and only if the
    /// formatter returns an error. Since quoting is an infallible operation,
    /// these failures will only result from inability to write to the
    /// underlying stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use uniquote::Quote;
    ///
    /// struct Strings<'a>(&'a str, &'a str);
    ///
    /// impl Quote for Strings<'_> {
    ///     fn escape(&self, f: &mut uniquote::Formatter<'_>) -> uniquote::Result {
    ///         self.0.escape(f)?;
    ///         ','.escape(f)?;
    ///         self.1.escape(f)
    ///     }
    /// }
    ///
    /// assert_eq!(r#""foo,bar""#, Strings("foo", "bar").quote().to_string());
    /// ```
    ///
    /// [format]: super#format
    fn escape(&self, formatter: &mut Formatter<'_>) -> Result;

    /// Quotes a string using the format described in the [the module-level
    /// documentation][format].
    ///
    /// The returned struct will implement [`Display`]. It can be output using
    /// a formatting macro or converted to a string by calling
    /// [`ToString::to_string`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use uniquote::Quote;
    ///
    /// # #[cfg(feature = "std")]
    /// println!("{}", env::current_exe()?.quote());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [format]: super#format
    #[inline]
    #[must_use]
    fn quote(&self) -> QuotedDisplay<&Self> {
        QuotedDisplay(self)
    }
}

macro_rules! r#impl {
    ( $type:ty , $length_method:ident ) => {
        impl $crate::Quote for $type {
            #[inline]
            fn escape(
                &self,
                formatter: &mut $crate::Formatter<'_>,
            ) -> $crate::Result {
                use $crate::escape::Escape;
                use $crate::Error;

                Escape::escape(self, &mut formatter.0).map_err(Error)
            }
        }
    };
    ( $type:ty ) => {
        impl $crate::Quote for $type {
            #[inline]
            fn escape(
                &self,
                formatter: &mut $crate::Formatter<'_>,
            ) -> $crate::Result {
                (**self).escape(formatter)
            }
        }
    };
}

r#impl!([u8], len);
r#impl!(char, len_utf8);
r#impl!(str, len);

#[cfg(any(feature = "const_generics", feature = "min_const_generics"))]
#[cfg_attr(uniquote_docs_rs, doc(cfg(feature = "min_const_generics")))]
impl<const N: usize> Quote for [u8; N] {
    #[inline]
    fn escape(&self, formatter: &mut Formatter<'_>) -> Result {
        self[..].escape(formatter)
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use alloc::string::String;
    use alloc::vec::Vec;

    r#impl!(String);
    r#impl!(Vec<u8>);
}

#[cfg(feature = "std")]
mod std {
    use std::ffi::CStr;
    use std::ffi::CString;
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    use super::Formatter;
    use super::Quote;
    use super::Result;

    r#impl!(OsStr, len);

    r#impl!(CString);
    r#impl!(OsString);
    r#impl!(PathBuf);

    impl Quote for CStr {
        #[inline]
        fn escape(&self, formatter: &mut Formatter<'_>) -> Result {
            self.to_bytes().escape(formatter)
        }
    }

    impl Quote for Path {
        #[inline]
        fn escape(&self, formatter: &mut Formatter<'_>) -> Result {
            self.as_os_str().escape(formatter)
        }
    }
}
