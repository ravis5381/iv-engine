//! Criterion benchmarks for Black-76 / Black–Scholes pricing.
//!
//! ```bash
//! cargo bench -p iv-engine --bench black
//! ```

#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use iv_engine::{black_price, black_scholes_price};

fn bench_black_price(c: &mut Criterion) {
    let mut group = c.benchmark_group("black_price");
    let cases = [
        ("atm", 100.0, 100.0, 1.0, 0.2, true),
        ("otm_call", 100.0, 120.0, 1.0, 0.2, true),
        ("itm_call", 100.0, 80.0, 1.0, 0.2, true),
        ("deep_otm", 100.0, 200.0, 0.25, 0.15, true),
        ("high_vol", 100.0, 100.0, 1.0, 2.0, false),
    ];
    for &(label, f, k, t, v, call) in &cases {
        group.bench_with_input(BenchmarkId::new("case", label), &label, |b, _| {
            b.iter(|| black_price(black_box(f), black_box(k), black_box(t), black_box(v), call));
        });
    }
    group.finish();
}

fn bench_black_scholes(c: &mut Criterion) {
    c.bench_function("black_scholes_atm", |b| {
        b.iter(|| {
            black_scholes_price(
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

fn bench_black_grid(c: &mut Criterion) {
    let strikes: Vec<f64> = (50..=150).map(f64::from).collect();
    c.bench_function("black_price_strike_grid_101", |b| {
        b.iter(|| {
            let mut acc = 0.0;
            for &k in black_box(&strikes) {
                acc += black_price(100.0, k, 1.0, 0.2, true).unwrap_or(0.0);
            }
            black_box(acc)
        });
    });
}

criterion_group!(
    benches,
    bench_black_price,
    bench_black_scholes,
    bench_black_grid
);
criterion_main!(benches);
