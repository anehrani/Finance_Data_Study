use crate::trend::ma::moving_average;

/// Bollinger Bands Output
#[derive(Debug, Clone)]
pub struct BollingerBandsOutput {
    /// Upper Band = Middle Band + (Multiplier * SD)
    pub upper_band: Vec<f64>,
    /// Middle Band = SMA
    pub middle_band: Vec<f64>,
    /// Lower Band = Middle Band - (Multiplier * SD)
    pub lower_band: Vec<f64>,
}

/// Computes Bollinger Bands.
///
/// # Arguments
///
/// * `data` - A slice of f64 values (typically closing prices).
/// * `period` - The window size for the moving average and standard deviation (typically 20).
/// * `multiplier` - The number of standard deviations for the bands (typically 2.0).
///
/// # Returns
///
/// A `BollingerBandsOutput` struct containing the upper, middle, and lower bands.
pub fn bollinger_bands(
    data: &[f64],
    period: usize,
    multiplier: f64,
) -> BollingerBandsOutput {
    if period == 0 || period > data.len() {
        let n = data.len();
        return BollingerBandsOutput {
            upper_band: vec![f64::NAN; n],
            middle_band: vec![f64::NAN; n],
            lower_band: vec![f64::NAN; n],
        };
    }

    let middle_band = moving_average(data, period);
    let mut upper_band = Vec::with_capacity(data.len());
    let mut lower_band = Vec::with_capacity(data.len());

    // Calculate rolling standard deviation
    for i in 0..data.len() {
        if i < period - 1 {
            upper_band.push(f64::NAN);
            lower_band.push(f64::NAN);
            continue;
        }

        let slice = &data[i + 1 - period..=i];
        let mean = middle_band[i];
        
        let variance: f64 = slice.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / period as f64;
        
        let std_dev = variance.sqrt();

        upper_band.push(mean + multiplier * std_dev);
        lower_band.push(mean - multiplier * std_dev);
    }

    BollingerBandsOutput {
        upper_band,
        middle_band,
        lower_band,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bollinger_bands() {
        let data = vec![
            10.0, 10.0, 10.0, 10.0, 10.0,
            12.0, 12.0, 12.0, 12.0, 12.0,
        ];
        let period = 5;
        let multiplier = 2.0;

        let output = bollinger_bands(&data, period, multiplier);

        assert_eq!(output.middle_band.len(), 10);
        
        // First 4 should be NaN
        assert!(output.middle_band[3].is_nan());
        assert!(output.upper_band[3].is_nan());
        assert!(output.lower_band[3].is_nan());

        // Index 4: [10, 10, 10, 10, 10] -> Mean 10, StdDev 0
        assert!((output.middle_band[4] - 10.0).abs() < 1e-10);
        assert!((output.upper_band[4] - 10.0).abs() < 1e-10);
        assert!((output.lower_band[4] - 10.0).abs() < 1e-10);

        // Index 9: [12, 12, 12, 12, 12] -> Mean 12, StdDev 0
        assert!((output.middle_band[9] - 12.0).abs() < 1e-10);
        assert!((output.upper_band[9] - 12.0).abs() < 1e-10);
        assert!((output.lower_band[9] - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_bollinger_bands_variation() {
        let data = vec![10.0, 12.0, 14.0, 16.0, 18.0];
        let period = 5;
        let multiplier = 2.0;

        let output = bollinger_bands(&data, period, multiplier);

        // Mean = 14
        // Variance = ((10-14)^2 + ... + (18-14)^2) / 5 = (16 + 4 + 0 + 4 + 16) / 5 = 40 / 5 = 8
        // StdDev = sqrt(8) approx 2.8284
        // Upper = 14 + 2 * 2.8284 = 19.6568
        // Lower = 14 - 2 * 2.8284 = 8.3432

        let mean = 14.0;
        let std_dev = 8.0_f64.sqrt();
        
        assert!((output.middle_band[4] - mean).abs() < 1e-10);
        assert!((output.upper_band[4] - (mean + multiplier * std_dev)).abs() < 1e-10);
        assert!((output.lower_band[4] - (mean - multiplier * std_dev)).abs() < 1e-10);
    }
}
