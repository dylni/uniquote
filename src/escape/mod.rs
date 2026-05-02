use core::char;
use core::fmt;
use core::fmt::Formatter;
use core::fmt::Write as _;

#[cfg(feature = "os_str_bytes")]
use os_str_bytes::OsUnit;

use super::END_ESCAPE;
use super::QUOTE;
use super::START_ESCAPE;

mod code_point;
use code_point::CodePoint;

mod tables;
use tables::UNPRINTABLE;

fn table_contains(table: &[(u32, u32)], code_point: CodePoint) -> bool {
    let code_point = code_point.into();
    table
        .binary_search_by_key(&code_point, |&(x, _)| x)
        .map(|_| true)
        .unwrap_or_else(|index| {
            index
                .checked_sub(1)
                .is_some_and(|x| code_point <= table[x].1)
        })
}

fn is_printable(ch: char) -> bool {
    // ASCII is very common, so it should be optimized.
    (' '..='~').contains(&ch)
        || (!ch.is_ascii() && !table_contains(UNPRINTABLE, ch.into()))
}

enum EscapedCodePoint {
    Hex { value: u64, byte: bool },
    Literal { ch: char, escape: bool },
    Quote(),
    Sequence(&'static str),
}

impl EscapedCodePoint {
    fn new(ch: char, byte: bool) -> Self {
        match ch {
            '\t' => Self::Sequence("t"),
            '\n' => Self::Sequence("n"),
            '\r' => Self::Sequence("r"),

            QUOTE => Self::Quote(),
            END_ESCAPE | START_ESCAPE => Self::Literal { ch, escape: true },

            _ if is_printable(ch) => Self::Literal { ch, escape: false },
            _ => Self::Hex {
                value: ch.into(),
                byte,
            },
        }
    }

    fn format(self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Self::Literal { ch, escape } = self {
            for _ in 0..=(escape.into()) {
                f.write_char(ch)?;
            }
            return Ok(());
        }

        f.write_char(START_ESCAPE)?;

        if matches!(self, Self::Quote()) {
            f.write_char(QUOTE)?;
        } else {
            f.write_char('~')?;

            match self {
                Self::Hex { value, byte } => {
                    write!(f, "{}{:x}", if byte { 'x' } else { 'u' }, value)?;
                }
                Self::Sequence(sequence) => f.write_str(sequence)?,
                _ => unreachable!(),
            }
        }

        f.write_char(END_ESCAPE)
    }
}

impl From<u8> for EscapedCodePoint {
    fn from(value: u8) -> Self {
        Self::new(value.into(), true)
    }
}

impl From<char> for EscapedCodePoint {
    fn from(value: char) -> Self {
        Self::new(value, false)
    }
}

impl From<CodePoint> for EscapedCodePoint {
    fn from(value: CodePoint) -> Self {
        // Upon error, [value] is known to be a surrogate, so it is
        // unprintable.
        char::try_from(value)
            .map(Into::into)
            .unwrap_or_else(|_| Self::Hex {
                value: value.into(),
                byte: false,
            })
    }
}

#[cfg(feature = "os_str_bytes")]
impl From<OsUnit> for EscapedCodePoint {
    fn from(value: OsUnit) -> Self {
        let value = value.to_u64();
        Self::Hex {
            value,
            byte: value <= 0xFF,
        }
    }
}

pub(super) trait Escape {
    fn escape(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl Escape for char {
    fn escape(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.encode_utf8(&mut [0; 4]).escape(f)
    }
}

#[cfg(feature = "os_str_bytes")]
impl Escape for OsUnit {
    fn escape(&self, f: &mut Formatter<'_>) -> fmt::Result {
        EscapedCodePoint::from(*self).format(f)
    }
}

impl Escape for str {
    fn escape(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // [str] can be written more efficiently than multiple [char] values,
        // since it is already encoded as UTF-8 bytes. The [Debug]
        // implementation for [str] uses the same optimization.
        let mut escaped_index = 0;
        macro_rules! push_literal {
            ( $index:expr ) => {
                let index = $index;
                if index != escaped_index {
                    f.write_str(&self[escaped_index..index])?;
                }
            };
        }

        let mut escaped = false;
        for (i, ch) in self.char_indices() {
            if escaped {
                escaped_index = i;
            }

            let code_point = ch.into();
            escaped = !matches!(
                code_point,
                EscapedCodePoint::Literal { escape: false, .. },
            );
            if escaped {
                push_literal!(i);
                code_point.format(f)?;
            }
        }
        if !escaped {
            push_literal!(self.len());
        }

        Ok(())
    }
}

impl Escape for [u8] {
    fn escape(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for chunk in self.utf8_chunks() {
            chunk.valid().escape(f)?;

            for &byte in chunk.invalid() {
                EscapedCodePoint::from(byte).format(f)?;
            }
        }
        Ok(())
    }
}

pub(super) fn utf16<I>(iter: I, f: &mut Formatter<'_>) -> fmt::Result
where
    I: IntoIterator<Item = u16>,
{
    for ch in char::decode_utf16(iter) {
        ch.map(EscapedCodePoint::from)
            .unwrap_or_else(|x| CodePoint::from(x).into())
            .format(f)?;
    }
    Ok(())
}
