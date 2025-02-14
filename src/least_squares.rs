use nalgebra::{DMatrix, DVector};
use rand::Rng;

// Function to solve a least squares problem given matrix A and vector b
pub fn solve_least_squares_given(a: &DMatrix<f64>, b: &DVector<f64>) -> DVector<f64> {
    let a_transpose = a.transpose();
    let ata = a_transpose.clone() * a;
    let atb = a_transpose * b;

    // Return the solution vector x
    ata.svd(true, true).solve(&atb, 1e-7).unwrap()
}

// Function to solve a random least squares problem
pub fn solve_least_squares() -> DVector<f64> {
    let mut rng = rand::thread_rng();

    // Create a random matrix A (10x5) and vector b (10x1)
    let a_data: Vec<f64> = (0..10 * 5).map(|_| rng.gen_range(-10.0..10.0)).collect();
    let a = DMatrix::from_vec(10, 5, a_data);

    let b_data: Vec<f64> = (0..10).map(|_| rng.gen_range(-10.0..10.0)).collect();
    let b = DVector::from_vec(b_data);

    // Solve the least squares problem Ax = b
    solve_least_squares_given(&a, &b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_solve_least_squares_given() {
        // Generate a fixed matrix and vector for testing
        let a = DMatrix::from_row_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let b = DVector::from_row_slice(&[1.0, 2.0]);

        // Solve the least squares problem
        let result = solve_least_squares_given(&a, &b);

        // Expected result
        let expected_result = DVector::from_row_slice(&[-0.0, 0.5]);

        // Check that the computed result is close to the expected result
        for (computed, expected) in result.iter().zip(expected_result.iter()) {
            assert_relative_eq!(computed, expected, epsilon = 1e-7);
        }
    }

    #[test]
    fn test_solve_least_squares() {
        let result = solve_least_squares();
        assert_eq!(result.len(), 5);
    }
}
