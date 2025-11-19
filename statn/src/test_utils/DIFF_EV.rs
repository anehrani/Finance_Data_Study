/******************************************************************************/
/*                                                                            */
/*  DIFF_EV - Differential evolution optimization                             */
/*                                                                            */
/*  Popsize should be 5 to 10 times n, more for a more global search.         */
/*  Overinit should be 0 for simple problems, or popsize for hard problems.   */
/*  Mutate_dev should be about 0.4 to 1.2, with larger values giving a more   */
/*  global search.                                                            */
/*  Pcross should be 0-1.  This is the probability that each parameter in     */
/*  crossover will be chosen from the noisy parent (as opposed to the pure    */
/*  parent against which the child will be tested).                           */
/*  The authors state that small values (like 0.1) produce a more global      */
/*  solution, but that is opposite my intuition.                              */
/*                                                                            */
/******************************************************************************/

use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static LOCAL_STATE: RefCell<LocalState> = RefCell::new(LocalState::new());
}

#[derive(Clone)]
struct LocalState {
    local_ivar: usize,
    local_base: f64,
    local_x: Vec<f64>,
    local_nvars: usize,
    local_nints: usize,
    local_mintrades: i32,
    local_low_bounds: Vec<f64>,
    local_high_bounds: Vec<f64>,
}

impl LocalState {
    fn new() -> Self {
        LocalState {
            local_ivar: 0,
            local_base: 0.0,
            local_x: Vec::new(),
            local_nvars: 0,
            local_nints: 0,
            local_mintrades: 0,
            local_low_bounds: Vec::new(),
            local_high_bounds: Vec::new(),
        }
    }
}

fn unifrand() -> f64 {
    // Returns random number between 0 and 1
    rand::random::<f64>()
}

fn ensure_legal(
    nvars: usize,
    nints: usize,
    low_bounds: &[f64],
    high_bounds: &[f64],
    params: &mut [f64],
) -> f64 {
    let mut penalty = 0.0;

    for i in 0..nvars {
        if i < nints {
            if params[i] >= 0.0 {
                params[i] = (params[i] + 0.5).floor();
            } else if params[i] < 0.0 {
                params[i] = -((0.5 - params[i]).floor());
            }
        }
        if params[i] > high_bounds[i] {
            penalty += 1.0e10 * (params[i] - high_bounds[i]);
            params[i] = high_bounds[i];
        }
        if params[i] < low_bounds[i] {
            penalty += 1.0e10 * (low_bounds[i] - params[i]);
            params[i] = low_bounds[i];
        }
    }

    penalty
}

fn c_func(param: f64) -> f64 {
    LOCAL_STATE.with(|state| {
        let mut st = state.borrow_mut();
        st.local_x[st.local_ivar] = param;
        let penalty = ensure_legal(
            st.local_nvars,
            st.local_nints,
            &st.local_low_bounds,
            &st.local_high_bounds,
            &mut st.local_x,
        );
        // Note: This would require passing the criterion function via thread-local storage
        // For now, we'll need to restructure this part
        0.0 - penalty
    })
}

