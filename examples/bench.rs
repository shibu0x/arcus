use arcus::{config::Config, engine::Engine, order::Side, price::Price};
use std::{hint::black_box, time::Instant};

const N: usize = 5_000_000;
const MID: Price = 1_000_000;
const CANCEL_WINDOW: u64 = 50_000;

#[derive(Clone, Copy)]
enum Cmd {
    Limit(Price, Side, u32),
    Cancel(u64),
}

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

fn gen_commands(n: usize, cancel_pct: u64) -> Vec<Cmd> {
    let mut rng = Rng(0x9E3779B97F4A7C15);
    let mut limits = 0u64;
    (0..n)
        .map(|_| {
            if limits > 0 && rng.next() % 100 < cancel_pct {
                return Cmd::Cancel(limits - rng.next() % limits.min(CANCEL_WINDOW));
            }
            limits += 1;
            let qty = (rng.next() % 100 + 1) as u32;
            let r = rng.next() % 23;
            if rng.next() & 1 == 0 {
                Cmd::Limit(MID - 20 + r, Side::Buy, qty)
            } else {
                Cmd::Limit(MID - 2 + r, Side::Sell, qty)
            }
        })
        .collect()
}

struct Stats {
    secs: f64,
    trades: usize,
    cancel_hits: usize,
    cancels: usize,
}

fn run(cmds: &[Cmd]) -> Stats {
    let mut engine = Engine::new(Config::new(1, 1, Price::MAX).unwrap());
    let (mut cancels, mut cancel_hits) = (0, 0);
    let start = Instant::now();
    for (i, &cmd) in cmds.iter().enumerate() {
        match cmd {
            Cmd::Limit(price, side, qty) => {
                black_box(engine.submit_limit_order(price, side, qty, i as u64).unwrap());
            }
            Cmd::Cancel(id) => {
                cancels += 1;
                cancel_hits += engine.cancel_order(black_box(id)).is_ok() as usize;
            }
        }
    }
    let secs = start.elapsed().as_secs_f64();
    engine.book().validate().expect("book invariant broken");
    Stats { secs, trades: engine.trades().len(), cancel_hits, cancels }
}

fn report(label: &str, cancel_pct: u64) {
    let cmds = gen_commands(N, cancel_pct);
    println!("{label}");
    for i in 0..5 {
        let s = run(&cmds);
        println!(
            "  run {i}: {:.2}M cmds/s  ({:.0} ns/cmd, {} trades, {}/{} cancels hit)",
            N as f64 / s.secs / 1e6,
            s.secs * 1e9 / N as f64,
            s.trades,
            s.cancel_hits,
            s.cancels
        );
    }
}

fn main() {
    run(&gen_commands(N / 5, 45));
    report("limits only", 0);
    report("mixed: 45% cancels", 45);
}
