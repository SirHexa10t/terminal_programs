use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::sync::LazyLock;

// ——— Configuration ——————————————————————————————
const DEFAULT_SEPARATOR: usize = 2;

// Regular expression patterns
static SPLIT_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s{2,}|\t+").unwrap());
static ANSI_ESCAPE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap());
static NUMERIC_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[+-]?[0-9]+(?:\.[0-9]+)?\s?[pKkMmGgTt]?(?:i?[bB]?(/s)?|%|Hz|@[0-9]+Hz)?$").unwrap()
});

// ——— Utilities ——————————————————————————————————————
pub fn strip_ansi(text: &str) -> String {
    ANSI_ESCAPE.replace_all(text, "").to_string()
}

pub fn is_numeric_or_neutral(text: &str) -> bool {
    let clean = strip_ansi(text);
    let clean = clean.trim();
    matches!(clean, "" | "-" | "--" | "---" | "*" | "−" | "=" | "y" | "n")
        || NUMERIC_PATTERN.is_match(clean)
}

fn split_row(line: &str) -> Vec<String> {
    SPLIT_PATTERN.split(line.trim()).map(String::from).collect()
}

fn detect_column_properties(rows: &[Vec<String>]) -> (Vec<usize>, Vec<bool>) {
    let num_cols = rows.iter().map(Vec::len).max().unwrap_or(0);

    // Calculate widths and numeric flags together
    let (widths, is_numeric): (Vec<_>, Vec<_>) = (0..num_cols)
        .into_par_iter()
        .map(|col_idx| {
            let width = rows.par_iter()
                .filter_map(|row| row.get(col_idx))
                .map(|cell| strip_ansi(cell).len())
                .max()
                .unwrap_or(0);

            let is_numeric = rows.par_iter()
                .skip(1) // Skip header
                .filter_map(|row| row.get(col_idx))
                .all(|cell| is_numeric_or_neutral(cell));

            (width, is_numeric)
        })
        .unzip();

    (widths, is_numeric)
}

fn format_row(cells: &[String], widths: &[usize], is_numeric: &[bool], sep_width: usize) -> String {
    let spacer = " ".repeat(sep_width);

    widths.iter()
        .enumerate()
        .map(|(i, &width)| {
            let cell = cells.get(i).map_or("", String::as_str);
            let visual_len = strip_ansi(cell).len();
            let pad = " ".repeat(width.saturating_sub(visual_len));

            if is_numeric[i] {
                format!("{pad}{cell}")  // right-align
            } else {
                format!("{cell}{pad}")  // left-align
            }
        })
        .collect::<Vec<_>>()
        .join(&spacer)
}

// ——— Core formatting functions ——————————————————————————————————
pub fn format_table(lines: &[String], separator: usize) -> Vec<String> {
    // Split rows - always use par_iter, rayon will handle the parallelization decision
    let rows: Vec<Vec<String>> = lines.par_iter().map(|line| split_row(line)).collect();
    let (widths, is_numeric) = detect_column_properties(&rows);

    // Format rows - always use par_iter
    rows.par_iter()
        .map(|row| format_row(row, &widths, &is_numeric, separator))
        .collect()
}

fn print_table(lines: &[String], separator: usize) {
    format_table(lines, separator)
        .iter()
        .for_each(|line| println!("{line}"));
}

// ——— CLI Options ——————————————————————————————————————
#[derive(Parser)]
#[command(author, version, about = "Align whitespace-delimited columns into a neat table")]
struct Args {
    /// Input file path (or use stdin if not provided)
    #[arg(default_value = "-")]
    input: String,

    /// Number of spaces to separate columns
    #[arg(short, long, default_value_t = DEFAULT_SEPARATOR)]
    separator: usize,
}

// ——— Main Function ——————————————————————————————————————
fn main() -> io::Result<()> {
    let args = Args::parse();

    let lines: Vec<String> = if args.input == "-" {
        io::stdin().lock().lines().collect::<Result<_, _>>()?
    } else {
        BufReader::new(File::open(&args.input)?).lines().collect::<Result<_, _>>()?
    };

    print_table(&lines, args.separator);
    Ok(())
}

// Include tests
#[cfg(test)]
mod tests;