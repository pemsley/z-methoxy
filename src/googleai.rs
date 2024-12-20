
use std::io::{BufRead, BufReader};
use std::fs::File;

#[derive(Debug)]
struct Entry {
    time: String,
    count: String,
    filename: String,
}

fn split_lines(input_string: &str) -> Vec<Entry> {
    let mut entries = Vec::new();

    for line in input_string.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 3 { // Ensure we have at least 3 fields
            let time = fields[0].to_string();
            let count = fields[1].to_string();

            // Reconstruct the filename from the remaining fields, handling spaces
            let filename = fields[2..].join(" ");

            entries.push(Entry { time, count, filename });
        } else {
            // Handle lines with fewer than 3 fields (e.g., log errors, skip)
            eprintln!("Invalid line format: {}", line); // Or choose a different handling strategy
        }
    }

    entries
}



fn read_file_and_split(filename: &str) -> Result<Vec<Entry>, std::io::Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = String::new();

    for line in reader.lines() {
        lines.push_str(&line?);
        lines.push('\n'); // Add back the newline
    }
    Ok(split_lines(&lines))

}

fn main() -> Result<(), std::io::Error> {
    let entries = read_file_and_split("your_file.txt")?;

    for entry in entries {
        println!("{:?}", entry);
    }

    Ok(())
}

