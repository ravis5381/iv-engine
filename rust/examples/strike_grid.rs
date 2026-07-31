//! Batch Black prices and IV recovery on a strike grid.
//!
//! ```bash
//! cargo run -p iv-engine --example strike_grid --features rayon
//! ```

use iv_engine::{black_price_slice, black_price_slice_par, implied_volatility_slice};

fn main() {
    let n = 21usize;
    let forwards = vec![100.0_f64; n];
    let strikes: Vec<f64> = (0..n).map(|i| 80.0 + i as f64 * 2.0).collect();
    let mats = vec![1.0_f64; n];
    let vols = vec![0.25_f64; n];
    let mut prices = vec![0.0_f64; n];
    let mut ivs = vec![0.0_f64; n];

    if iv_engine::rayon_enabled() {
        black_price_slice_par(&forwards, &strikes, &mats, &vols, true, &mut prices)
            .expect("parallel price");
    } else {
        black_price_slice(&forwards, &strikes, &mats, &vols, true, &mut prices).expect("price");
    }
    implied_volatility_slice(&prices, &forwards, &strikes, &mats, true, &mut ivs).expect("iv");

    println!("{:>8} {:>14} {:>14}", "K", "price", "iv");
    for i in 0..n {
        println!("{:>8.1} {:>14.8} {:>14.10}", strikes[i], prices[i], ivs[i]);
    }
}
