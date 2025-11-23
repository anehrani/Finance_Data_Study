/// Quicksort implementation for f64 slices
pub fn qsortd(data: &mut [f64]) {
    if data.len() <= 1 {
        return;
    }
    qsortd_impl(data, 0, data.len() - 1);
}

fn qsortd_impl(data: &mut [f64], first: usize, last: usize) {
    if first >= last {
        return;
    }

    let split = data[(first + last) / 2];
    let mut lower = first;
    let mut upper = last;

    loop {
        while split > data[lower] {
            lower += 1;
        }
        while split < data[upper] {
            upper = upper.saturating_sub(1);
        }

        if lower == upper {
            lower += 1;
            if upper == 0 {
                break;
            }
            upper -= 1;
        } else if lower < upper {
            data.swap(lower, upper);
            lower += 1;
            if upper == 0 {
                break;
            }
            upper -= 1;
        }

        if lower > upper {
            break;
        }
    }

    if first < upper && upper != usize::MAX {
        qsortd_impl(data, first, upper);
    }
    if lower < last {
        qsortd_impl(data, lower, last);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qsortd() {
        let mut data = vec![5.0, 2.0, 8.0, 1.0, 9.0, 3.0];
        qsortd(&mut data);
        assert_eq!(data, vec![1.0, 2.0, 3.0, 5.0, 8.0, 9.0]);
    }

    #[test]
    fn test_qsortd_empty() {
        let mut data: Vec<f64> = vec![];
        qsortd(&mut data);
        assert_eq!(data, vec![]);
    }

    #[test]
    fn test_qsortd_single() {
        let mut data = vec![5.0];
        qsortd(&mut data);
        assert_eq!(data, vec![5.0]);
    }
}
