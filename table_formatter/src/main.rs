use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::sync::LazyLock;
use itertools::izip;
use std::fmt::Write;
use std::iter::repeat;

// ——— Configuration ——————————————————————————————
const DEFAULT_SEPARATOR: usize = 2;

// Regular expression patterns
static SPLIT_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s{2,}|\t+").unwrap());
static NUMERIC_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[+-]?[0-9]+(?:\.[0-9]+)?\s?[pKkMmGgTt]?(?:i?[bB]?(/s)?|%|Hz|@[0-9]+Hz)?$").unwrap()
});

// ——— Utilities ——————————————————————————————————————
pub fn strip_ansi(text: &str) -> String {
    console::strip_ansi_codes(text).to_string()
}

pub fn visible_len(text: &str) -> usize {
    console::measure_text_width(&console::strip_ansi_codes(text))
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

    // Transpose table: convert rows to columns
    let mut columns = vec![vec![]; num_cols];
    for (col_idx, cell) in rows.iter().flat_map(|row| row.iter().enumerate()) {
        columns[col_idx].push(cell);
    }

    // Return calculated widths and numeric-flags
    (0..num_cols).into_par_iter()
        .map(|col_idx| {
            let col = &columns[col_idx];
            let width = col.par_iter().map(|cell| visible_len(cell)).max().unwrap_or(0);
            let is_numeric = col.par_iter().skip(1).all(|cell| is_numeric_or_neutral(cell));
            (width, is_numeric)
        })
        .unzip()
}

fn format_row(cells: &[String], widths: &[usize], is_numeric: &[bool], sep_width: usize, ) -> String {
    // Pre-compute total capacity
    let total = widths.iter().sum::<usize>()
        + sep_width * widths.len().saturating_sub(1);
    let mut out = String::with_capacity(total);
    let spacer = " ".repeat(sep_width);

    // Bind a single empty String for all "missing" cells
    let empty = String::new();

    // Zip widths, flags, and cells (falling back to &empty)
    for (&width, &numeric, cell) in izip!(
        widths.iter(),
        is_numeric.iter(),
        cells.iter().chain(repeat(&empty))
    ) {
        if numeric { write!(out, "{:>width$}", cell, width = width).unwrap(); }
        else { write!(out, "{:<width$}", cell, width = width).unwrap(); }
        out.push_str(&spacer);
    }

    // Trim off the trailing separator
    out.truncate(out.len().saturating_sub(sep_width));
    out
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
