

use matlib::qsortd;

pub fn clean_tails(raw: &mut [f64], tail_frac: f64) {
    let n = raw.len();
    let cover = 1.0 - 2.0 * tail_frac;

    if n == 0 {
        return;
    }

    let mut work = raw.to_vec();
    qsortd(0, n - 1, &mut work);

    let istart = 0;
    let mut istop = ((cover * (n as f64 + 1.0)) as usize).saturating_sub(1);
    if istop >= n {
        istop = n - 1;
    }

    let mut best = 1e60;
    let mut best_start = 0;
    let mut best_stop = 0;

    let mut i_start = istart;
    let mut i_stop = istop;

    while i_stop < n {
        let range = work[i_stop] - work[i_start];
        if range < best {
            best = range;
            best_start = i_start;
            best_stop = i_stop;
        }
        i_start += 1;
        i_stop += 1;
    }

    let minval = work[best_start];
    let maxval = work[best_stop];

    if maxval <= minval {
        let maxval_adj = minval * (1.0 + 1e-10);
        let minval_adj = minval * (1.0 - 1e-10);
        for item in raw.iter_mut() {
            if *item < minval_adj {
                *item = minval_adj;
            } else if *item > maxval_adj {
                *item = maxval_adj;
            }
        }
        return;
    }

    let limit = (maxval - minval) * (1.0 - cover);
    let scale = -1.0 / (maxval - minval);

    for item in raw.iter_mut() {
        if *item < minval {
            *item = minval - limit * (1.0 - (-scale * (minval - *item)).exp());
        } else if *item > maxval {
            *item = maxval + limit * (1.0 - (-scale * (*item - maxval)).exp());
        }
    }
}
