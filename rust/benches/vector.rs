//! Batch Black IV throughput.
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iv_engine::{black_price_slice, implied_volatility_slice};

fn bench_iv_slice(c: &mut Criterion) {
    const N: usize = 1024;
    let forwards = vec![100.0_f64; N];
    let strikes: Vec<f64> = (0..N)
        .map(|i| 80.0 + (i as f64) * 40.0 / N as f64)
        .collect();
    let mats = vec![1.0_f64; N];
    let vols = vec![0.2_f64; N];
    let mut prices = vec![0.0_f64; N];
    black_price_slice(&forwards, &strikes, &mats, &vols, true, &mut prices).unwrap();
    let mut out = vec![0.0_f64; N];

    c.bench_function("implied_volatility_slice_1024", |b| {
        b.iter(|| {
            implied_volatility_slice(
                black_box(&prices),
                black_box(&forwards),
                black_box(&strikes),
                black_box(&mats),
                true,
                black_box(&mut out),
            )
            .unwrap();
        });
    });
}

criterion_group!(benches, bench_iv_slice);
criterion_main!(benches);
