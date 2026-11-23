use std::collections::{BTreeMap, VecDeque};

use arcus::{config::Config, engine::Engine, order::Side};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fill {
    id: u64,
    maker: u64,
    taker: u64,
    price: u64,
    qty: u32,
    side: Side,
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

struct Reference {
    bids: BTreeMap<u64, VecDeque<(u64, u32)>>,
    asks: BTreeMap<u64, VecDeque<(u64, u32)>>,
    next_order: u64,
    next_trade: u64,
}

impl Reference {
    fn new() -> Self {
        Self { bids: BTreeMap::new(), asks: BTreeMap::new(), next_order: 1, next_trade: 1 }
    }

    fn submit(&mut self, price: u64, side: Side, qty: u32) -> (u64, Vec<Fill>) {
        let id = self.next_order;
        self.next_order += 1;
        let mut left = qty;
        let mut fills = Vec::new();

        while left > 0 {
            let best = match side {
                Side::Buy => self.asks.keys().next().copied(),
                Side::Sell => self.bids.keys().next_back().copied(),
            };
            let Some(best) = best else { break };
            let crosses = match side {
                Side::Buy => price >= best,
                Side::Sell => price <= best,
            };
            if !crosses {
                break;
            }

            let book = match side {
                Side::Buy => &mut self.asks,
                Side::Sell => &mut self.bids,
            };
            let queue = book.get_mut(&best).unwrap();
            let front = queue.front_mut().unwrap();
            let maker = front.0;
            let filled = left.min(front.1);
            front.1 -= filled;
            left -= filled;
            if front.1 == 0 {
                queue.pop_front();
            }
            if queue.is_empty() {
                book.remove(&best);
            }

            fills.push(Fill { id: self.next_trade, maker, taker: id, price: best, qty: filled, side });
            self.next_trade += 1;
        }

        if left > 0 {
            let book = match side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };
            book.entry(price).or_default().push_back((id, left));
        }
        (id, fills)
    }

    fn cancel(&mut self, id: u64) -> Option<u32> {
        for book in [&mut self.bids, &mut self.asks] {
            let found = book.iter().find_map(|(&price, queue)| {
                queue.iter().position(|&(order, _)| order == id).map(|pos| (price, pos))
            });
            if let Some((price, pos)) = found {
                let queue = book.get_mut(&price).unwrap();
                let (_, qty) = queue.remove(pos).unwrap();
                if queue.is_empty() {
                    book.remove(&price);
                }
                return Some(qty);
            }
        }
        None
    }

    fn depth(&self, side: Side) -> Vec<(u64, u64)> {
        let sum = |queue: &VecDeque<(u64, u32)>| queue.iter().map(|&(_, q)| u64::from(q)).sum();
        match side {
            Side::Buy => self.bids.iter().rev().map(|(&p, q)| (p, sum(q))).collect(),
            Side::Sell => self.asks.iter().map(|(&p, q)| (p, sum(q))).collect(),
        }
    }

    fn best_bid(&self) -> Option<u64> {
        self.bids.keys().next_back().copied()
    }

    fn best_ask(&self) -> Option<u64> {
        self.asks.keys().next().copied()
    }
}

fn check_book(engine: &Engine, reference: &Reference, context: &str) {
    assert_eq!(engine.book().validate(), Ok(()), "{context}");
    assert_eq!(engine.book().depth(Side::Buy), reference.depth(Side::Buy), "bids: {context}");
    assert_eq!(engine.book().depth(Side::Sell), reference.depth(Side::Sell), "asks: {context}");
}

fn run(seed: u64, ops: usize, chunk_bits: u32) {
    let config = Config::new(1, 1, u64::MAX).unwrap().with_pool(chunk_bits, None).unwrap().with_expected_orders(1_000);
    let mut engine = Engine::new(config);
    let mut reference = Reference::new();
    let mut rng = Rng(seed);
    let mut seen = 0;

    for step in 0..ops {
        let context = format!("seed {seed}, chunk_bits {chunk_bits}, step {step}");

        if rng.next() % 100 < 45 {
            let issued = reference.next_order - 1;
            let id = if issued > 0 && rng.next() % 5 != 0 {
                issued - rng.next() % issued.min(100)
            } else {
                rng.next() % (issued + 4)
            };
            let expected = reference.cancel(id);
            let actual = engine.cancel_order(id).ok();
            assert_eq!(actual, expected, "cancel {id}: {context}");
        } else {
            let side = if rng.next() & 1 == 0 { Side::Buy } else { Side::Sell };
            let price = 90 + rng.next() % 21;
            let qty = 1 + (rng.next() % 20) as u32;

            let (expected_id, expected) = reference.submit(price, side, qty);
            let id = engine.submit_limit_order(price, side, qty, step as u64).unwrap();
            assert_eq!(id, expected_id, "{context}");

            let actual: Vec<Fill> = engine
                .trades()
                .iter()
                .skip(seen)
                .map(|t| Fill {
                    id: t.id(),
                    maker: t.maker_id(),
                    taker: t.taker_id(),
                    price: t.price(),
                    qty: t.quantity(),
                    side: t.side(),
                })
                .collect();
            seen += actual.len();
            assert_eq!(actual, expected, "trades: {context}");
        }

        assert_eq!(engine.book().best_bid(), reference.best_bid(), "{context}");
        assert_eq!(engine.book().best_ask(), reference.best_ask(), "{context}");
        if step % 16 == 0 {
            check_book(&engine, &reference, &context);
        }
    }

    check_book(&engine, &reference, &format!("seed {seed}, chunk_bits {chunk_bits}, final"));
    assert_eq!(engine.trades().len(), seen);
}

#[test]
fn matches_reference_model_with_tiny_pool_chunks() {
    for seed in 1..=20 {
        run(seed, 20_000, 2);
    }
}

#[test]
fn matches_reference_model_with_default_pool_chunks() {
    for seed in 21..=40 {
        run(seed, 20_000, 16);
    }
}