pub fn diff_ev<F, S>(
    criter: F,
    nvars: usize,
    nints: usize,
    popsize: usize,
    overinit: usize,
    mintrd: i32,
    max_evals: usize,
    max_bad_gen: usize,
    mutate_dev: f64,
    pcross: f64,
    pclimb: f64,
    low_bounds: &[f64],
    high_bounds: &[f64],
    params: &mut [f64],
    print_progress: bool,
    stoc_bias: Option<&mut S>,
) -> i32
where
    F: Fn(&[f64], i32) -> f64,
    S: StocBiasTrait,
{
    let mut ret_code = 0;
    let mut local_mintrades = mintrd;

    let dim = nvars + 1; // Each case is nvars variables plus criterion

    let mut pop1 = vec![0.0; dim * popsize];
    let mut pop2 = vec![0.0; dim * popsize];
    let mut best = vec![0.0; dim];

    let mut failures = 0;
    let mut n_evals = 0;

    if let Some(sb) = stoc_bias {
        sb.collect(true);
    }

    // Generate initial population
    let mut grand_best = 0.0;
    let mut worstf = 0.0;
    let mut avgf = 0.0;

    for ind in 0..(popsize + overinit) {
        let popptr = if ind < popsize {
            &mut pop1[ind * dim..(ind + 1) * dim]
        } else {
            &mut pop2[0..dim]
        };

        for i in 0..nvars {
            if i < nints {
                popptr[i] =
                    low_bounds[i] + ((unifrand() * (high_bounds[i] - low_bounds[i] + 1.0)) as i32) as f64;
                if popptr[i] > high_bounds[i] {
                    popptr[i] = high_bounds[i];
                }
            } else {
                popptr[i] = low_bounds[i] + (unifrand() * (high_bounds[i] - low_bounds[i]));
            }
        }

        let value = criter(popptr, local_mintrades);
        popptr[nvars] = value;
        n_evals += 1;

        if ind == 0 {
            grand_best = value;
            worstf = value;
            avgf = value;
            best.copy_from_slice(popptr);
        }

        if value <= 0.0 {
            if n_evals > max_evals {
                goto_finished(&mut pop1, &mut pop2);
                return 1;
            }
            if failures >= 500 {
                failures = 0;
                local_mintrades = (local_mintrades * 9) / 10;
                if local_mintrades < 1 {
                    local_mintrades = 1;
                }
            }
            continue;
        } else {
            failures = 0;
        }

        if value > grand_best {
            best.copy_from_slice(popptr);
            grand_best = value;
        }

        if value < worstf {
            worstf = value;
        }

        avgf += value;

        if print_progress {
            let avg = if ind < popsize {
                avgf / (ind as f64 + 1.0)
            } else {
                avgf / popsize as f64
            };
            println!(
                "\n{}: Val={:.4} Best={:.4} Worst={:.4} Avg={:.4} (fail rate={:.1})",
                ind,
                value,
                grand_best,
                worstf,
                avg,
                n_evals as f64 / (ind as f64 + 1.0)
            );
            for i in 0..nvars {
                print!(" {:.4}", popptr[i]);
            }
        }

        if ind >= popsize {
            avgf = 0.0;
            let mut minptr_idx = 0;
            let mut min_worst = (pop1[nvars], 0);

            for i in 0..popsize {
                let dtemp = pop1[i * dim + nvars];
                avgf += dtemp;
                if dtemp < min_worst.0 {
                    min_worst = (dtemp, i);
                    minptr_idx = i;
                }
            }

            if value > min_worst.0 {
                pop1[minptr_idx * dim..(minptr_idx + 1) * dim].copy_from_slice(popptr);
                avgf += value - min_worst.0;
            }
        }
    }

    if let Some(sb) = stoc_bias {
        sb.collect(false);
    }

    // Find best individual in initial population
    let mut ibest = 0;
    let mut best_val = pop1[nvars];
    for ind in 1..popsize {
        let val = pop1[ind * dim + nvars];
        if val > best_val {
            best_val = val;
            ibest = ind;
        }
    }

    // Main evolutionary loop
    let mut old_gen = &mut pop1;
    let mut new_gen = &mut pop2;
    let mut bad_generations = 0;

    for generation in 1..usize::MAX {
        worstf = 1.0e60;
        avgf = 0.0;
        let mut improved = false;

        for ind in 0..popsize {
            let parent1 = &old_gen[ind * dim..(ind + 1) * dim];
            let dest_ptr = &mut new_gen[ind * dim..(ind + 1) * dim];

            // Generate three different random indices
            let mut i = (unifrand() * popsize as f64) as usize;
            while i >= popsize || i == ind {
                i = (unifrand() * popsize as f64) as usize;
            }

            let mut j = (unifrand() * popsize as f64) as usize;
            while j >= popsize || j == ind || j == i {
                j = (unifrand() * popsize as f64) as usize;
            }

            let mut k = (unifrand() * popsize as f64) as usize;
            while k >= popsize || k == ind || k == i || k == j {
                k = (unifrand() * popsize as f64) as usize;
            }

            let parent2 = &old_gen[i * dim..(i + 1) * dim];
            let diff1 = &old_gen[j * dim..(j + 1) * dim];
            let diff2 = &old_gen[k * dim..(k + 1) * dim];

            // Pick a starting parameter
            let mut j = (unifrand() * nvars as f64) as usize % nvars;
            let mut used_mutated_parameter = false;

            for _ in 0..nvars {
                if !used_mutated_parameter || unifrand() < pcross {
                    dest_ptr[j] = parent2[j] + mutate_dev * (diff1[j] - diff2[j]);
                    used_mutated_parameter = true;
                } else {
                    dest_ptr[j] = parent1[j];
                }
                j = (j + 1) % nvars;
            }

            ensure_legal(nvars, nints, low_bounds, high_bounds, dest_ptr);

            let value = criter(dest_ptr, local_mintrades);

            if value > parent1[nvars] {
                dest_ptr[nvars] = value;
                if value > grand_best {
                    grand_best = value;
                    best.copy_from_slice(dest_ptr);
                    ibest = ind;
                    improved = true;
                }
            } else {
                dest_ptr.copy_from_slice(parent1);
            }

            if pclimb > 0.0 && ((ind == ibest) || unifrand() < pclimb) {
                // Hill climbing step would go here
                // This requires passing criterion function to the local optimization
                // Implementation details depend on your specific needs
            }

            let final_value = dest_ptr[nvars];
            if final_value < worstf {
                worstf = final_value;
            }
            avgf += final_value;
        }

        if print_progress {
            println!(
                "\nGen {} Best={:.4} Worst={:.4} Avg={:.4}",
                generation,
                grand_best,
                worstf,
                avgf / popsize as f64
            );
            for i in 0..nvars {
                print!(" {:.4}", best[i]);
            }
        }

        if !improved {
            bad_generations += 1;
            if bad_generations > max_bad_gen {
                break;
            }
        } else {
            bad_generations = 0;
        }

        // Swap generations
        std::mem::swap(&mut old_gen, &mut new_gen);
    }

    params[0..dim].copy_from_slice(&best);
    ret_code
}

fn goto_finished(pop1: &mut Vec<f64>, pop2: &mut Vec<f64>) {
    // Cleanup is automatic with Vec in Rust
}

// Trait for StocBias if needed
pub trait StocBiasTrait {
    fn collect(&mut self, enable: bool);
}

// Placeholder implementations for functions that would be imported
pub fn glob_max<F>(
    _low: f64,
    _high: f64,
    _npts: usize,
    _log_space: bool,
    _c_func: F,
) -> (f64, f64, f64, f64, f64, f64)
where
    F: Fn(f64) -> f64,
{
    // Implementation would go here
    (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
}

pub fn brentmax<F>(_nits: usize, _tol: f64, _step: f64, _c_func: F, _x1: &mut f64, _x2: &mut f64, _x3: &mut f64, _y2: f64)
where
    F: Fn(f64) -> f64,
{
    // Implementation would go here
}

pub fn paramcor(_popsize: usize, _nvars: usize, _gen: &[f64]) -> bool {
    // Implementation would go here
    false
}