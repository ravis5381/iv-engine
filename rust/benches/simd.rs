//! Scalar vs SIMD Normal PDF throughput.
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iv_engine::{norm_pdf, norm_pdf_slice};

fn bench_norm_pdf(c: &mut Criterion) {
    const N: usize = 4096;
    let xs: Vec<f64> = (0..N).map(|i| (i as f64) * 0.01 - 20.0).collect();
    let mut out = vec![0.0_f64; N];

    c.bench_function("norm_pdf_scalar_4096", |b| {
        b.iter(|| {
            for (i, &x) in black_box(&xs).iter().enumerate() {
                out[i] = norm_pdf(x);
            }
        });
    });

    c.bench_function("norm_pdf_slice_4096", |b| {
        b.iter(|| {
            norm_pdf_slice(black_box(&xs), black_box(&mut out)).unwrap();
        });
    });
}

criterion_group!(benches, bench_norm_pdf);
criterion_main!(benches);
