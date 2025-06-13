
use crate::{format_table, strip_ansi, is_numeric_or_neutral, DEFAULT_SEPARATOR};

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

fn to_strings(arr: &[&str]) -> Vec<String> {
    arr.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_directly() {
    assert_eq!(format_table(&to_strings(SAMPLE_INPUT), DEFAULT_SEPARATOR), to_strings(SAMPLE_OUTPUT));
    assert_eq!(format_table(&to_strings(SMTOUHOU_DATA), DEFAULT_SEPARATOR), to_strings(SMTOUHOU_DATA_ORGANIZED));
}

#[test]
fn test_solution_unchanging() {
    assert_eq!(format_table(&to_strings(SAMPLE_OUTPUT), DEFAULT_SEPARATOR), to_strings(SAMPLE_OUTPUT));
    assert_eq!(format_table(&to_strings(SMTOUHOU_DATA_ORGANIZED), DEFAULT_SEPARATOR), to_strings(SMTOUHOU_DATA_ORGANIZED));
}

#[cfg(feature = "cli_tests")]
mod cli_tests {
    use super::*;
    use assert_cmd::Command;
    use std::fs;
    use tempfile::NamedTempFile;
    use std::path::PathBuf;

    #[test]
    fn test_file_input() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, SAMPLE_INPUT.join("\n")).unwrap();

        Command::cargo_bin("table_formatter").unwrap()
            .arg(temp_file.path())
            .assert()
            .success()
            .stdout(format!("{}\n", SAMPLE_OUTPUT.join("\n")));
    }

    // covers test for symbols that take a different number of chars than displayed
    #[test]
    fn test_large_file_input() {
        let test_file_path = PathBuf::from("testing/edf4.1_ranger_testfile.csv");

        let output = Command::cargo_bin("table_formatter").unwrap()
            .arg(&test_file_path)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let stdout = String::from_utf8_lossy(&output);

        assert!(
            stdout.contains("Type           LV  LV                                 DPS   RDPS     DPM  Ammo  \"Rate of Fire (fire/sec)\"  Damage  \"Reload (sec)\"  \"Range (m)\"  Accuracy                    Zoom  Lock time  -    -        time per mag"),
            "Header line missing or messed-up"
        );
        assert!(
            stdout.contains("Sniper         72  Nova Buster ZD                   80000  80000   80000     1                          1   80000               0         1240  S+                          5x            -  -    -                   1"),
            "Line below header missing or messed-up"
        );
        assert!(
            stdout.contains("GrenL          37  Splash Grenade α                 20000   2857   20000     1                          1   20000               6           10  Timed / 10sec               -             -  -    -                   7"),
            "Line with 2-char symbol missing or messed-up"
        );
        assert!(
            stdout.contains("Sniper          0  MMF40                               77     60     550     5                        0.7     110               2          600  S+                          4x            -  -    -         9.142857143"),
            "Arbitrary late line missing or messed-up"
        );
    }

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
