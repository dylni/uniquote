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
            r#"{~x80}{~x81}{~x82}{~x83}{~x84}{~x85}{~x86}{~x87}"#,
            r#"{~x88}{~x89}{~x8a}{~x8b}{~x8c}{~x8d}{~x8e}{~x8f}"#,
            r#"{~x90}{~x91}{~x92}{~x93}{~x94}{~x95}{~x96}{~x97}"#,
            r#"{~x98}{~x99}{~x9a}{~x9b}{~x9c}{~x9d}{~x9e}{~x9f}"#,
            r#"{~xa0}¡¢£¤¥¦§¨©ª«¬{~xad}®¯°±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏ"#,
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

#[cfg(feature = "os_str_bytes")]
#[test]
fn test_os_string() {
    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        test(
            r#""fo{~x80}o""#,
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
