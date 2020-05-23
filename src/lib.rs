//! This crate allows quoting strings for use in output. It works similarly to
//! [`str::escape_debug`], but the result is meant to be shown to users. Simply
//! call [`Quote::quote`] on an argument passed to [`println!`] or a similar
//! macro to quote it.
//!
//! One of the primary uses for this crate is displaying paths losslessly.
//! Since [`Path`] has no [`Display`] implementation, it is usually output by
//! calling [`Path::display`] or [`Path::to_string_lossy`] beforehand. However,
//! both of those methods are lossy; they replace all invalid characters with
//! [`REPLACEMENT_CHARACTER`]. This crate escapes those invalid characters
//! instead, allowing them to always be displayed correctly.
//!
//! Unprintable characters are also escaped, to give unambiguous output. All
//! code points are supported, but the Unicode Standard does not define which
//! are unprintable. So, a typical subset is used that may change between minor
//! versions. Guarantees are made in the next section.
//!
//! # Format
//!
//! The format used to represent strings is different from typical [`Debug`]
//! output, because it is designed to show most paths correctly on any
//! platform. In particular, backslashes (`\`) will never be escaped, since
//! Windows uses them as directory separators. They exist in almost every path
//! users provide on Windows.
//!
//! In their place, curly braces (`{` and `}`) are substituted, since they
//! appear less frequently. Thus, normal paths should not require any escaping
//! at all. The intention is to make the result easily readable on any system.
//!
//! These are some examples of the quoting format:
//!
//! ```
//! use uniquote::Quote;
//!
//! assert_eq!(r#""foo bar""#,      "foo bar".quote().to_string());
//! assert_eq!(r#""foo{~n}bar""#,   "foo\nbar".quote().to_string());
//! assert_eq!(r#""foo{~u7f}bar""#, "foo\x7Fbar".quote().to_string());
//! assert_eq!(r#""foo{"}bar""#,    "foo\"bar".quote().to_string());
//! ```
//!
//! The only ASCII characters escaped are `"`, `{`, `}`, and [control
//! characters]. Other characters are not guaranteed to be quoted in a specific
//! way but will generally only be escaped if unprintable.
//!
//! # Features
//!
//! These features are optional and can be enabled or disabled in a
//! "Cargo.toml" file. Nightly features are unstable, since they rely on
//! unstable Rust features.
//!
//! ### Default Features
//!
//! - **alloc** -
//!   Provides implementations of [`Quote`] for types that require allocation.
//!   This feature is enabled automatically when the **std** feature is
//!   enabled.
//!
//! - **std** -
//!   Provides implementations of [`Quote`] for types that require the standard
//!   library. When this feature is disabled, this crate can be used in
//!   `#![no_std]` environments.
//!
//! ### Nightly Features
//!
//! - **const_generics** -
//!   Provides an implementation of [`Quote`] for [`[u8; N]`][array].
//!
//! # Examples
//!
//! Print arguments passed on the command line:
//!
//! ```
//! use std::env;
//! # use std::io;
//!
//! use uniquote::Quote;
//!
//! # #[cfg(feature = "std")]
//! for (i, arg) in env::args_os().enumerate() {
//!     println!("arg #{} is {}", i, arg.quote());
//! }
//! ```
//!
//! Create a descriptive error message:
//!
//! ```
//! use std::error::Error;
//! use std::fmt;
//! use std::fmt::Display;
//! use std::fmt::Formatter;
//! use std::path::PathBuf;
//!
//! use uniquote::Quote;
//!
//! #[derive(Debug)]
//! struct FileNotFoundError(PathBuf);
//!
//! # #[cfg(feature = "std")]
//! # {
//! impl Display for FileNotFoundError {
//!     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//!         write!(f, "file not found at {}", self.0.quote())
//!     }
//! }
//!
//! impl Error for FileNotFoundError {}
//! # }
//! ```
//!
//! [array]: https://doc.rust-lang.org/std/primitive.array.html
//! [control characters]: https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_control
//! [`Debug`]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
//! [`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
//! [`Path`]: https://doc.rust-lang.org/std/path/struct.Path.html
//! [`Path::display`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.display
//! [`Path::to_string_lossy`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.to_string_lossy
//! [`println!`]: https://doc.rust-lang.org/std/macro.println.html
//! [`Quote`]: trait.Quote.html
//! [`Quote::quote`]: trait.Quote.html#method.quote
//! [`REPLACEMENT_CHARACTER`]: https://doc.rust-lang.org/std/char/constant.REPLACEMENT_CHARACTER.html
//! [`str::escape_debug`]: https://doc.rust-lang.org/std/primitive.str.html#method.escape_debug

#![cfg_attr(feature = "const_generics", allow(incomplete_features))]
#![doc(html_root_url = "https://docs.rs/uniquote/*")]
#![cfg_attr(feature = "const_generics", feature(const_generics))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unused_results)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

mod escape;

mod formatter;
pub use formatter::Error;
pub use formatter::Formatter;
pub use formatter::Result;

mod quote;
pub use quote::Quote;

const QUOTE: char = '"';

const START_ESCAPE: char = '{';

const END_ESCAPE: char = '}';
