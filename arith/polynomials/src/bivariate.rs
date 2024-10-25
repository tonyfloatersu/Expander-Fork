mod bi_fft;

use arith::{FFTField, Field};
use ark_std::{end_timer, start_timer};
use bi_fft::bi_fft_in_place;
use itertools::Itertools;
use rand::RngCore;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::powers_of_field_elements;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BivariatePolynomial<F> {
    pub coefficients: Vec<F>,
    pub degree_0: usize,
    pub degree_1: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BivariateLagrangePolynomial<F> {
    pub coefficients: Vec<F>,
    pub degree_0: usize,
    pub degree_1: usize,
}

impl<F: FFTField> BivariatePolynomial<F> {
    #[inline]
    pub fn new(coefficients: Vec<F>, degree_0: usize, degree_1: usize) -> Self {
        assert_eq!(coefficients.len(), degree_0 * degree_1);
        Self {
            coefficients,
            degree_0,
            degree_1,
        }
    }

    pub fn random(mut rng: impl RngCore, degree_0: usize, degree_1: usize) -> Self {
        let coefficients = (0..degree_0 * degree_1)
            .map(|_| F::random_unsafe(&mut rng))
            .collect();
        Self::new(coefficients, degree_0, degree_1)
    }

    /// evaluate the polynomial at (x, y)
    pub fn evaluate(&self, x: &F, y: &F) -> F {
        let x_power = powers_of_field_elements(x, self.degree_0);
        let y_power = powers_of_field_elements(y, self.degree_1);

        self.coefficients
            .chunks_exact(self.degree_0)
            .zip(y_power.iter())
            .fold(F::ZERO, |acc, (chunk, y_i)| {
                acc + chunk
                    .iter()
                    .zip(x_power.iter())
                    .fold(F::ZERO, |acc, (c, x_i)| acc + *c * *x_i)
                    * y_i
            })
    }

    /// evaluate the polynomial at y, return a univariate polynomial in x
    pub fn evaluate_at_y(&self, y: &F) -> Vec<F> {
        let mut f_x_b = self.coefficients[0..self.degree_0].to_vec();
        let powers_of_b = powers_of_field_elements(y, self.degree_1);
        powers_of_b
            .iter()
            .zip_eq(self.coefficients.chunks_exact(self.degree_0))
            .skip(1)
            .for_each(|(bi, chunk_i)| {
                f_x_b
                    .iter_mut()
                    .zip(chunk_i.iter())
                    .for_each(|(f, c)| *f += *c * *bi)
            });

        f_x_b
    }

    /// same as interpolate but slower.
    pub fn evaluate_at_roots(&self) -> Vec<F> {
        let timer = start_timer!(|| format!(
            "Lagrange coefficients of degree {} {}",
            self.degree_0, self.degree_1
        ));

        // roots of unity for supported_n and supported_m
        let (omega_0, omega_1) = {
            let omega = F::root_of_unity();
            let omega_0 = omega.exp((1 << F::TWO_ADICITY) / self.degree_0 as u128);
            let omega_1 = omega.exp((1 << F::TWO_ADICITY) / self.degree_1 as u128);

            assert!(
                omega_0.exp(self.degree_0 as u128) == F::ONE,
                "omega_0 is not root of unity for supported_n"
            );
            assert!(
                omega_1.exp(self.degree_1 as u128) == F::ONE,
                "omega_1 is not root of unity for supported_m"
            );
            (omega_0, omega_1)
        };
        let powers_of_omega_0 = powers_of_field_elements(&omega_0, self.degree_0);
        let powers_of_omega_1 = powers_of_field_elements(&omega_1, self.degree_1);

        let mut res = vec![];
        for omega_1_power in powers_of_omega_1.iter() {
            for omega_0_power in powers_of_omega_0.iter() {
                res.push(self.evaluate(omega_0_power, omega_1_power));
            }
        }
        end_timer!(timer);
        res
    }

    /// interpolate the polynomial over the roots via bi-variate FFT
    pub fn interpolate(&self) -> Vec<F> {
        let timer = start_timer!(|| format!(
            "Lagrange coefficients of degree {} {}",
            self.degree_0, self.degree_1
        ));

        let mut coeff = self.coefficients.clone();
        bi_fft_in_place(&mut coeff, self.degree_0, self.degree_1);
        end_timer!(timer);
        coeff
    }
}

/// For a point x, compute the coefficients of Lagrange polynomial L_{i}(x) at x, given the roots.
/// `L_{i}(x) = \prod_{j \neq i} \frac{x - r_j}{r_i - r_j}`
pub(crate) fn lagrange_coefficients<F: Field + Send + Sync>(roots: &[F], points: &F) -> Vec<F> {
    roots
        .par_iter()
        .enumerate()
        .map(|(i, _)| {
            let mut numerator = F::ONE;
            let mut denominator = F::ONE;
            for j in 0..roots.len() {
                if i == j {
                    continue;
                }
                numerator *= roots[j] - points;
                denominator *= roots[j] - roots[i];
            }
            numerator * denominator.inv().unwrap()
        })
        .collect()
}

/// Compute poly / (x-point) using univariate division
pub(crate) fn univariate_quotient<F: Field>(poly: &[F], point: &F) -> Vec<F> {
    let timer = start_timer!(|| format!("Univariate quotient of degree {}", poly.len()));
    let mut dividend_coeff = poly.to_vec();
    let divisor = [-*point, F::from(1u32)];
    let mut coefficients = vec![];

    let mut dividend_pos = dividend_coeff.len() - 1;
    let divisor_pos = divisor.len() - 1;
    let mut difference = dividend_pos as isize - divisor_pos as isize;

    while difference >= 0 {
        let term_quotient = dividend_coeff[dividend_pos] * divisor[divisor_pos].inv().unwrap();
        coefficients.push(term_quotient);

        for i in (0..=divisor_pos).rev() {
            let difference = difference as usize;
            let x = divisor[i];
            let y = x * term_quotient;
            let z = dividend_coeff[difference + i];
            dividend_coeff[difference + i] = z - y;
        }

        dividend_pos -= 1;
        difference -= 1;
    }
    coefficients.reverse();
    coefficients.push(F::from(0u32));
    end_timer!(timer);
    coefficients
}

impl<F: Field> BivariateLagrangePolynomial<F> {
    fn new(coeffs: Vec<F>, degree_0: usize, degree_1: usize) -> Self {
        assert_eq!(coeffs.len(), degree_0 * degree_1);
        Self {
            coefficients: coeffs,
            degree_0,
            degree_1,
        }
    }
}

impl<F: FFTField> From<BivariatePolynomial<F>> for BivariateLagrangePolynomial<F> {
    fn from(poly: BivariatePolynomial<F>) -> Self {
        Self::from(&poly)
    }
}

impl<F: FFTField> From<&BivariatePolynomial<F>> for BivariateLagrangePolynomial<F> {
    fn from(poly: &BivariatePolynomial<F>) -> Self {
        let coeffs = poly.interpolate();
        BivariateLagrangePolynomial::new(coeffs, poly.degree_0, poly.degree_1)
    }
}

impl<F: FFTField> BivariateLagrangePolynomial<F> {
    /// construct a bivariate lagrange polynomial from a monomial f(y) = y - b
    pub(crate) fn from_y_monomial(b: &F, n: usize, m: usize) -> Self {
        // roots of unity for supported_n and supported_m
        let omega_1 = {
            let omega = F::root_of_unity();
            omega.exp((1 << F::TWO_ADICITY) / m as u128)
        };
        let mut coeffs = vec![F::ZERO; n * m];
        for i in 0..m {
            let element = omega_1.exp(i as u128) - *b;
            coeffs[i * n..(i + 1) * n].copy_from_slice(vec![element; n].as_slice());
        }
        BivariateLagrangePolynomial::new(coeffs, n, m)
    }
}