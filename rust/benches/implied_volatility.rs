//! Criterion benchmarks for Let's Be Rational implied volatility.
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iv_engine::{black_price, implied_volatility};

fn bench_implied_volatility(c: &mut Criterion) {
    c.bench_function("implied_volatility_atm", |b| {
        let price = black_price(100.0, 100.0, 1.0, 0.2, true).unwrap();
        b.iter(|| {
            implied_volatility(
                black_box(price),
                black_box(100.0),
                black_box(100.0),
                black_box(1.0),
                true,
            )
        });
    });
}

criterion_group!(benches, bench_implied_volatility);
criterion_main!(benches);
