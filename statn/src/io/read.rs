use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::core::data_utils::chart::{BarData, parse_ohlc_line};

/*
Read market file
*/

pub fn read_market_file(filename: &str) -> Result<BarData, String> {
    let mut bars = BarData::new();
    let mut prior_date = 0u32;

    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            println!("Reading market file...");

            for (line_num, line) in reader.lines().enumerate() {
                match line {
                    Ok(line_content) => {
                        let trimmed = line_content.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        match parse_ohlc_line(trimmed) {
                            Some((full_date, open, high, low, close)) => {
                                if full_date <= prior_date {
                                    return Err(format!("Date failed to increase at line {}", line_num + 1));
                                }
                                prior_date = full_date;
                                bars.push(full_date, open, high, low, close);
                            }
                            None => {
                                return Err(format!("Invalid data at line {}: {}", line_num + 1, trimmed));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(format!("Error reading line {}: {}", line_num + 1, e));
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Cannot open file {}: {}", filename, e));
        }
    }

    if bars.len() == 0 {
        return Err("No data read from file".to_string());
    }

    Ok(bars)
}