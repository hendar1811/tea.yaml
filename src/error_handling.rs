use colored::*;
use std::ops::Range;

/// Prints an error message along with the location in the source file where the error occurred
pub fn pretty_print_error(
    message: &str,
    span: Range<usize>,
    source: &str,
    filename: std::path::PathBuf,
) -> () {
    // Find the start and end of the error as line/col pair.
    let (start_line, start_col) = index_to_file_position(source, span.start);
    let (end_line, end_col) = index_to_file_position(source, span.end);

    let lines: Vec<_> = source.split("\n").collect();

    // let the user know there has been an error
    println!("ERROR in {:?}: {}", filename, message);

    // Print the error location
    for i in start_line..(end_line + 1) {
        // print the line from the source file
        println!("{:4} {} {}", i + 1, "|".blue().bold(), lines[i]);

        // print the error underline (some amount of blank followed by the underline)
        let blank_size;
        let underline_size;
        if i == start_line && i == end_line {
            blank_size = start_col;
            underline_size = end_col - start_col;
        } else if i == start_line {
            blank_size = start_col;
            underline_size = lines[i].chars().count() - start_col;
        } else if i == end_line {
            blank_size = 0;
            underline_size = end_col;
        } else {
            blank_size = 0;
            underline_size = lines[i].chars().count();
        }
        println!(
            "{:4} {} {}{}",
            "",
            "|".blue().bold(),
            " ".repeat(blank_size),
            "^".repeat(underline_size).red().bold()
        );
    }
}

/// Converts an index into a string into a line/column pair for that same string
fn index_to_file_position(source: &str, index: usize) -> (usize, usize) {
    let mut line = 0;
    let mut last_newline_index = 0;

    for (i, c) in source[0..index].chars().enumerate() {
        if c == '\n' {
            line += 1;
            last_newline_index = i;
        }
    }

    let col = index - last_newline_index;

    return (line, col);
}