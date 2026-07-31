//! Criterion benchmarks for analytical Greeks.
//!
//! ```bash
//! cargo bench -p iv-engine --bench greeks
//! ```

#![allow(missing_docs, clippy::semicolon_if_nothing_returned)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iv_engine::{black_scholes_delta, delta, gamma, theta, vanna, vega, vomma};

fn bench_black_greeks(c: &mut Criterion) {
    let mut group = c.benchmark_group("black_greeks");
    group.bench_function("delta", |b| {
        b.iter(|| {
            delta(
                black_box(100.0),
                black_box(100.0),
                black_box(1.0),
                black_box(0.2),
                true,
            )
        });
    });
    group.bench_function("gamma", |b| {
        b.iter(|| {
            gamma(
                black_box(100.0),
                black_box(100.0),
                black_box(1.0),
                black_box(0.2),
            )
        });
    });
    group.bench_function("vega", |b| {
        b.iter(|| {
            vega(
                black_box(100.0),
                black_box(100.0),
                black_box(1.0),
                black_box(0.2),
            )
        });
    });
    group.bench_function("theta", |b| {
        b.iter(|| {
            theta(
                black_box(100.0),
                black_box(100.0),
                black_box(1.0),
                black_box(0.2),
            )
        });
    });
    group.bench_function("vomma", |b| {
        b.iter(|| {
            vomma(
                black_box(100.0),
                black_box(100.0),
                black_box(1.0),
                black_box(0.2),
            )
        });
    });
    group.bench_function("vanna", |b| {
        b.iter(|| {
            vanna(
                black_box(100.0),
                black_box(100.0),
                black_box(1.0),
                black_box(0.2),
            )
        });
    });
    group.finish();
}

fn bench_bs_delta(c: &mut Criterion) {
    c.bench_function("black_scholes_delta", |b| {
        b.iter(|| {
            black_scholes_delta(
                black_box(100.0),
                black_box(100.0),
                black_box(1.0),
                black_box(0.05),
                black_box(0.02),
                black_box(0.2),
                true,
            )
        });
    });
}

criterion_group!(benches, bench_black_greeks, bench_bs_delta);
criterion_main!(benches);
