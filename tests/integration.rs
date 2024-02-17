use std::char::REPLACEMENT_CHARACTER;
use std::fmt::Display;

use uniquote::Quote;

fn test<T>(expected: &str, result: T)
where
    T: Display,
{
    assert_eq!(expected, result.to_string());
}

fn test_unchanged<T>(string: &T)
where
    T: Display + Quote + ?Sized,
{
    test(&format!(r#""{}""#, string), string.quote());
}

#[test]
fn test_bytes() {
    test(
        concat!(
            r#"""#,
            r#"{~u0}{~u1}{~u2}{~u3}{~u4}{~u5}{~u6}{~u7}"#,
            r#"{~u8}{~t}{~n}{~ub}{~uc}{~r}{~ue}{~uf}"#,
            r#"{~u10}{~u11}{~u12}{~u13}{~u14}{~u15}{~u16}{~u17}"#,
            r#"{~u18}{~u19}{~u1a}{~u1b}{~u1c}{~u1d}{~u1e}{~u1f}"#,
            r#" !{"}#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNO"#,
            r#"PQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{{|}}~{~u7f}"#,
            r#"{~u80}{~u81}{~u82}{~u83}{~u84}{~u85}{~u86}{~u87}"#,
            r#"{~u88}{~u89}{~u8a}{~u8b}{~u8c}{~u8d}{~u8e}{~u8f}"#,
            r#"{~u90}{~u91}{~u92}{~u93}{~u94}{~u95}{~u96}{~u97}"#,
            r#"{~u98}{~u99}{~u9a}{~u9b}{~u9c}{~u9d}{~u9e}{~u9f}"#,
            r#"{~ua0}¡¢£¤¥¦§¨©ª«¬{~uad}®¯°±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏ"#,
            r#"ÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ"#,
            r#"""#,
        ),
        (0..=u8::MAX).collect::<Vec<_>>().quote(),
    );
}

#[test]
fn test_strings() {
    test_unchanged("abc");
    test_unchanged("a c");
    test_unchanged("éèê");

    test(r#""{~r}{~n}{~t}""#, "\r\n\t".quote());
    test(r#""'{"}\{{}}""#, "'\"\\{}".quote());
    test(r#""{~u7f}ÿ""#, "\x7F\u{FF}".quote());
    test(r#""Ā{~uffff}""#, "\u{100}\u{FFFF}".quote());
    test(r#""က{~u10ffff}""#, "\u{1000}\u{10FFFF}".quote());
    test(r#""ab{~u200b}""#, "ab\u{200B}".quote());
    test(r#""{~u10d4ea}{~r}""#, "\u{10D4EA}\r".quote());
}

#[test]
fn test_chinese() {
    test_unchanged("系统找不到指定的文件");
    test_unchanged("文件不存在");
}

#[test]
fn test_replacement_character() {
    test_unchanged(&REPLACEMENT_CHARACTER);
}

#[cfg(feature = "std")]
#[test]
fn test_os_string() {
    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        test(
            r#""fo{~u80}o""#,
            OsStr::from_bytes(b"\x66\x6F\x80\x6F").quote(),
        );
    }
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        test(
            r#""fo{~ud800}o""#,
            OsString::from_wide(&[0x66, 0x6F, 0xD800, 0x6F]).quote(),
        );
    }
}
