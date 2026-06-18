//! Configurable-degree least-squares polynomial fitting (power basis).
//!
//! Solves the normal equations `(VᵀV) c = Vᵀy` for the Vandermonde matrix `V`
//! of the sample x-values, returning ascending power-basis coefficients. Used by
//! the dense within-span segment fitter.

/// Fits a degree-`degree` polynomial to `(x, y)` samples by least squares.
/// Returns ascending power-basis coefficients, or `None` if there are fewer
/// samples than coefficients or the normal-equations matrix is singular.
pub fn fit_polynomial_lsq(samples: &[(f64, f64)], degree: usize) -> Option<Vec<f64>> {
    let n_coeffs = degree + 1;
    if samples.len() < n_coeffs {
        return None;
    }
    // Build normal-equations matrix A (n_coeffs x n_coeffs) and rhs b.
    let mut a = vec![vec![0.0f64; n_coeffs]; n_coeffs];
    let mut b = vec![0.0f64; n_coeffs];
    for &(x, y) in samples {
        let mut powers = vec![1.0f64; n_coeffs];
        for k in 1..n_coeffs {
            powers[k] = powers[k - 1] * x;
        }
        for i in 0..n_coeffs {
            b[i] += powers[i] * y;
            for j in 0..n_coeffs {
                a[i][j] += powers[i] * powers[j];
            }
        }
    }
    solve_linear_system(a, b)
}

/// Gaussian elimination with partial pivoting. Returns `None` if singular.
#[allow(clippy::needless_range_loop)]
fn solve_linear_system(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let mut pivot = col;
        for row in (col + 1)..n {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }
        if a[pivot][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in (col + 1)..n {
            let factor = a[row][col] / a[col][col];
            for k in col..n {
                a[row][k] -= factor * a[col][k];
            }
            b[row] -= factor * b[col];
        }
    }
    let mut x = vec![0.0f64; n];
    for row in (0..n).rev() {
        let mut sum = b[row];
        for k in (row + 1)..n {
            sum -= a[row][k] * x[k];
        }
        x[row] = sum / a[row][row];
    }
    if x.iter().all(|v| v.is_finite()) {
        Some(x)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_a_known_cubic_exactly() {
        // y = 1 + 2x - 3x^2 + 0.5x^3 sampled at 12 points over [0,1].
        let f = |x: f64| 1.0 + 2.0 * x - 3.0 * x * x + 0.5 * x * x * x;
        let samples: Vec<(f64, f64)> = (0..12)
            .map(|i| {
                let x = i as f64 / 11.0;
                (x, f(x))
            })
            .collect();
        let coeffs = fit_polynomial_lsq(&samples, 3).expect("fit should succeed");
        let expected = [1.0, 2.0, -3.0, 0.5];
        assert_eq!(coeffs.len(), 4);
        for (got, want) in coeffs.iter().zip(expected) {
            assert!((got - want).abs() < 1e-6, "coeff {got} vs {want}");
        }
    }

    #[test]
    fn underdetermined_returns_none() {
        let samples = [(0.0, 1.0), (1.0, 2.0)];
        assert!(fit_polynomial_lsq(&samples, 5).is_none());
    }

    #[test]
    fn fits_higher_degree_smooth_function_within_tolerance() {
        // sin over [0,1] fit at degree 8 with oversampling -> tiny residual.
        let f = |x: f64| (x * std::f64::consts::PI).sin();
        let samples: Vec<(f64, f64)> = (0..27)
            .map(|i| {
                let x = i as f64 / 26.0;
                (x, f(x))
            })
            .collect();
        let coeffs = fit_polynomial_lsq(&samples, 8).expect("fit");
        let eval = |x: f64| coeffs.iter().rev().fold(0.0, |acc, c| acc * x + c);
        for i in 0..50 {
            let x = i as f64 / 49.0;
            assert!((eval(x) - f(x)).abs() < 1e-4, "residual too large at {x}");
        }
    }
}
