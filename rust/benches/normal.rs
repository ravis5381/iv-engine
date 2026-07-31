//! Criterion benchmarks for Standard Normal PDF / CDF.
//!
//! Run with:
//! ```bash
//! cargo bench -p iv-engine --bench normal
//! ```

#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use iv_engine::normal::{norm_cdf, norm_cdf_c, norm_pdf};

fn bench_norm_pdf(c: &mut Criterion) {
    let mut group = c.benchmark_group("norm_pdf");
    for &x in &[0.0_f64, 1.0, 2.5, 8.0, -3.0] {
        group.bench_with_input(BenchmarkId::from_parameter(x), &x, |b, &x| {
            b.iter(|| norm_pdf(black_box(x)));
        });
    }
    group.finish();
}

fn bench_norm_cdf(c: &mut Criterion) {
    let mut group = c.benchmark_group("norm_cdf");
    for &x in &[0.0_f64, 1.0, 2.5, 8.0, -3.0] {
        group.bench_with_input(BenchmarkId::from_parameter(x), &x, |b, &x| {
            b.iter(|| norm_cdf(black_box(x)));
        });
    }
    group.finish();
}

fn bench_norm_cdf_c(c: &mut Criterion) {
    let mut group = c.benchmark_group("norm_cdf_c");
    for &x in &[0.0_f64, 1.0, 2.5, 8.0, -3.0] {
        group.bench_with_input(BenchmarkId::from_parameter(x), &x, |b, &x| {
            b.iter(|| norm_cdf_c(black_box(x)));
        });
    }
    group.finish();
}

fn bench_norm_batch(c: &mut Criterion) {
    // Serial loop over a representative moneyness grid — proxy for Phase 7
    // vector throughput before dedicated slice kernels exist.
    let xs: Vec<f64> = (-200..=200).map(|i| f64::from(i) * 0.05).collect();
    c.bench_function("norm_cdf_grid_401", |b| {
        b.iter(|| {
            let mut acc = 0.0;
            for &x in black_box(&xs) {
                acc += norm_cdf(x) + norm_cdf_c(x) + norm_pdf(x);
            }
            black_box(acc)
        });
    });
}

criterion_group!(
    benches,
    bench_norm_pdf,
    bench_norm_cdf,
    bench_norm_cdf_c,
    bench_norm_batch
);
criterion_main!(benches);
