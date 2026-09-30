//! Print wall-clock throughput for scalar and batch APIs (no Criterion).
//!
//! ```bash
//! cargo run -p iv-engine --release --example bench_throughput
//! cargo run -p iv-engine --release --features rayon --example bench_throughput
//! ```

use std::time::Instant;

use iv_engine::{
    black_price, black_price_slice, delta_slice, implied_volatility, implied_volatility_slice,
    vega_slice, IVError,
};

#[cfg(feature = "rayon")]
use iv_engine::{black_price_slice_par, implied_volatility_slice_par};

const WARMUP: u32 = 3;
const SCALAR_ITERS: u32 = 500_000;
const BATCH_ITERS: u32 = 3;

type Book = (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>);

fn time_secs<F: FnMut()>(mut f: F, iters: u32) -> f64 {
    for _ in 0..WARMUP {
        f();
    }
    let t0 = Instant::now();
    for _ in 0..iters {
        f();
    }
    t0.elapsed().as_secs_f64() / f64::from(iters)
}

fn make_book(n: usize, seed: u64) -> Book {
    let mut rng = seed;
    let mut next = || {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng >> 11) as f64 / (1u64 << 53) as f64
    };

    let forwards = vec![100.0; n];
    let mut strikes = Vec::with_capacity(n);
    let mut mats = Vec::with_capacity(n);
    let mut vols = Vec::with_capacity(n);
    for _ in 0..n {
        let z = next() * 2.0 - 1.0;
        strikes.push(100.0 * (0.15 * z).exp());
        mats.push(0.05 + next() * 1.95);
        vols.push(0.10 + next() * 0.40);
    }
    let mut prices = vec![0.0; n];
    black_price_slice(&forwards, &strikes, &mats, &vols, true, &mut prices).unwrap();
    (forwards, strikes, mats, vols, prices)
}

fn print_scalar() -> Result<(), IVError> {
    let price = black_price(100.0, 100.0, 1.0, 0.2, true)?;

    let iv = time_secs(
        || {
            implied_volatility(price, 100.0, 100.0, 1.0, true).unwrap();
        },
        SCALAR_ITERS,
    );
    let bp = time_secs(
        || {
            black_price(100.0, 100.0, 1.0, 0.2, true).unwrap();
        },
        SCALAR_ITERS,
    );

    println!("Scalar (median-ish single call, {SCALAR_ITERS} repeats)");
    println!(
        "  black_price ATM          {:>8.1} ns/call  ({:>10.0} calls/s)",
        bp * 1e9,
        1.0 / bp
    );
    println!(
        "  implied_volatility ATM   {:>8.1} ns/call  ({:>10.0} calls/s)",
        iv * 1e9,
        1.0 / iv
    );
    Ok(())
}

fn bench_iv_batch(label: &str, n: usize, run: impl Fn(&[f64], &[f64], &[f64], &[f64], &mut [f64])) {
    let (f, k, t, _v, prices) = make_book(n, 42);
    let mut out = vec![0.0; n];
    let sec = time_secs(|| run(&prices, &f, &k, &t, &mut out), BATCH_ITERS);
    let rows_per_sec = n as f64 / sec;
    println!("  {label:<28} n={n:>7}  {sec:>8.3} s/run  {rows_per_sec:>12.0} rows/s");
}

fn print_batch() {
    println!("\nBatch implied_volatility_slice ({BATCH_ITERS} runs, new book each size)");

    for &n in &[10_000, 100_000, 1_000_000] {
        bench_iv_batch("serial", n, |prices, f, k, t, out| {
            implied_volatility_slice(prices, f, k, t, true, out).unwrap();
        });

        #[cfg(feature = "rayon")]
        bench_iv_batch("rayon parallel", n, |prices, f, k, t, out| {
            implied_volatility_slice_par(prices, f, k, t, true, out).unwrap();
        });
    }

    println!("\nBatch black_price_slice + delta_slice + vega_slice (n=1_000_000, serial)");
    let n = 1_000_000;
    let (f, k, t, v, _) = make_book(n, 7);
    let mut out = vec![0.0; n];

    let bp = time_secs(
        || black_price_slice(&f, &k, &t, &v, true, &mut out).unwrap(),
        1,
    );
    println!(
        "  black_price_slice            {:>8.3} s  {:>12.0} rows/s",
        bp,
        n as f64 / bp
    );

    let d = time_secs(|| delta_slice(&f, &k, &t, &v, true, &mut out).unwrap(), 1);
    println!(
        "  delta_slice                  {:>8.3} s  {:>12.0} rows/s",
        d,
        n as f64 / d
    );

    let vg = time_secs(|| vega_slice(&f, &k, &t, &v, &mut out).unwrap(), 1);
    println!(
        "  vega_slice                   {:>8.3} s  {:>12.0} rows/s",
        vg,
        n as f64 / vg
    );

    #[cfg(feature = "rayon")]
    {
        let bpp = time_secs(
            || black_price_slice_par(&f, &k, &t, &v, true, &mut out).unwrap(),
            1,
        );
        println!(
            "  black_price_slice (rayon)    {:>8.3} s  {:>12.0} rows/s",
            bpp,
            n as f64 / bpp
        );
    }
}

fn main() -> Result<(), IVError> {
    println!("iv-engine throughput (release build)\n");
    print_scalar()?;
    print_batch();
    Ok(())
}
