use jabr_table_core::matrix::DenseMatrix;

fn matrix(rows: Vec<Vec<f64>>) -> DenseMatrix {
    DenseMatrix::from_rows(rows).expect("fixture matrix must be valid")
}

fn assert_close(actual: f64, expected: f64) {
    let tolerance = 1.0e-10 * expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual {actual} differs from expected {expected} (tolerance {tolerance})"
    );
}

fn assert_matrix_close(actual: &DenseMatrix, expected: &DenseMatrix) {
    assert_eq!(
        (actual.rows(), actual.columns()),
        (expected.rows(), expected.columns())
    );
    for (index, (actual, expected)) in actual
        .as_slice()
        .iter()
        .zip(expected.as_slice())
        .enumerate()
    {
        assert_close(*actual, *expected);
        assert!(actual.is_finite(), "entry {index} is not finite");
    }
}

#[test]
fn two_dimensional_truss_local_stiffness_fixture() {
    // Axial bar: E=200, A=0.01, L=2 => EA/L = 1.
    // DOF order: [u1, v1, u2, v2].
    let axial_operator = matrix(vec![vec![-1.0, 0.0, 1.0, 0.0]]);
    let local = axial_operator
        .transpose()
        .multiply(&axial_operator)
        .unwrap();
    let expected = matrix(vec![
        vec![1.0, 0.0, -1.0, 0.0],
        vec![0.0, 0.0, 0.0, 0.0],
        vec![-1.0, 0.0, 1.0, 0.0],
        vec![0.0, 0.0, 0.0, 0.0],
    ]);
    assert_matrix_close(&local, &expected);
    assert_eq!(
        local.multiply_vector(&[1.0, 0.0, -1.0, 0.0]).unwrap(),
        vec![2.0, 0.0, -2.0, 0.0]
    );
}

#[test]
fn two_dimensional_beam_local_stiffness_fixture() {
    // Euler-Bernoulli beam: EI=3, L=2, so EI/L^3 = 3/8.
    // DOF order: [v1, theta1, v2, theta2].
    let factor = 3.0 / 8.0;
    let expected = matrix(vec![
        vec![12.0 * factor, 12.0 * factor, -12.0 * factor, 12.0 * factor],
        vec![12.0 * factor, 16.0 * factor, -12.0 * factor, 8.0 * factor],
        vec![
            -12.0 * factor,
            -12.0 * factor,
            12.0 * factor,
            -12.0 * factor,
        ],
        vec![12.0 * factor, 8.0 * factor, -12.0 * factor, 16.0 * factor],
    ]);
    let actual = matrix(vec![
        vec![4.5, 4.5, -4.5, 4.5],
        vec![4.5, 6.0, -4.5, 3.0],
        vec![-4.5, -4.5, 4.5, -4.5],
        vec![4.5, 3.0, -4.5, 6.0],
    ]);
    assert_matrix_close(&actual, &expected);
}

#[test]
fn rotated_truss_stiffness_matches_closed_form_global_matrix() {
    // EA/L=2, theta=30 degrees. Global stiffness is
    // (EA/L) * [[c², cs, -c², -cs], [cs, s², -cs, -s²], ...].
    let c = 3.0_f64.sqrt() / 2.0;
    let s = 0.5_f64;
    let transform = matrix(vec![
        vec![c, s, 0.0, 0.0],
        vec![-s, c, 0.0, 0.0],
        vec![0.0, 0.0, c, s],
        vec![0.0, 0.0, -s, c],
    ]);
    let local = matrix(vec![
        vec![2.0, 0.0, -2.0, 0.0],
        vec![0.0, 0.0, 0.0, 0.0],
        vec![-2.0, 0.0, 2.0, 0.0],
        vec![0.0, 0.0, 0.0, 0.0],
    ]);
    let expected = matrix(vec![
        vec![1.5, 3.0_f64.sqrt() / 2.0, -1.5, -3.0_f64.sqrt() / 2.0],
        vec![3.0_f64.sqrt() / 2.0, 0.5, -3.0_f64.sqrt() / 2.0, -0.5],
        vec![-1.5, -3.0_f64.sqrt() / 2.0, 1.5, 3.0_f64.sqrt() / 2.0],
        vec![-3.0_f64.sqrt() / 2.0, -0.5, 3.0_f64.sqrt() / 2.0, 0.5],
    ]);
    let global = transform
        .transpose()
        .multiply(&local)
        .unwrap()
        .multiply(&transform)
        .unwrap();
    assert_matrix_close(&global, &expected);
    assert_matrix_close(&global, &global.transpose());
}

#[test]
fn matrix_identities_hold_for_a_range_of_rectangular_shapes() {
    for rows in 1..=5 {
        for columns in 1..=5 {
            let values: Vec<f64> = (0..rows * columns).map(|i| i as f64 + 0.25).collect();
            let a = DenseMatrix::new(rows, columns, values).unwrap();
            assert_matrix_close(&a, &a.transpose().transpose());

            let zeros = DenseMatrix::new(rows, columns, vec![0.0; rows * columns]).unwrap();
            assert_matrix_close(&a, &a.add(&zeros).unwrap());
            assert_matrix_close(&zeros, &a.subtract(&a).unwrap());
        }
    }
}

#[test]
fn arithmetic_overflow_is_reported_instead_of_returning_non_finite_entries() {
    let large = DenseMatrix::new(1, 1, vec![f64::MAX]).unwrap();
    assert!(large.add(&large).is_err());
    assert!(large.multiply(&large).is_err());
    assert!(large.multiply_vector(&[2.0]).is_err());
}
