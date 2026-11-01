use arcus::{config::Config, engine::Engine, order::Side, price::PRICE_MULTIPLIER};

fn main() {
    // tick 0.01, range 1.00 ..= 1000.00
    let config = Config::new(100, PRICE_MULTIPLIER, 1000 * PRICE_MULTIPLIER).unwrap();
    let mut engine = Engine::new(config);

    engine.submit_limit_order(99 * PRICE_MULTIPLIER, Side::Sell, 5, 1).unwrap();
    engine.submit_limit_order(100 * PRICE_MULTIPLIER, Side::Buy, 8, 2).unwrap();

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
