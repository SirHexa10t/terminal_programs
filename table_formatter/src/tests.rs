use crate::{format_table, strip_ansi, is_numeric_or_neutral, DEFAULT_SEPARATOR};
use test_case::test_case;

// numerical column needs to align right
// extra excessive spaces need to be trimmed off
// tabs need to be deleted (including '	')
// 1-spaced words need to stay together
// colored word needs to avoid padding the whole column with invisible
const SAMPLE_INPUT: &[&str] = &[
    "num  word\ta  long_word   b",
    "   1  one   ",
    "2  very long spaced  a  c  d  e	f\tg  h  i  j  k",
    "5k  a  b  c  \u{1b}[31mcolored\u{1b}[0m  d",
];

const SAMPLE_OUTPUT: &[&str] = &[
    "num  word              a  long_word  b                           ",
    "  1  one                                                         ",
    "  2  very long spaced  a  c          d        e  f  g  h  i  j  k",
    " 5k  a                 b  c          \u{1b}[31mcolored\u{1b}[0m  d                  ",
];

const SMTOUHOU_DATA: &[&str] = &[
    "  #      Name            Lv.   HP      MP      ATK   DEF",
    "1      Reimu            40      193   211   63      82   ",
    "2      Marisa         28      125   166   46      57   ",
    "3      Shingyoku      89      620   505   202   182",
    "4      Yugenmagan   87      628   576   176   189",
    "5      Elis            78      495   448   215   145",
    "6      Sariel         90      690   630   164   217",
    "7      Mima            74      494   472   146   166",
];

const SMTOUHOU_DATA_ORGANIZED: &[&str] = &[
    "#  Name        Lv.   HP   MP  ATK  DEF",
    "1  Reimu        40  193  211   63   82",
    "2  Marisa       28  125  166   46   57",
    "3  Shingyoku    89  620  505  202  182",
    "4  Yugenmagan   87  628  576  176  189",
    "5  Elis         78  495  448  215  145",
    "6  Sariel       90  690  630  164  217",
    "7  Mima         74  494  472  146  166",
];

const LONG_TABLE: &[&str] = &[
    "A  B",
    " 1      X",
    "2    X",
    "3    X",
    "4     X",
    "5    X",
    "6  X",
    "7    X",
    "7  X",
    "8        X",
    "8      X",
];

const LONG_TABLE_ORGANIZED: &[&str] = &[
    "A  B",
    "1  X",
    "2  X",
    "3  X",
    "4  X",
    "5  X",
    "6  X",
    "7  X",
    "7  X",
    "8  X",
    "8  X",
];

const WIDE_TABLE: &[&str] = &[
    "A  B       c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v       w  x  y  z",
    "A  B  c  d  e  f  g  h  i  j  k  l  m  n       o  p  q  r  s  t  u  v  w  x  y  z",
    "A  B  c  d  e  f  g  h  i  j       k  l  m  n  o  p  q       r  s  t  u  v  w  x  y  z",
    "A       B  c  d       e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u       v  w  x  y  z",
];

const WIDE_TABLE_ORGANIZED: &[&str] = &[
    "A  B  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z",
    "A  B  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z",
    "A  B  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z",
    "A  B  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z",
];

const VARYING_LENGTH_TABLE: &[&str] = &[
    "A            1  c  d  e       f  g ",
    "B       2",
    "C  4  c  d  e  f  g ",
    "D  3       c",
    "E       5  c  d  e  f  g ",
    "F  6",
    "G  7       c  d       e  f       g ",
];

const VARYING_LENGTH_TABLE_ORGANIZED: &[&str] = &[
    "A  1  c  d  e  f  g",
    "B  2               ",
    "C  4  c  d  e  f  g",
    "D  3  c            ",
    "E  5  c  d  e  f  g",
    "F  6               ",
    "G  7  c  d  e  f  g",
];

const MISSING_LINES: &[&str] = &[
    "A  B",
    " 1      X",
    "2    X",
    "3    X",
    "",
    "5    X",
    "",
    "7    X",
    "7  X",
    "8        X",
    "8      X",
];

const MISSING_LINES_ORGANIZED: &[&str] = &[
    "A  B",
    "1  X",
    "2  X",
    "3  X",
    "    ",
    "5  X",
    "    ",
    "7  X",
    "7  X",
    "8  X",
    "8  X",
];

