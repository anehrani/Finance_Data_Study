/******************************************************************************/
/*                                                                            */
/*  GLOB_MAX - Check equispaced intervals to find rough global maximum        */
/*             of a univariate function                                       */
/*                                                                            */
/*  This is called with a lower and upper bound for the domain to be          */
/*  searched.  If the function is still increasing at one of these endpoints  */
/*  the search will continue beyond the specified endpoint.                   */
/*  The total interval is equally divided into npts-1 subintervals.           */
/*  These subintervals will be spaced arithmetically or logarithmically       */
/*  according to log_space.                                                   */
/*  Three points will be returned.  The center point, (x2,y2), will have      */
/*  greater function value (y2) than its neighbors.  (In pathological         */
/*  cases they may be equal.)                                                 */
/*                                                                            */
/*  If npts is input negative, that means the user is inputting f(hi) in y2.  */
/*  That sometimes saves a function evaluation.                               */
/*                                                                            */
/*  Normally it returns zero.  It returns one if user pressed ESCape before   */
/*  the minimum was found.                                                    */
/*                                                                            */
/******************************************************************************/

pub fn glob_max<F>(
    low: f64,                  // Lower limit for search
    high: f64,                 // Upper limit
    mut npts: i32,             // Number of points to try
    log_space: bool,           // Space by log?
    c_func: F,                 // Criterion function
    x1: &mut f64,
    y1: &mut f64,              // Lower X value and function there
    x2: &mut f64,
    y2: &mut f64,              // Middle (best)
    x3: &mut f64,
    y3: &mut f64,              // And upper
) -> i32
where
    F: Fn(f64) -> f64,
{
    let know_first_point = if npts < 0 {
        npts = -npts;
        true
    } else {
        false
    };

    let rate = if log_space {
        (high / low).ln() / (npts as f64 - 1.0)
    } else {
        (high - low) / (npts as f64 - 1.0)
    };

    let mut x = low;
    let mut previous = 0.0; // Avoids "use before set" compiler warnings
    let mut ibest = -1; // For proper critlim escape
    let mut turned = false; // Must know if function improved

    for i in 0..npts {
        let y = if i != 0 || !know_first_point {
            c_func(x)
        } else {
            *y2
        };

        if i == 0 || y > *y2 {
            // Keep track of best here
            ibest = i;
            *x2 = x;
            *y2 = y;
            *y1 = previous; // Function value to its left
            turned = false; // Flag that min is not yet bounded
        } else if i == (ibest + 1) {
            // Didn't improve so this point may
            *y3 = y; // be the right neighbor of the best
            turned = true; // Flag that min is bounded
        }

        previous = y; // Keep track for left neighbor of best

        if log_space {
            x *= rate.exp();
        } else {
            x += rate;
        }
    }

    /*
       At this point we have a maximum (within low,high) at (x2,y2).
       Compute x1 and x3, its neighbors.
       We already know y1 and y3 (unless the maximum is at an endpoint!).
    */

    if log_space {
        *x1 = *x2 / rate.exp();
        *x3 = *x2 * rate.exp();
    } else {
        *x1 = *x2 - rate;
        *x3 = *x2 + rate;
    }

    /*
       Normally we would now be done.  However, the careless user may have
       given us a bad x range (low,high) for the global search.
       If the function was still improving at an endpoint, bail out the
       user by continuing the search.
    */

    if !turned {
        // Must extend to the right (larger x)
        let mut rate = rate;
        loop {
            // Endless loop goes as long as necessary

            *y3 = c_func(*x3);

            if *y3 < *y2 {
                // If function decreased we are done
                break;
            }
            if (*y1 == *y2) && (*y2 == *y3) {
                // Give up if flat
                break;
            }

            *x1 = *x2; // Shift all points
            *y1 = *y2;
            *x2 = *x3;
            *y2 = *y3;

            rate *= 3.0; // Step further each time
            if log_space {
                // And advance to new frontier
                *x3 *= rate.exp();
            } else {
                *x3 += rate;
            }
        }
    } else if ibest == 0 {
        // Must extend to the left (smaller x)
        let mut rate = rate;
        loop {
            // Endless loop goes as long as necessary

            *y1 = c_func(*x1);

            if *y1 < *y2 {
                // If function decreased we are done
                break;
            }
            if (*y1 == *y2) && (*y2 == *y3) {
                // Give up if flat
                break;
            }

            *x3 = *x2; // Shift all points
            *y3 = *y2;
            *x2 = *x1;
            *y2 = *y1;

            rate *= 3.0; // Step further each time
            if log_space {
                // And advance to new frontier
                *x1 /= rate.exp();
            } else {
                *x1 -= rate;
            }
        }
    }
    0
}