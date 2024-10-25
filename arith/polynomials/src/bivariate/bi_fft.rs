//! Two layer FFT and its inverse

use arith::FFTField;
use ark_std::log2;

use crate::{best_fft, primitive_root_of_unity};

#[inline]
fn bitreverse(mut n: usize, l: usize) -> usize {
    let mut r = 0;
    for _ in 0..l {
        r = (r << 1) | (n & 1);
        n >>= 1;
    }
    r
}

#[inline]
fn deep_swap_chunks<F: Clone + Copy>(a: &mut [&mut [F]], rk: usize, k: usize) {
    let x = a[k].as_mut_ptr();
    let y = a[rk].as_mut_ptr();
    unsafe {
        // is there a
        for i in 0..a[k].len() {
            std::ptr::swap(x.add(i), y.add(i));
        }
    }
}

#[inline]
fn assign_vec<F: FFTField>(a: &mut [F], b: &[F]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a = *b);
}

#[inline]
fn add_assign_vec<F: FFTField>(a: &mut [F], b: &[F]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

#[inline]
fn sub_assign_vec<F: FFTField>(a: &mut [F], b: &[F]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a -= b);
}

#[inline]
fn mul_assign_vec<F: FFTField>(a: &mut [F], b: &F) {
    a.iter_mut().for_each(|a| *a *= b);
}

// code copied from Halo2curves with adaption to vectors
//
//
/// Performs a radix-$2$ Fast-Fourier Transformation (FFT) on a vector of size
/// $n = 2^k$, when provided `log_n` = $k$ and an element of multiplicative
/// order $n$ called `omega` ($\omega$). The result is that the vector `a`, when
/// interpreted as the coefficients of a polynomial of degree $n - 1$, is
/// transformed into the evaluations of this polynomial at each of the $n$
/// distinct powers of $\omega$. This transformation is invertible by providing
/// $\omega^{-1}$ in place of $\omega$ and dividing each resulting field element
/// by $n$.
///
/// This will use multithreading if beneficial.
pub fn best_fft_vec_in_place<F: FFTField>(a: &mut [F], omega: F, log_n: u32, log_m: u32) {
    let threads = rayon::current_num_threads();
    let log_threads = threads.ilog2();
    let mn = a.len();
    let m = 1 << log_m;
    let n = 1 << log_n;
    assert_eq!(mn, 1 << (log_n + log_m));

    let mut a_vec_ptrs = a.chunks_exact_mut(n).collect::<Vec<_>>();

    for k in 0..m {
        let rk = bitreverse(k, log_m as usize);

        if k < rk {
            // `a_vec_ptrs.swap(rk, k)` doesn't work here as it only swaps the pointers not the actual data
            deep_swap_chunks(&mut a_vec_ptrs, rk, k);
        }
    }

    // precompute twiddle factors
    let twiddles: Vec<_> = (0..(m / 2))
        .scan(F::ONE, |w, _| {
            let tw = *w;
            *w *= &omega;
            Some(tw)
        })
        .collect();

    if log_n <= log_threads {
        let mut chunk = 2_usize;
        let mut twiddle_chunk = m / 2;
        for _ in 0..log_m {
            a_vec_ptrs.chunks_mut(chunk).for_each(|coeffs| {
                let (left, right) = coeffs.split_at_mut(chunk / 2);

                // case when twiddle factor is one
                let (a, left) = left.split_at_mut(1);
                let (b, right) = right.split_at_mut(1);
                let t = b[0].to_vec();

                // compute the following in vectors
                // b[0] = a[0];
                // a[0] += &t;
                // b[0] -= &t;
                assign_vec(b[0], a[0]);
                add_assign_vec(a[0], &t);
                sub_assign_vec(b[0], &t);

                left.iter_mut()
                    .zip(right.iter_mut())
                    .enumerate()
                    .for_each(|(i, (a, b))| {
                        let mut t = b.to_vec();

                        // compute the following in vectors
                        // t *= &twiddles[(i + 1) * twiddle_chunk];
                        // *b = *a;
                        // *a += &t;
                        // *b -= &t;
                        mul_assign_vec(&mut t, &twiddles[(i + 1) * twiddle_chunk]);
                        assign_vec(b, a);
                        add_assign_vec(a, &t);
                        sub_assign_vec(b, &t);
                    });
            });
            chunk *= 2;
            twiddle_chunk /= 2;
        }
    } else {
        recursive_butterfly_arithmetic(&mut a_vec_ptrs, m, 1, &twiddles)
    }
}

/// This perform recursive butterfly arithmetic
fn recursive_butterfly_arithmetic<F: FFTField>(
    a: &mut [&mut [F]],
    m: usize,
    twiddle_chunk: usize,
    twiddles: &[F],
) {
    if m == 2 {
        let t1 = a[1].to_vec();
        let t0 = a[0].to_vec();
        // compute the following in vectors
        // a[1] = a[0];
        // a[0] += &t;
        // a[1] -= &t;
        assign_vec(a[1], &t0);
        add_assign_vec(a[0], &t1);
        sub_assign_vec(a[1], &t1);
    } else {
        let (left, right) = a.split_at_mut(m / 2);
        rayon::join(
            || recursive_butterfly_arithmetic(left, m / 2, twiddle_chunk * 2, twiddles),
            || recursive_butterfly_arithmetic(right, m / 2, twiddle_chunk * 2, twiddles),
        );

        // case when twiddle factor is one
        let (a, left) = left.split_at_mut(1);
        let (b, right) = right.split_at_mut(1);
        let t = b[0].to_vec();
        // compute the following in vectors
        // b[0] = a[0];
        // a[0] += &t;
        // b[0] -= &t;
        assign_vec(b[0], a[0]);
        add_assign_vec(a[0], &t);
        sub_assign_vec(b[0], &t);

        left.iter_mut()
            .zip(right.iter_mut())
            .enumerate()
            .for_each(|(i, (a, b))| {
                let mut t = b.to_vec();
                // compute the following in vectors
                // t *= &twiddles[(i + 1) * twiddle_chunk];
                // *b = *a;
                // *a += &t;
                // *b -= &t;
                mul_assign_vec(&mut t, &twiddles[(i + 1) * twiddle_chunk]);
                assign_vec(b, a);
                add_assign_vec(a, &t);
                sub_assign_vec(b, &t);
            });
    }
}

/// Convert a polynomial in coefficient form to evaluation form using a two layer FFT
pub fn bi_fft_in_place<F: FFTField>(coeffs: &mut [F], degree_n: usize, degree_m: usize) {
    assert_eq!(coeffs.len(), degree_n * degree_m);
    assert!(degree_n.is_power_of_two());
    assert!(degree_m.is_power_of_two());

    // roots of unity for supported_n and supported_m
    let omega_0 = primitive_root_of_unity(degree_n);
    let omega_1 = primitive_root_of_unity(degree_m);

    // inner layer of FFT over variable x
    coeffs
        .chunks_exact_mut(degree_n)
        .for_each(|chunk| best_fft(chunk, omega_0, log2(degree_n)));

    // outer layer of FFT over variable y
    best_fft_vec_in_place(coeffs, omega_1, log2(degree_n), log2(degree_m));
}
