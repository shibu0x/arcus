use std::time::{SystemTime, UNIX_EPOCH};

use arcus::{config::Config, engine::Engine, order::Side, price::PRICE_MULTIPLIER};

fn now_ns() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
}

fn main() {
    let config = Config::new(100, PRICE_MULTIPLIER, 1000 * PRICE_MULTIPLIER).unwrap();
    let mut engine = Engine::new(config);

    engine.submit_limit_order(99 * PRICE_MULTIPLIER, Side::Sell, 5, now_ns()).unwrap();
    engine.submit_limit_order(100 * PRICE_MULTIPLIER, Side::Buy, 8, now_ns()).unwrap();

    for trade in engine.trades().iter() {
        println!(
            "Trade {}: maker={} taker={} price={} qty={} side={:?} t={}",
            trade.id(),
            trade.maker_id(),
            trade.taker_id(),
            trade.price(),
            trade.quantity(),
            trade.side(),
            trade.time()
        );
    }

    println!(
        "best bid={:?} best ask={:?}",
        engine.book().best_bid(),
        engine.book().best_ask()
    );
}
