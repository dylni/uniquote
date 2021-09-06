use core::fmt;
use core::fmt::Display;
use core::fmt::Write as _;

use super::Error;
use super::Formatter;
use super::Result;
use super::QUOTE;

#[derive(Debug)]
pub struct QuotedDisplay<T>(T);

impl<T> Display for QuotedDisplay<&T>
where
    T: Quote + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(QUOTE)?;

        self.0.escape(Formatter::new(f)).map_err(|x| x.0)?;

        f.write_char(QUOTE)
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
    fn escape(&self, f: &mut Formatter<'_>) -> Result;

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
    ( $($type:ty),+ ) => {
        $(
            impl Quote for $type {
                #[inline]
                fn escape(&self, f: &mut Formatter<'_>) -> $crate::Result {
                    use super::escape::Escape;

                    Escape::escape(self, &mut f.0).map_err(Error)
                }
            }
        )+
    };
}
r#impl!(char, str, [u8]);

#[cfg(feature = "min_const_generics")]
#[cfg_attr(uniquote_docs_rs, doc(cfg(feature = "min_const_generics")))]
impl<const N: usize> Quote for [u8; N] {
    #[inline]
    fn escape(&self, f: &mut Formatter<'_>) -> Result {
        self[..].escape(f)
    }
}

#[cfg_attr(not(feature = "std"), allow(unused_macros))]
macro_rules! impl_with_deref {
    ( $($type:ty),+ ) => {
        $(
            impl $crate::Quote for $type {
                #[inline]
                fn escape(
                    &self,
                    f: &mut $crate::Formatter<'_>
                ) -> $crate::Result {
                    (**self).escape(f)
                }
            }
        )+
    };
}

#[cfg(feature = "alloc")]
mod alloc {
    use alloc::string::String;
    use alloc::vec::Vec;

    impl_with_deref!(String, Vec<u8>);
}

#[cfg(feature = "std")]
mod std {
    use std::ffi::CStr;
    use std::ffi::CString;
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    use crate::Formatter;
    use crate::Result;

    use super::Quote;

    impl Quote for CStr {
        #[inline]
        fn escape(&self, f: &mut Formatter<'_>) -> Result {
            self.to_bytes().escape(f)
        }
    }

    impl Quote for OsStr {
        #[inline]
        fn escape(&self, f: &mut Formatter<'_>) -> Result {
            #[cfg(not(windows))]
            {
                #[cfg(any(
                    target_os = "hermit",
                    target_os = "redox",
                    unix,
                ))]
                use std::os::unix as os;
                #[cfg(target_os = "wasi")]
                use std::os::wasi as os;

                use os::ffi::OsStrExt;

                self.as_bytes().escape(f)
            }
            #[cfg(windows)]
            {
                use std::os::windows::ffi::OsStrExt;

                f.escape_utf16(self.encode_wide())
            }
        }
    }

    impl Quote for Path {
        #[inline]
        fn escape(&self, f: &mut Formatter<'_>) -> Result {
            self.as_os_str().escape(f)
        }
    }

    impl_with_deref!(CString, OsString, PathBuf);
}
