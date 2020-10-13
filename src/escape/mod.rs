use core::char;
use core::convert::TryFrom;
use core::fmt;
use core::fmt::Formatter;
use core::fmt::Write as _;
use core::str;

use super::END_ESCAPE;
use super::QUOTE;
use super::START_ESCAPE;

mod code_point;
use code_point::CodePoint;

mod tables;
use tables::UNPRINTABLE;

fn table_contains(table: &[(u32, u32)], code_point: CodePoint) -> bool {
    let code_point = code_point.into();
    match table.binary_search_by_key(&code_point, |&(x, _)| x) {
        Ok(_) => true,
        Err(index) => index
            .checked_sub(1)
            .filter(|&x| code_point <= table[x].1)
            .is_some(),
    }
}

fn is_printable(ch: char) -> bool {
    // ASCII is very common, so it should be optimized.
    match ch {
        ' '..='~' => true,
        _ if ch.is_ascii() => false,
        _ => !table_contains(UNPRINTABLE, ch.into()),
    }
}

enum EscapedCodePoint {
    Hex(CodePoint),
    Literal(char),
    Quote(),
    Repeated(char),
    Sequence(&'static str),
}

impl EscapedCodePoint {
    fn format(self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(ch) => return formatter.write_char(ch),
            Self::Repeated(ch) => {
                for _ in 0..2 {
                    formatter.write_char(ch)?;
                }
                return Ok(());
            }
            _ => {}
        }

        formatter.write_char(START_ESCAPE)?;

        if let Self::Quote() = self {
            formatter.write_char(QUOTE)?;
        } else {
            formatter.write_char('~')?;

            match self {
                Self::Hex(code_point) => {
                    write!(formatter, "u{:x}", u32::from(code_point))?;
                }
                Self::Sequence(sequence) => formatter.write_str(sequence)?,
                _ => unreachable!(),
            }
        }

        formatter.write_char(END_ESCAPE)
    }
}

impl From<u8> for EscapedCodePoint {
    fn from(value: u8) -> Self {
        char::from(value).into()
    }
}

impl From<char> for EscapedCodePoint {
    fn from(value: char) -> Self {
        match value {
            '\t' => Self::Sequence("t"),
            '\n' => Self::Sequence("n"),
            '\r' => Self::Sequence("r"),

            QUOTE => Self::Quote(),
            END_ESCAPE | START_ESCAPE => Self::Repeated(value),

            _ if is_printable(value) => Self::Literal(value),
            _ => Self::Hex(value.into()),
        }
    }
}

impl From<CodePoint> for EscapedCodePoint {
    fn from(value: CodePoint) -> Self {
        match char::try_from(value) {
            Ok(ch) => ch.into(),
            // [value] is now known to be a surrogate, so it is unprintable.
            Err(_) => Self::Hex(value),
        }
    }
}

pub(super) trait Escape {
    fn escape(&self, formatter: &mut Formatter<'_>) -> fmt::Result;
}

impl Escape for [u8] {
    fn escape(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let mut string = self;
        while !string.is_empty() {
            let mut invalid = None;
            let valid = match str::from_utf8(string) {
                Ok(string) => string,
                Err(error) => {
                    let (valid, string) = string.split_at(error.valid_up_to());

                    let invalid_length =
                        error.error_len().unwrap_or_else(|| string.len());
                    invalid = Some(&string[..invalid_length]);

                    // SAFETY: This slice was validated to be UTF-8.
                    unsafe { str::from_utf8_unchecked(valid) }
                }
            };

            valid.escape(formatter)?;
            string = &string[valid.len()..];

            if let Some(invalid) = invalid {
                for &byte in invalid {
                    EscapedCodePoint::from(byte).format(formatter)?;
                }
                string = &string[invalid.len()..];
            }
        }
        Ok(())
    }
}

impl Escape for char {
    fn escape(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.encode_utf8(&mut [0; 4]).escape(formatter)
    }
}

impl Escape for str {
    fn escape(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        // [str] can be written more efficiently than multiple [char] values,
        // since it is already encoded as UTF-8 bytes. The [Debug]
        // implementation for [str] uses the same optimization.
        let mut escaped_index = 0;
        macro_rules! push_literal {
            ( $index:expr ) => {
                let index = $index;
                if index != escaped_index {
                    formatter.write_str(&self[escaped_index..index])?;
                }
            };
        }

        let mut escaped = false;
        for (i, ch) in self.char_indices() {
            if escaped {
                escaped_index = i;
            }

            let code_point = ch.into();
            escaped = if let EscapedCodePoint::Literal(_) = code_point {
                false
            } else {
                push_literal!(i);
                code_point.format(formatter)?;
                true
            };
        }
        if !escaped {
            push_literal!(self.len());
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
mod std {
    use std::ffi::OsStr;
    use std::fmt;
    use std::fmt::Formatter;

    use super::Escape;

    impl Escape for OsStr {
        fn escape(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
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

                self.as_bytes().escape(formatter)
            }
            #[cfg(windows)]
            {
                use std::char;
                use std::os::windows::ffi::OsStrExt;

                use super::CodePoint;
                use super::EscapedCodePoint;

                for ch in char::decode_utf16(self.encode_wide()) {
                    ch.map(EscapedCodePoint::from)
                        .map_err(CodePoint::from)
                        .unwrap_or_else(Into::into)
                        .format(formatter)?;
                }
                Ok(())
            }
        }
    }
}
