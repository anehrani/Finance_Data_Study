/// Computes On-Balance Volume (OBV).
///
/// # Arguments
///
/// * `closes` - A slice of closing prices.
/// * `volumes` - A slice of volumes.
///
/// # Returns
///
/// A Vec<f64> containing the OBV values.
pub fn on_balance_volume(
    closes: &[f64],
    volumes: &[f64],
) -> Vec<f64> {
    let n = closes.len();
    if n != volumes.len() || n == 0 {
        return vec![f64::NAN; n];
    }

    let mut obv = Vec::with_capacity(n);
    let mut current_obv = 0.0;
    
    // Initialize first OBV value. 
    // Standard practice varies, but typically starts at 0 or the first volume.
    // Here we'll start accumulation from the second point relative to the first.
    // The first point will be 0 (or we could treat it as 0 change).
    obv.push(current_obv);

    for i in 1..n {
        if closes[i] > closes[i - 1] {
            current_obv += volumes[i];
        } else if closes[i] < closes[i - 1] {
            current_obv -= volumes[i];
        }
        // If equal, OBV remains same
        obv.push(current_obv);
    }

    obv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obv() {
        let closes = vec![10.0, 12.0, 11.0, 11.0, 13.0];
        let volumes = vec![100.0, 200.0, 150.0, 50.0, 300.0];

        let output = on_balance_volume(&closes, &volumes);

        assert_eq!(output.len(), 5);
        
        // i=0: 0.0
        assert_eq!(output[0], 0.0);

        // i=1: Close 12 > 10 -> +200. OBV = 200
        assert_eq!(output[1], 200.0);

        // i=2: Close 11 < 12 -> -150. OBV = 200 - 150 = 50
        assert_eq!(output[2], 50.0);

        // i=3: Close 11 == 11 -> No change. OBV = 50
        assert_eq!(output[3], 50.0);

        // i=4: Close 13 > 11 -> +300. OBV = 50 + 300 = 350
        assert_eq!(output[4], 350.0);
    }
}
