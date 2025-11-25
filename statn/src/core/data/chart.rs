

/*
Bar data structure
*/

#[derive(Clone)]
pub struct BarData {
    pub date: Vec<u32>,
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
}

impl Default for BarData {
    fn default() -> Self {
        Self::new()
    }
}

impl BarData {
    pub fn new() -> Self {
        BarData {
            date: Vec::new(),
            open: Vec::new(),
            high: Vec::new(),
            low: Vec::new(),
            close: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.open.len()
    }

    pub fn push(&mut self, date: u32, open: f64, high: f64, low: f64, close: f64) {
        self.date.push(date);
        self.open.push(open);
        self.high.push(high);
        self.low.push(low);
        self.close.push(close);
    }

    pub fn validate_ohlc(&self, idx: usize) -> bool {
        self.low[idx] <= self.open[idx]
            && self.low[idx] <= self.close[idx]
            && self.high[idx] >= self.open[idx]
            && self.high[idx] >= self.close[idx]
    }
}


/*
Parse OHLC from line
*/

pub fn parse_ohlc_line(line: &str) -> Option<(u32, f64, f64, f64, f64)> {
    if line.len() < 9 {
        return None;
    }

    let date_str = &line[0..8];
    if !date_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let full_date: u32 = date_str.parse().ok()?;
    let year = full_date / 10000;
    let month = (full_date / 100) % 100;
    let day = full_date % 100;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || !(1800..=2030).contains(&year) {
        return None;
    }

    let parts: Vec<&str> = line[9..]
        .split([' ', '\t', ','])
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() < 4 {
        return None;
    }

    let open_price = parts[0].parse::<f64>().ok()?.ln();
    let high_price = parts[1].parse::<f64>().ok()?.ln();
    let low_price = parts[2].parse::<f64>().ok()?.ln();
    let close_price = parts[3].parse::<f64>().ok()?.ln();

    if low_price > open_price
        || low_price > close_price
        || high_price < open_price
        || high_price < close_price
    {
        return None;
    }

    Some((full_date, open_price, high_price, low_price, close_price))
}