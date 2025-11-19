/****************************************************************************/
/*                                                                          */
/*                           EVEC_RS                                        */
/*                                                                          */
/*   Compute eigenvalues and vectors of real symmetric matrix               */
/*                                                                          */
/****************************************************************************/

use std::f64;

/*
   The input matrix is mat_in.  It is not touched.  The upper minor triangle
   of it is ignored, and hence may be garbage.  Its column dimension is n.
   The vect matrix must be supplied, even if the eigenvectors are not computed.
   The eigenvectors, if computed, are output in vect, which has column dimension n.
   The calling program may use the same matrix for mat_in and vect,
   in which case the input is simply replaced.
   The eigenvalues are output in eval.  Workv is a double work vector n long.
   This returns the number of eigenvalues which could not be computed,
   which is virtually always 0.  I've exhaustively tested this routine and
   never seen it return a nonzero value.
*/

pub fn evec_rs(
    mat_in: &[f64],
    n: usize,
    find_vec: bool,
    vect: &mut [f64],
    eval: &mut [f64],
    workv: &mut [f64],
) -> usize {
    // Compzero is an accuracy versus speed tradeoff.  The algorithm is most accurate when compzero=0.
    // But by letting 'zero' be a very small positive number, we can take some early loop exits
    // with very little penalty, insignificant most of the time.
    let compzero = 1.0e-16;
    // let compzero = 0.0;

    // Eps is used only for splitting a large matrix into two smaller matrices at a 'zero' diagonal,
    // greatly speeding operation.  But if the diagonal is not quite zero, this does introduce a tiny,
    // usually insignificant, error.
    // The algorithm is most accurate when eps=0, but very small values are fine for most work.
    let eps = 1.0e-12;
    // let eps = 0.0;

    /* copy lower triangle of input to output. */
    for i in 0..n {
        for j in 0..=i {
            vect[i * n + j] = mat_in[i * n + j];
        }
    }

    /*
    ------------------------------------------------------------------------------

       This section converts the matrix (now in vect) to tri-diagonal form
       using Householder's method.  It is done backwards; The last row is done
       first. The subdiagonal is saved in workv as it is found.

    ------------------------------------------------------------------------------
    */
    for irow in (1..n).rev() {
        let irowm1 = irow - 1;
        let mut h = 0.0;

        /* We can improve computational accuracy by scaling the row. */
        let mut scale = 0.0;
        for i in 0..=irowm1 {
            /* do left of diag only */
            scale += vect[irow * n + i].abs();
        }

        /* Avoid a lot of work if this row already tri-diagonal */
        if scale < compzero || irow == 1 {
            workv[irow] = vect[irow * n + irowm1];
        } else {
            /*  Do actual scaling (left of diag only).  Cumulate sum squares */
            for i in 0..=irowm1 {
                let x = vect[irow * n + i] / scale;
                vect[irow * n + i] = x;
                h += x * x;
            }

            /*  The 'U' vector of the literature is the row vector except that
                its first element (f) has the length of the vector (sqrt(h))
                either added or subtracted (g), whichever gives the largest
                absolute value. */
            let f = vect[irow * n + irowm1];
            let g = if f > 0.0 { -h.sqrt() } else { h.sqrt() };
            workv[irow] = g * scale; /* subdiagonal compensated for scaling */

            h -= f * g;
            vect[irow * n + irowm1] = f - g;

            /* Prepare to reduce vect.  Use upper triangle for storage. */

            let mut f = 0.0;
            for j in 0..=irowm1 {
                if find_vec {
                    vect[j * n + irow] = vect[irow * n + j] / h;
                }

                /* Form element of A * U */
                let mut g = 0.0;
                for k in 0..=j {
                    g += vect[j * n + k] * vect[irow * n + k];
                }
                if j < irowm1 {
                    for k in (j + 1)..=irowm1 {
                        g += vect[k * n + j] * vect[irow * n + k];
                    }
                }

                /* Compute an element of P.  Use the positions in workv below
                   those already determined subdiagonals as work areas. */
                workv[j] = g / h;
                f += workv[j] * vect[irow * n + j];
            } /* for f=0.0  j=0  */

            /* Reduce A such that all elements of row irow are zero except the
               diagonal and the element to its left (ignoring symmetric
               elements).  Naturally we need not compute those zeroes.  Just
               modify the rows above irow.  */
            let hh = f / (h + h);
            for j in 0..=irowm1 {
                let f = vect[irow * n + j];
                let g = workv[j] - hh * f;
                workv[j] = g;
                for k in 0..=j {
                    vect[j * n + k] -= f * workv[k] + g * vect[irow * n + k];
                }
            }
        } /*  else scale<compzero  */

        /* We are done with this row!  Save h in eval.  */
        eval[irow] = h;
    } /* for irow=n-1 */

    /*
    ------------------------------------------------------------------------------

       We are nearly done with the tri-diagonalization.  The transformation
       itself has been done to the matrix and the subdiagonals are stored in
       workv.  H for each row is in eval.  Complete the job by recovering
       the transformation matrix and diagonal.

    ------------------------------------------------------------------------------
    */
    workv[0] = 0.0;
    if find_vec {
        eval[0] = 0.0;
        for irow in 0..n {
            let irowm1 = irow as i32 - 1;
            if eval[irow].abs() > compzero {
                for j in 0..=(irow - 1) {
                    let mut g = 0.0;
                    for k in 0..=(irow - 1) {
                        g += vect[irow * n + k] * vect[k * n + j];
                    }
                    for k in 0..=(irow - 1) {
                        vect[k * n + j] -= g * vect[k * n + irow];
                    }
                }
            }
            /*  Recover diagonal and zero matrix elements which are truly zero
                but were not computed.  */
            eval[irow] = vect[irow * n + irow];
            vect[irow * n + irow] = 1.0;
            for j in 0..=(irow - 1) {
                vect[irow * n + j] = 0.0;
                vect[j * n + irow] = 0.0;
            }
        } /*  for  irow=0  */
    } else {
        for irow in 0..n {
            eval[irow] = vect[irow * n + irow];
        }
    }

    /*
    ------------------------------------------------------------------------------

       The matrix is now completely tridiagonal.  The diagonal is in eval and
       the subdiagonal still in workv.  The transformation matrix is in vect.
       Now we use the QL method to find the eigenvalues and vectors.

    ------------------------------------------------------------------------------
    */
    if n == 1 {
        return 0;
    }

    /*  The first element of the subdiagonal does not exist.  Shift workv.  */
    for i in 1..n {
        workv[i - 1] = workv[i];
    }
    workv[n - 1] = 0.0;

    let mut shift = 0.0;
    let mut b = 0.0;

    /*
       This is the main loop.  The rotation isolates one eigenvalue at a time.
    */
    for ival in 0..n {
        let mut iercnt = 0; /* count tries for this eigenvalue  */

        /*  It is always nice to be able to split a matrix into two parts
            in order to reduce it from one big problem to two smaller ones.
            We use 'b' as a computational zero.  If a subdiagonal element
            is smaller than b we have a split.  */
        let mut h = eps * (eval[ival].abs() + workv[ival].abs());
        h = if h > compzero { h } else { compzero }; /* needed in some cases */
        b = if b > h { b } else { h };

        /* Recall we set workv[n-1]=0.0  This loop at least finds that.  */
        let mut msplit = ival;
        for m in ival..n {
            if workv[m].abs() <= b {
                msplit = m;
                break;
            }
            msplit = m + 1;
        }

        /*  We might luck out.  If the first subdiagonal is 'zero' then
            the corresponding diagonal is an eigenvalue.  Thus we only need to
            do the computation if that is not the case.  */
        if msplit > ival {
            loop {
                iercnt += 1;
                if iercnt > 100 {
                    /* avoid useless repetition */
                    return n - ival;
                }

                /*  Before transforming we shift all eigenvalues by a constant to
                    accelerate convergence.  Now shift by an additional h for
                    this one.  */
                let ivalp1 = ival + 1;
                let g = eval[ival];
                let p = (eval[ivalp1] - g) / (2.0 * workv[ival]); /* tricky denom */
                let r = (p * p + 1.0).sqrt();
                eval[ival] = workv[ival] / (p + if p > 0.0 { r } else { -r });

                let h = g - eval[ival];
                /* We just shifted ival'th.  Do same for others.  */
                for i in ivalp1..n {
                    /* above 'if' insures ivalp1<n */
                    eval[i] -= h;
                }
                shift += h;

                /* This is the actual QL transform */
                let mut p = eval[msplit];
                let mut cosine = 1.0;
                let mut sine = 0.0;

                /* Only rotate between last eigenvalue computed and split point */
                for i in (ival..msplit).rev() {
                    let g = cosine * workv[i];
                    let h = cosine * p;

                    if p.abs() >= workv[i].abs() {
                        cosine = workv[i] / p;
                        let r = (cosine * cosine + 1.0).sqrt();
                        workv[i + 1] = sine * p * r;
                        sine = cosine / r;
                        cosine = 1.0 / r;
                    } else {
                        cosine = p / workv[i];
                        let r = (cosine * cosine + 1.0).sqrt();
                        workv[i + 1] = sine * workv[i] * r;
                        sine = 1.0 / r;
                        cosine = cosine * sine;
                    }

                    p = cosine * eval[i] - sine * g;
                    eval[i + 1] = h + sine * (cosine * g + sine * eval[i]);

                    /* now we must transform vect the same way, so that we get
                       the eigenvector of the original matrix.  Note that
                       previous vectors are untouched.  */
                    if find_vec {
                        for k in 0..n {
                            let h = vect[k * n + i + 1];
                            vect[k * n + i + 1] = sine * vect[k * n + i] + cosine * h;
                            vect[k * n + i] = cosine * vect[k * n + i] - sine * h;
                        }
                    }
                } /*  for i=msplit-1  */

                /*  A tentative eigenvalue has been found.  Save it.  */
                eval[ival] = cosine * p;
                workv[ival] = sine * p;

                /*  Repeat until satisfactory accuracy is achieved.  */
                if !(workv[ival].abs() > b) {
                    break;
                }
            }
        } /*  if  msplit > ival  */

        /*  We have an eigenvalue.  Compensate for shifting.  */
        eval[ival] += shift;
    } /*  for ival=0  */

    /*
    ------------------------------------------------------------------------------

       This is it.  We are all done.  However, many programs prefer for the
       eigenvalues (and corresponding vectors!) to be sorted in decreasing
       order.  Do this now.  Then flip signs in any column which has more
       negatives than positives.  This is appreciated during interpretation.

    ------------------------------------------------------------------------------
    */

    for i in 1..n {
        let im1 = i - 1;
        let mut ibig = im1;
        let mut big = eval[im1];

        /*  Find largest eval beyond im1  */
        for j in i..n {
            let x = eval[j];
            if x > big {
                big = x;
                ibig = j;
            }
        }

        if ibig != im1 {
            /* swap */
            eval[ibig] = eval[im1];
            eval[im1] = big;
            if find_vec {
                for j in 0..n {
                    let x = vect[j * n + im1];
                    let p = vect[j * n + ibig]; /* using p due to compiler error */
                    vect[j * n + im1] = p;
                    vect[j * n + ibig] = x;
                }
            }
        }
    }

    if find_vec {
        for i in 0..n {
            let mut k = 0;
            for j in 0..n {
                if vect[j * n + i] < 0.0 {
                    k += 1;
                }
            }
            if 2 * k > n {
                for j in 0..n {
                    vect[j * n + i] *= -1.0;
                }
            }
        }
    }

    0
}