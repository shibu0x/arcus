use arcus::{engine::Engine, order::Side, price::to_price};

fn main() {
    let mut engine = Engine::new();

    engine.submit_limit_order(to_price(99.0), Side::Sell, 5).unwrap();
    engine.submit_limit_order(to_price(100.0), Side::Buy, 8).unwrap();

    for trade in engine.trades().iter() {
        println!(
            "Trade {}: maker={} taker={} price={} qty={} side={:?}",
            trade.id(),
            trade.maker_id(),
            trade.taker_id(),
            trade.price(),
            trade.quantity(),
            trade.side()
        );
    }

    println!(
        "best bid={:?} best ask={:?}",
        engine.book().best_bid(),
        engine.book().best_ask()
    );
}
