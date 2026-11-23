use arcus::{config::Config, engine::Engine, order::Side, price::Price};
use std::{hint::black_box, time::Instant};

const N: usize = 5_000_000;
const CANCEL_WINDOW: u64 = 50_000;
const NARROW_MID: Price = 1_000_000;
const DEEP_MID: Price = 2_000_000;
const DEEP_TICK: Price = 10;
const DEEP_SPREAD_TICKS: u64 = 20_000;

#[derive(Clone, Copy)]
enum Cmd {
    Limit(Price, Side, u32),
    Cancel(u64),
}

#[derive(Clone, Copy)]
enum Shape {
    Narrow,
    Deep,
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn limit(rng: &mut Rng, shape: Shape) -> Cmd {
    let qty = (rng.next() % 100 + 1) as u32;
    match shape {
        Shape::Narrow => {
            let r = rng.next() % 23;
            if rng.next() & 1 == 0 {
                Cmd::Limit(NARROW_MID - 20 + r, Side::Buy, qty)
            } else {
                Cmd::Limit(NARROW_MID - 2 + r, Side::Sell, qty)
            }
        }
        Shape::Deep => {
            let distance = (rng.next() % DEEP_SPREAD_TICKS).min(rng.next() % DEEP_SPREAD_TICKS);
            if rng.next() & 1 == 0 {
                Cmd::Limit(DEEP_MID + 2 * DEEP_TICK - distance * DEEP_TICK, Side::Buy, qty)
            } else {
                Cmd::Limit(DEEP_MID - 2 * DEEP_TICK + distance * DEEP_TICK, Side::Sell, qty)
            }
        }
    }
}

fn gen_commands(n: usize, cancel_pct: u64, shape: Shape) -> Vec<Cmd> {
    let mut rng = Rng(0x9E3779B97F4A7C15);
    let mut limits = 0u64;
    (0..n)
        .map(|_| {
            if limits > 0 && rng.next() % 100 < cancel_pct {
                return Cmd::Cancel(limits - rng.next() % limits.min(CANCEL_WINDOW));
            }
            limits += 1;
            limit(&mut rng, shape)
        })
        .collect()
}

fn config(shape: Shape) -> Config {
    let base = match shape {
        Shape::Narrow => Config::new(1, 1, Price::MAX),
        Shape::Deep => Config::new(DEEP_TICK, DEEP_TICK, 10_000_000),
    };
    base.unwrap().with_expected_orders(N)
}

fn apply(engine: &mut Engine, step: usize, cmd: Cmd) -> bool {
    match cmd {
        Cmd::Limit(price, side, qty) => {
            black_box(engine.submit_limit_order(price, side, qty, step as u64).unwrap());
            true
        }
        Cmd::Cancel(id) => engine.cancel_order(black_box(id)).is_ok(),
    }
}

fn throughput(label: &str, cmds: &[Cmd], config: Config) {
    println!("{label}");
    for i in 0..5 {
        let mut engine = Engine::new(config);
        let mut cancels = 0;
        let mut hits = 0;
        let start = Instant::now();
        for (step, &cmd) in cmds.iter().enumerate() {
            let hit = apply(&mut engine, step, cmd);
            if let Cmd::Cancel(_) = cmd {
                cancels += 1;
                hits += hit as usize;
            }
        }
        let secs = start.elapsed().as_secs_f64();
        engine.book().validate().expect("book invariant broken");
        println!(
            "  run {i}: {:.2}M cmds/s  ({:.0} ns/cmd, {} trades, {hits}/{cancels} cancels hit, levels bid/ask {}/{})",
            cmds.len() as f64 / secs / 1e6,
            secs * 1e9 / cmds.len() as f64,
            engine.trades().len(),
            engine.book().depth(Side::Buy).len(),
            engine.book().depth(Side::Sell).len()
        );
    }
}

fn percentile(sorted: &[u32], p: f64) -> u32 {
    sorted[((sorted.len() - 1) as f64 * p) as usize]
}

fn timer_floor() -> u32 {
    let mut samples: Vec<u32> = (0..1_000_000)
        .map(|_| {
            let start = Instant::now();
            black_box(());
            start.elapsed().as_nanos() as u32
        })
        .collect();
    samples.sort_unstable();
    percentile(&samples, 0.5)
}

fn latency(label: &str, cmds: &[Cmd], config: Config) {
    let mut engine = Engine::new(config);
    let mut samples = Vec::with_capacity(cmds.len());
    let mut spikes = Vec::new();
    for (step, &cmd) in cmds.iter().enumerate() {
        let start = Instant::now();
        apply(&mut engine, step, cmd);
        let ns = start.elapsed().as_nanos() as u32;
        if ns > 500_000 {
            spikes.push((step, ns));
        }
        samples.push(ns);
    }
    samples.sort_unstable();

    println!("{label} latency per command (ns)");
    println!(
        "  p50 {}  p90 {}  p99 {}  p99.9 {}  p99.99 {}  max {}",
        percentile(&samples, 0.5),
        percentile(&samples, 0.9),
        percentile(&samples, 0.99),
        percentile(&samples, 0.999),
        percentile(&samples, 0.9999),
        samples[samples.len() - 1]
    );
    let over = |ns: u32| samples.len() - samples.partition_point(|&s| s <= ns);
    println!(
        "  over 1us: {}  over 10us: {}  over 100us: {}  over 1ms: {}",
        over(1_000),
        over(10_000),
        over(100_000),
        over(1_000_000)
    );
    spikes.sort_by_key(|&(_, ns)| std::cmp::Reverse(ns));
    let shown: Vec<String> = spikes
        .iter()
        .take(8)
        .map(|&(step, ns)| format!("step {step} ({:.1} ms)", ns as f64 / 1e6))
        .collect();
    println!("  worst: {}", shown.join(", "));
}

fn main() {
    println!("timer floor (median empty Instant::now/elapsed): {} ns", timer_floor());
    let scenarios = [
        ("narrow, limits only", 0, Shape::Narrow),
        ("narrow, mixed 45% cancels", 45, Shape::Narrow),
        ("deep (tick 0.001, ~20k levels), mixed 45% cancels", 45, Shape::Deep),
    ];
    let filter = std::env::args().nth(1).unwrap_or_default();
    for (label, cancel_pct, shape) in scenarios {
        if !label.contains(&filter) {
            continue;
        }
        let cmds = gen_commands(N, cancel_pct, shape);
        let config = config(shape);
        throughput(label, &cmds, config);
        latency(label, &cmds, config);
    }
}
