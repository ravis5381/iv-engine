//! ATM Black price → implied volatility round-trip.
//!
//! ```bash
//! cargo run -p iv-engine --example atm_roundtrip
//! ```

use iv_engine::{black_price, implied_volatility};

fn main() {
    let forward = 100.0;
    let strike = 100.0;
    let maturity = 1.0;
    let sigma = 0.20;
    let is_call = true;

    let price = black_price(forward, strike, maturity, sigma, is_call).expect("price");
    let recovered =
        implied_volatility(price, forward, strike, maturity, is_call).expect("implied vol");

    println!("Black ATM call price = {price:.12}");
    println!("Recovered σ          = {recovered:.12}");
    println!("|σ − 0.2|            = {:.3e}", (recovered - sigma).abs());
}
