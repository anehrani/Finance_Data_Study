use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

#[derive(Debug)]
pub struct MarketData {
    #[allow(dead_code)]
    pub dates: Vec<i32>,
    #[allow(dead_code)]
    pub opens: Vec<f64>,
    pub highs: Vec<f64>,
    pub lows: Vec<f64>,
    pub closes: Vec<f64>,
}

pub fn read_market_data(filename: &str) -> MarketData {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("\n\nCannot open market history file {}", filename);
            process::exit(1);
        }
    };

    let reader = BufReader::new(file);
    let mut dates = Vec::new();
    let mut opens = Vec::new();
    let mut highs = Vec::new();
    let mut lows = Vec::new();
    let mut closes = Vec::new();
    let mut prior_date = 0;

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => {
                eprintln!("Error reading line {}", line_num + 1);
                process::exit(1);
            }
        };

        if line.is_empty() {
            break;
        }

        let (date, open, high, low, close) = parse_line(&line, line_num + 1, filename);

        if date <= prior_date {
            eprintln!("ERROR... Date failed to increase in line {}", line_num + 1);
            process::exit(1);
        }

        prior_date = date;
        dates.push(date);
        opens.push(open);
        highs.push(high);
        lows.push(low);
        closes.push(close);
    }

    MarketData {
        dates,
        opens,
        highs,
        lows,
        closes,
    }
}

fn parse_line(line: &str, line_num: usize, filename: &str) -> (i32, f64, f64, f64, f64) {
    // Parse date
    if line.len() < 9 {
        eprintln!("Invalid date reading line {} of file {}", line_num, filename);
        process::exit(1);
    }

    let date_str = &line[0..8];
    let full_date: i32 = match date_str.parse() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Invalid date reading line {} of file {}", line_num, filename);
            process::exit(1);
        }
    };

    let year = full_date / 10000;
    let month = (full_date % 10000) / 100;
    let day = full_date % 100;

    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(1800..=2030).contains(&year)
    {
        eprintln!("ERROR... Invalid date {} in line {}", full_date, line_num);
        process::exit(1);
    }

    // Parse prices
    let parts: Vec<&str> = line[9..]
        .split([' ', '\t', ','])
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() < 4 {
        eprintln!("Invalid price data reading line {} of file {}", line_num, filename);
        process::exit(1);
    }

    let open_price: f64 = match parts[0].parse::<f64>() {
        Ok(p) => p.ln(),
        Err(_) => {
            eprintln!("Invalid open price reading line {}", line_num);
            process::exit(1);
        }
    };

    let high_price: f64 = match parts[1].parse::<f64>() {
        Ok(p) => p.ln(),
        Err(_) => {
            eprintln!("Invalid high price reading line {}", line_num);
            process::exit(1);
        }
    };

    let low_price: f64 = match parts[2].parse::<f64>() {
        Ok(p) => p.ln(),
        Err(_) => {
            eprintln!("Invalid low price reading line {}", line_num);
            process::exit(1);
        }
    };

    let close_price: f64 = match parts[3].parse::<f64>() {
        Ok(p) => p.ln(),
        Err(_) => {
            eprintln!("Invalid close price reading line {}", line_num);
            process::exit(1);
        }
    };

    if low_price > open_price
        || low_price > close_price
        || high_price < open_price
        || high_price < close_price
    {
        eprintln!(
            "Invalid open/high/low/close reading line {} of file {}",
            line_num, filename
        );
        process::exit(1);
    }

    (full_date, open_price, high_price, low_price, close_price)
}