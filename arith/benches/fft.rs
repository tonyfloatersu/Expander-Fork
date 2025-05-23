use std::hint::black_box;

use arith::{FFTField, Field, Fr};
use ark_std::test_rng;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn fft_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("FFT");

    const MAX_FFT_SIZE: usize = 1 << 16;
    let mut rng = test_rng();
    let mut bn254_buf: Vec<Fr> = (0..MAX_FFT_SIZE)
        .map(|_| Fr::random_unsafe(&mut rng))
        .collect();

    for group_size_bits in 9..=MAX_FFT_SIZE.ilog2() {
        group.bench_with_input(
            BenchmarkId::new(
                format!("benchmark BN254 {group_size_bits}-bits FFT in place"),
                group_size_bits,
            ),
            &group_size_bits,
            |b, group_size_bits| {
                b.iter(|| {
                    let group_size = 1 << group_size_bits;
                    black_box(Fr::fft_in_place(&mut bn254_buf[..group_size]));
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new(
                format!("benchmark BN254 {group_size_bits}-bits iFFT in place"),
                group_size_bits,
            ),
            &group_size_bits,
            |b, group_size_bits| {
                b.iter(|| {
                    let group_size = 1 << group_size_bits;
                    black_box(Fr::ifft_in_place(&mut bn254_buf[..group_size]));
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new(
                format!("benchmark BN254 {group_size_bits}-bits FFT in place by halo2 serial"),
                group_size_bits,
            ),
            &group_size_bits,
            |b, group_size_bits| {
                b.iter(|| {
                    let group_size = 1 << group_size_bits;
                    let omega = Fr::two_adic_generator(*group_size_bits as usize);
                    black_box(halo2_serial_fft(
                        &mut bn254_buf[..group_size],
                        omega,
                        *group_size_bits,
                    ));
                })
            },
        );
    }
}

pub fn halo2_serial_fft<F: FFTField>(a: &mut [F], omega: F, log_n: u32) {
    fn bitreverse(mut n: usize, l: usize) -> usize {
        let mut r = 0;
        for _ in 0..l {
            r = (r << 1) | (n & 1);
            n >>= 1;
        }
        r
    }

    let n = a.len();
    assert_eq!(n, 1 << log_n);

    for k in 0..n {
        let rk = bitreverse(k, log_n as usize);
        if k < rk {
            a.swap(rk, k);
        }
    }

    // precompute twiddle factors
    let twiddles: Vec<_> = (0..(n / 2))
        .scan(F::ONE, |w, _| {
            let tw = *w;
            *w *= &omega;
            Some(tw)
        })
        .collect();

    let mut chunk = 2_usize;
    let mut twiddle_chunk = n / 2;
    for _ in 0..log_n {
        a.chunks_mut(chunk).for_each(|coeffs| {
            let (left, right) = coeffs.split_at_mut(chunk / 2);

            // case when twiddle factor is one
            let (a, left) = left.split_at_mut(1);
            let (b, right) = right.split_at_mut(1);
            let t = b[0];
            b[0] = a[0];
            a[0] += &t;
            b[0] -= &t;

            left.iter_mut()
                .zip(right.iter_mut())
                .enumerate()
                .for_each(|(i, (a, b))| {
                    let mut t = *b;
                    t *= &twiddles[(i + 1) * twiddle_chunk];
                    *b = *a;
                    *a += &t;
                    *b -= &t;
                });
        });
        chunk *= 2;
        twiddle_chunk /= 2;
    }
}

criterion_group!(benches, fft_benchmark);
criterion_main!(benches);
