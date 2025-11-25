
pub fn qsort_helper(data: &mut [f64], first: i32, last: i32) {
    if first >= last {
        return;
    }

    let split = data[((first + last) / 2) as usize];
    let mut lower = first;
    let mut upper = last;

    loop {
        while split > data[lower as usize] {
            lower += 1;
        }
        while split < data[upper as usize] {
            upper -= 1;
        }

        if lower == upper {
            lower += 1;
            upper -= 1;
        } else if lower < upper {
            data.swap(lower as usize, upper as usize);
            lower += 1;
            upper -= 1;
        }

        if lower > upper {
            break;
        }
    }

    if first < upper {
        qsort_helper(data, first, upper);
    }
    if lower < last {
        qsort_helper(data, lower, last);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT - Quick sort a double array
--------------------------------------------------------------------------------
*/
pub fn qsortd(first: usize, last: usize, data: &mut [f64]) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortd(first, upper, data);
    }
    if lower < last {
        qsortd(lower, last, data);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT with one slave array
--------------------------------------------------------------------------------
*/
pub fn qsortds(first: usize, last: usize, data: &mut [f64], slave: &mut [f64]) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortds(first, upper, data, slave);
    }
    if lower < last {
        qsortds(lower, last, data, slave);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT with two slave arrays
--------------------------------------------------------------------------------
*/
pub fn qsortds2(
    first: usize,
    last: usize,
    data: &mut [f64],
    slave: &mut [f64],
    slave2: &mut [f64],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            slave2.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortds2(first, upper, data, slave, slave2);
    }
    if lower < last {
        qsortds2(lower, last, data, slave, slave2);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT with three slave arrays
--------------------------------------------------------------------------------
*/
pub fn qsortds3(
    first: usize,
    last: usize,
    data: &mut [f64],
    slave: &mut [f64],
    slave2: &mut [f64],
    slave3: &mut [f64],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            slave2.swap(lower, upper);
            slave3.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortds3(first, upper, data, slave, slave2, slave3);
    }
    if lower < last {
        qsortds3(lower, last, data, slave, slave2, slave3);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT with four slave arrays
--------------------------------------------------------------------------------
*/
pub fn qsortds4(
    first: usize,
    last: usize,
    data: &mut [f64],
    slave: &mut [f64],
    slave2: &mut [f64],
    slave3: &mut [f64],
    slave4: &mut [f64],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            slave2.swap(lower, upper);
            slave3.swap(lower, upper);
            slave4.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortds4(first, upper, data, slave, slave2, slave3, slave4);
    }
    if lower < last {
        qsortds4(lower, last, data, slave, slave2, slave3, slave4);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT with five slave arrays
--------------------------------------------------------------------------------
*/
pub fn qsortds5(
    first: usize,
    last: usize,
    data: &mut [f64],
    slave: &mut [f64],
    slave2: &mut [f64],
    slave3: &mut [f64],
    slave4: &mut [f64],
    slave5: &mut [f64],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            slave2.swap(lower, upper);
            slave3.swap(lower, upper);
            slave4.swap(lower, upper);
            slave5.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortds5(first, upper, data, slave, slave2, slave3, slave4, slave5);
    }
    if lower < last {
        qsortds5(lower, last, data, slave, slave2, slave3, slave4, slave5);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT with six slave arrays
--------------------------------------------------------------------------------
*/
pub fn qsortds6(
    first: usize,
    last: usize,
    data: &mut [f64],
    slave: &mut [f64],
    slave2: &mut [f64],
    slave3: &mut [f64],
    slave4: &mut [f64],
    slave5: &mut [f64],
    slave6: &mut [f64],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            slave2.swap(lower, upper);
            slave3.swap(lower, upper);
            slave4.swap(lower, upper);
            slave5.swap(lower, upper);
            slave6.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortds6(first, upper, data, slave, slave2, slave3, slave4, slave5, slave6);
    }
    if lower < last {
        qsortds6(lower, last, data, slave, slave2, slave3, slave4, slave5, slave6);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT with seven slave arrays
--------------------------------------------------------------------------------
*/
pub fn qsortds7(
    first: usize,
    last: usize,
    data: &mut [f64],
    slave: &mut [f64],
    slave2: &mut [f64],
    slave3: &mut [f64],
    slave4: &mut [f64],
    slave5: &mut [f64],
    slave6: &mut [f64],
    slave7: &mut [f64],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            slave2.swap(lower, upper);
            slave3.swap(lower, upper);
            slave4.swap(lower, upper);
            slave5.swap(lower, upper);
            slave6.swap(lower, upper);
            slave7.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortds7(first, upper, data, slave, slave2, slave3, slave4, slave5, slave6, slave7);
    }
    if lower < last {
        qsortds7(lower, last, data, slave, slave2, slave3, slave4, slave5, slave6, slave7);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT with eight slave arrays
--------------------------------------------------------------------------------
*/
pub fn qsortds8(
    first: usize,
    last: usize,
    data: &mut [f64],
    slave: &mut [f64],
    slave2: &mut [f64],
    slave3: &mut [f64],
    slave4: &mut [f64],
    slave5: &mut [f64],
    slave6: &mut [f64],
    slave7: &mut [f64],
    slave8: &mut [f64],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            slave2.swap(lower, upper);
            slave3.swap(lower, upper);
            slave4.swap(lower, upper);
            slave5.swap(lower, upper);
            slave6.swap(lower, upper);
            slave7.swap(lower, upper);
            slave8.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortds8(first, upper, data, slave, slave2, slave3, slave4, slave5, slave6, slave7, slave8);
    }
    if lower < last {
        qsortds8(lower, last, data, slave, slave2, slave3, slave4, slave5, slave6, slave7, slave8);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT with nine slave arrays
--------------------------------------------------------------------------------
*/
pub fn qsortds9(
    first: usize,
    last: usize,
    data: &mut [f64],
    slave: &mut [f64],
    slave2: &mut [f64],
    slave3: &mut [f64],
    slave4: &mut [f64],
    slave5: &mut [f64],
    slave6: &mut [f64],
    slave7: &mut [f64],
    slave8: &mut [f64],
    slave9: &mut [f64],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            slave2.swap(lower, upper);
            slave3.swap(lower, upper);
            slave4.swap(lower, upper);
            slave5.swap(lower, upper);
            slave6.swap(lower, upper);
            slave7.swap(lower, upper);
            slave8.swap(lower, upper);
            slave9.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortds9(
            first, upper, data, slave, slave2, slave3, slave4, slave5, slave6, slave7, slave8,
            slave9,
        );
    }
    if lower < last {
        qsortds9(
            lower, last, data, slave, slave2, slave3, slave4, slave5, slave6, slave7, slave8,
            slave9,
        );
    }
}

/*
--------------------------------------------------------------------------------
   QSORT for doubles with integer slave
--------------------------------------------------------------------------------
*/
pub fn qsortdsi(first: usize, last: usize, data: &mut [f64], slave: &mut [i32]) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortdsi(first, upper, data, slave);
    }
    if lower < last {
        qsortdsi(lower, last, data, slave);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT for doubles with u64 slave
--------------------------------------------------------------------------------
*/
pub fn qsortds64(first: usize, last: usize, data: &mut [f64], slave: &mut [u64]) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortds64(first, upper, data, slave);
    }
    if lower < last {
        qsortds64(lower, last, data, slave);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT for doubles with double and int slaves
--------------------------------------------------------------------------------
*/
pub fn qsortdsri(
    first: usize,
    last: usize,
    data: &mut [f64],
    slave: &mut [f64],
    slave2: &mut [i32],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            slave2.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortdsri(first, upper, data, slave, slave2);
    }
    if lower < last {
        qsortdsri(lower, last, data, slave, slave2);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT for doubles with two int slaves
--------------------------------------------------------------------------------
*/
pub fn qsortdsii(
    first: usize,
    last: usize,
    data: &mut [f64],
    slave: &mut [i32],
    slave2: &mut [i32],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            slave2.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortdsii(first, upper, data, slave, slave2);
    }
    if lower < last {
        qsortdsii(lower, last, data, slave, slave2);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT for integers
--------------------------------------------------------------------------------
*/
pub fn qsorti(first: usize, last: usize, data: &mut [i32]) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsorti(first, upper, data);
    }
    if lower < last {
        qsorti(lower, last, data);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT for integers with int slave
--------------------------------------------------------------------------------
*/
pub fn qsortisi(first: usize, last: usize, data: &mut [i32], slave: &mut [i32]) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortisi(first, upper, data, slave);
    }
    if lower < last {
        qsortisi(lower, last, data, slave);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT for integers with double slave
--------------------------------------------------------------------------------
*/
pub fn qsortisd(first: usize, last: usize, data: &mut [i32], slave: &mut [f64]) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortisd(first, upper, data, slave);
    }
    if lower < last {
        qsortisd(lower, last, data, slave);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT for integers with short int and int slaves
--------------------------------------------------------------------------------
*/
pub fn qsortissii(
    first: usize,
    last: usize,
    data: &mut [i32],
    slave1: &mut [i16],
    slave2: &mut [i32],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave1.swap(lower, upper);
            slave2.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortissii(first, upper, data, slave1, slave2);
    }
    if lower < last {
        qsortissii(lower, last, data, slave1, slave2);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT for u64 with short int and int slaves
--------------------------------------------------------------------------------
*/
pub fn qsort64ssii(
    first: usize,
    last: usize,
    data: &mut [u64],
    slave1: &mut [i16],
    slave2: &mut [i32],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave1.swap(lower, upper);
            slave2.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsort64ssii(first, upper, data, slave1, slave2);
    }
    if lower < last {
        qsort64ssii(lower, last, data, slave1, slave2);
    }
}

/*
--------------------------------------------------------------------------------
   QSORT for integers with four double slaves
--------------------------------------------------------------------------------
*/
pub fn qsortid4(
    first: usize,
    last: usize,
    data: &mut [i32],
    slave1: &mut [f64],
    slave2: &mut [f64],
    slave3: &mut [f64],
    slave4: &mut [f64],
) {
    if first >= last {
        return;
    }

    let mut lower = first;
    let mut upper = last;
    let split = data[(first + last) / 2];

    loop {
        while lower < data.len() && split > data[lower] {
            lower += 1;
        }
        while upper > 0 && split < data[upper] {
            upper -= 1;
        }

        if lower == upper {
            if lower < data.len() - 1 {
                lower += 1;
            }
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            slave1.swap(lower, upper);
            slave2.swap(lower, upper);
            slave3.swap(lower, upper);
            slave4.swap(lower, upper);
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if (lower > upper) {
            break;
        }
    }

    if first < upper {
        qsortid4(first, upper, data, slave1, slave2, slave3, slave4);
    }
    if lower < last {
        qsortid4(lower, last, data, slave1, slave2, slave3, slave4);
    }
}