const SPECIAL_CHARS: &[&str] = &[
    "A  B",
    "1  x",
    "🌎     X",
    "🇺🇸     X",
    "3  X",
];

const SPECIAL_CHARS_ORGANIZED: &[&str] = &[
    "A   B",
    "1   x",
    "🌎   X",
    "🇺🇸  X",
    "3   X",
];

fn to_strings(arr: &[&str]) -> Vec<String> {
    arr.iter().map(|s| s.to_string()).collect()
}

#[test_case(SAMPLE_INPUT, SAMPLE_OUTPUT)]
#[test_case(SMTOUHOU_DATA, SMTOUHOU_DATA_ORGANIZED)]
#[test_case(LONG_TABLE, LONG_TABLE_ORGANIZED)]
#[test_case(WIDE_TABLE, WIDE_TABLE_ORGANIZED)]
#[test_case(VARYING_LENGTH_TABLE, VARYING_LENGTH_TABLE_ORGANIZED)]
#[test_case(MISSING_LINES, MISSING_LINES_ORGANIZED)]
#[test_case(SPECIAL_CHARS, SPECIAL_CHARS_ORGANIZED)]
fn test_directly(input: &[&str], expected: &[&str]) {    
    assert_eq!(format_table(&to_strings(input), DEFAULT_SEPARATOR), to_strings(expected));
}

#[test_case(SAMPLE_OUTPUT)]
#[test_case(SMTOUHOU_DATA_ORGANIZED)]
#[test_case(LONG_TABLE_ORGANIZED)]
#[test_case(WIDE_TABLE_ORGANIZED)]
#[test_case(VARYING_LENGTH_TABLE_ORGANIZED)]
#[test_case(MISSING_LINES_ORGANIZED)]
#[test_case(SPECIAL_CHARS_ORGANIZED)]
fn test_solution_unchanging(input: &[&str]) {
    assert_eq!(format_table(&to_strings(input), DEFAULT_SEPARATOR), to_strings(input));
}

fn run_with_file(file: &str) -> String {
    use assert_cmd::Command;

    match Command::cargo_bin("table_formatter") {
        Ok(_) => println!("Binary found ✅"),
        Err(e) => panic!("Binary not found ❌: {:?}", e),
    }

    let output = Command::cargo_bin("table_formatter").unwrap()
        .arg(file)
        .output()
        .expect("failed to execute process");

    assert!(output.status.success(), "program exited with error");
    String::from_utf8(output.stdout).expect("not UTF-8")
}

#[test_case(SAMPLE_INPUT, SAMPLE_OUTPUT)]
fn test_file_input(input: &[&str], expected: &[&str]) {
    use tempfile::NamedTempFile;
    use std::fs;


    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, input.join("\n")).unwrap();

    let result = run_with_file(temp_file.path().to_str().unwrap());

    assert_eq!(result, expected.join("\n"));
}

#[test_case("testing/edf4.1_ranger_testfile.csv")]
fn test_with_large_file(input_file: &str) {  // covers test for symbols that take a different number of chars than displayed
    let result = run_with_file(input_file);

    // Example assertion (you can customize)
    assert!(!result.is_empty());
    assert!(
            result.contains("Type           LV  LV                                 DPS   RDPS     DPM  Ammo  \"Rate of Fire (fire/sec)\"  Damage  \"Reload (sec)\"  \"Range (m)\"  Accuracy                    Zoom  Lock time  -    -        time per mag"),
            "Header line missing or messed-up"
        );
        assert!(
            result.contains("Sniper         72  Nova Buster ZD                   80000  80000   80000     1                          1   80000               0         1240  S+                          5x            -  -    -                   1"),
            "Line below header missing or messed-up"
        );
        assert!(
            result.contains("GrenL          37  Splash Grenade α                 20000   2857   20000     1                          1   20000               6           10  Timed / 10sec               -             -  -    -                   7"),
            "Line with 2-char symbol missing or messed-up"
        );
        assert!(
            result.contains("Sniper          0  MMF40                               77     60     550     5                        0.7     110               2          600  S+                          4x            -  -    -         9.142857143"),
            "Arbitrary late line missing or messed-up"
        );
}


#[cfg(feature = "cli_tests")]
mod cli_tests {
    use super::*;
    use assert_cmd::Command;
    use std::fs;
    use tempfile::NamedTempFile;
    use std::path::PathBuf;



