use arcus::{config::Config, engine::Engine, order::Side, price::Price};
use std::{hint::black_box, time::Instant};

const N: usize = 5_000_000;
const MID: Price = 1_000_000;

// xorshift64: fixed seed so every run sees identical input
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

// Buys in [MID-20, MID+2], sells in [MID-2, MID+20]: the overlap makes ~10% cross.
// ponytail: no cancels yet (engine has none), add them to the mix with cancel support.
fn gen_orders(n: usize) -> Vec<(Price, Side, u32)> {
    let mut rng = Rng(0x9E3779B97F4A7C15);
    (0..n)
        .map(|_| {
            let qty = (rng.next() % 100 + 1) as u32;
            let r = rng.next() % 23;
            if rng.next() & 1 == 0 {
                (MID - 20 + r, Side::Buy, qty)
            } else {
                (MID - 2 + r, Side::Sell, qty)
            }
        })
        .collect()
}

fn run(orders: &[(Price, Side, u32)]) -> (f64, usize) {
    let mut engine = Engine::new(Config::new(1, 1, Price::MAX).unwrap());
    let start = Instant::now();
    for (i, &(price, side, qty)) in orders.iter().enumerate() {
        black_box(engine.submit_limit_order(price, side, qty, i as u64).unwrap());
    }
    let secs = start.elapsed().as_secs_f64();
    engine.book().validate().expect("book invariant broken");
    (secs, engine.trades().len())
}

fn main() {
    run(&gen_orders(N / 5)); // warm-up
    let orders = gen_orders(N); // generated outside the timer
    for i in 0..5 {
        let (secs, trades) = run(&orders);
        println!(
            "run {i}: {:.2}M orders/s  ({:.0} ns/order, {trades} trades)",
            N as f64 / secs / 1e6,
            secs * 1e9 / N as f64
        );
    }
}
