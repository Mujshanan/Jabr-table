//! Reproducible, dependency-isolated comparison of the native dense matrix,
//! faer, and nalgebra for representative FEM element matrix products.
//!
//! Run with: cargo bench -p jabr-table-core --bench matrix_backend_compare
use std::hint::black_box;
use std::time::{Duration, Instant};

use faer::Mat;
use jabr_table_core::matrix::DenseMatrix;
use nalgebra::{DMatrix, SMatrix};

fn values(size: usize, offset: f64) -> Vec<f64> {
    (0..size * size)
        .map(|index| ((index * 17 % 31) as f64 + offset) / 31.0)
        .collect()
}

fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn measure(mut operation: impl FnMut(), iterations: usize) -> Duration {
    let mut samples = Vec::with_capacity(5);
    for _ in 0..5 {
        let start = Instant::now();
        for _ in 0..iterations {
            operation();
        }
        samples.push(start.elapsed() / iterations as u32);
    }
    median(samples)
}

fn report(label: &str, duration: Duration) {
    println!("{label:>20} {:>12.2} ns/op", duration.as_nanos() as f64);
}

fn assert_close(actual: f64, expected: f64) {
    let tolerance = 1.0e-9 * expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= tolerance,
        "backend mismatch: actual={actual}, expected={expected}"
    );
}

fn benchmark_size(size: usize, iterations: usize) {
    let left_values = values(size, 1.0);
    let right_values = values(size, 3.0);

    let native_left = DenseMatrix::new(size, size, left_values.clone()).unwrap();
    let native_right = DenseMatrix::new(size, size, right_values.clone()).unwrap();
    let faer_left = Mat::from_fn(size, size, |row, column| left_values[row * size + column]);
    let faer_right = Mat::from_fn(size, size, |row, column| right_values[row * size + column]);
    let nalgebra_left =
        DMatrix::from_fn(size, size, |row, column| left_values[row * size + column]);
    let nalgebra_right =
        DMatrix::from_fn(size, size, |row, column| right_values[row * size + column]);

    // Verify numerical equivalence before recording any timings.
    let native_result = native_left.multiply(&native_right).unwrap();
    let faer_result = &faer_left * &faer_right;
    let nalgebra_result = &nalgebra_left * &nalgebra_right;
    for row in 0..size {
        for column in 0..size {
            let expected = native_result.get(row, column).unwrap();
            assert_close(faer_result[(row, column)], expected);
            assert_close(nalgebra_result[(row, column)], expected);
        }
    }

    println!("\n{size}x{size} matrix ({iterations} iterations/sample; median of 5)");
    report(
        "native DenseMatrix",
        measure(
            || {
                let result = black_box(&native_left)
                    .multiply(black_box(&native_right))
                    .unwrap();
                black_box(result.get(0, 0));
            },
            iterations,
        ),
    );
    report(
        "faer",
        measure(
            || {
                let result = black_box(&faer_left) * black_box(&faer_right);
                black_box(result[(0, 0)]);
            },
            iterations,
        ),
    );
    report(
        "nalgebra DMatrix",
        measure(
            || {
                let result = black_box(&nalgebra_left) * black_box(&nalgebra_right);
                black_box(result[(0, 0)]);
            },
            iterations,
        ),
    );

    if size == 4 {
        let fixed_left =
            SMatrix::<f64, 4, 4>::from_fn(|row, column| left_values[row * size + column]);
        let fixed_right =
            SMatrix::<f64, 4, 4>::from_fn(|row, column| right_values[row * size + column]);
        report(
            "nalgebra SMatrix",
            measure(
                || {
                    let result = black_box(&fixed_left) * black_box(&fixed_right);
                    black_box(result[(0, 0)]);
                },
                iterations,
            ),
        );
    }
}

fn main() {
    println!("Jabr-table dense matrix backend comparison");
    println!("Single-threaded faer build; correctness checked before timing.");
    for (size, iterations) in [(4, 2_000), (6, 2_000), (12, 500), (24, 100), (64, 10)] {
        benchmark_size(size, iterations);
    }
}