    #[test]
    fn test_piped_input() {
        Command::cargo_bin("table_formatter").unwrap()
            .write_stdin(SAMPLE_INPUT.join("\n"))
            .assert()
            .success()
            .stdout(format!("{}\n", SAMPLE_OUTPUT.join("\n")));
    }
}

#[test]
fn test_strip_ansi() {
    let cases = [
        "\u{1b}[38;5;208mthis is my text\u{1b}[0m", "\u{1b}[30mthis is my text\u{1b}[0m",
        "\u{1b}[31mthis is my text\u{1b}[0m", "\u{1b}[32mthis is my text\u{1b}[0m",
        "\u{1b}[33mthis is my text\u{1b}[0m", "\u{1b}[34mthis is my text\u{1b}[0m",
        "\u{1b}[35mthis is my text\u{1b}[0m", "\u{1b}[36mthis is my text\u{1b}[0m",
        "\u{1b}[37mthis is my text\u{1b}[0m", "\u{1b}[90mthis is my text\u{1b}[0m",
        "\u{1b}[91mthis is my text\u{1b}[0m", "\u{1b}[92mthis is my text\u{1b}[0m",
        "\u{1b}[93mthis is my text\u{1b}[0m", "\u{1b}[94mthis is my text\u{1b}[0m",
        "\u{1b}[95mthis is my text\u{1b}[0m", "\u{1b}[96mthis is my text\u{1b}[0m",
        "\u{1b}[97mthis is my text\u{1b}[0m", "\u{1b}[40mthis is my text\u{1b}[0m",
        "\u{1b}[41mthis is my text\u{1b}[0m", "\u{1b}[42mthis is my text\u{1b}[0m",
        "\u{1b}[43mthis is my text\u{1b}[0m", "\u{1b}[44mthis is my text\u{1b}[0m",
        "\u{1b}[45mthis is my text\u{1b}[0m", "\u{1b}[46mthis is my text\u{1b}[0m",
        "\u{1b}[47mthis is my text\u{1b}[0m", "\u{1b}[100mthis is my text\u{1b}[0m",
        "\u{1b}[101mthis is my text\u{1b}[0m", "\u{1b}[102mthis is my text\u{1b}[0m",
        "\u{1b}[103mthis is my text\u{1b}[0m", "\u{1b}[104mthis is my text\u{1b}[0m",
        "\u{1b}[105mthis is my text\u{1b}[0m", "\u{1b}[106mthis is my text\u{1b}[0m",
        "\u{1b}[107mthis is my text\u{1b}[0m", "\u{1b}[1mthis is my text\u{1b}[0m",
        "\u{1b}[2mthis is my text\u{1b}[0m", "\u{1b}[3mthis is my text\u{1b}[0m",
        "\u{1b}[4mthis is my text\u{1b}[0m", "\u{1b}[5mthis is my text\u{1b}[0m",
        "\u{1b}[6mthis is my text\u{1b}[0m", "\u{1b}[7mthis is my text\u{1b}[0m",
        "\u{1b}[8mthis is my text\u{1b}[0m", "\u{1b}[9mthis is my text\u{1b}[0m",
        "\u{1b}[22mthis is my text\u{1b}[0m", "\u{1b}[23mthis is my text\u{1b}[0m",
        "this is my text\u{1b}[0m", "\u{1b}[46m\u{1b}[23mthis is my text\u{1b}[0m",
        "this is my text",
    ];

    for case in cases {
        assert_eq!(strip_ansi(case), "this is my text");
    }
}

#[test]
fn test_is_numeric_or_neutral() {
    let numeric = [
        "10.0", "123", "123K", "123.45M", "2MB", "-1.23Gi", "5TiB", "1K", "1k", "2.5G",
        "10MiB", "4.5", "2.000", "5 TiB", "+12.5", "10%", "2k%", "1.3 k", "1.12 kb/s",
        "2 MB/s", "4.4GB/s", "4K", "1080p", "60Hz", "1440p@120Hz"
    ];

    let non_numeric = [
        "abc", "1.2X", "1.2.3", "1 0", "2/2", "kB", "2%k", "1440p@Hz", "5950X"
    ];

    for val in numeric {
        assert!(is_numeric_or_neutral(val), "{} should be numeric", val);
    }

    for val in non_numeric {
        assert!(!is_numeric_or_neutral(val), "{} should not be numeric", val);
    }
}
