use arcus::{
    config::Config,
    engine::Engine,
    error::{CancelError, ConfigError, OrderError},
    order::Side,
    price::PRICE_MULTIPLIER,
};

fn p(whole: u64) -> u64 {
    whole * PRICE_MULTIPLIER
}

// tick = 100 ticks (0.01), range 1.00 ..= 1000.00
fn engine() -> Engine {
    Engine::new(Config::new(100, p(1), p(1000)).unwrap())
}

#[test]
fn ids_are_sequential_and_rejects_do_not_consume_one() {
    let mut engine = engine();

    assert_eq!(engine.submit_limit_order(p(99), Side::Sell, 5, 0), Ok(1));
    assert_eq!(
        engine.submit_limit_order(p(99), Side::Sell, 0, 0),
        Err(OrderError::ZeroQuantity)
    );
    assert_eq!(engine.submit_limit_order(p(98), Side::Sell, 5, 0), Ok(2));
}

#[test]
fn trade_records_ids_and_caller_timestamp() {
    let mut engine = engine();

    let maker = engine.submit_limit_order(p(99), Side::Sell, 5, 10).unwrap();
    let taker = engine.submit_limit_order(p(100), Side::Buy, 5, 20).unwrap();

    let trade = engine.trades().iter().next().unwrap();
    assert_eq!(trade.id(), 1);
    assert_eq!(trade.maker_id(), maker);
    assert_eq!(trade.taker_id(), taker);
    assert_eq!(trade.price(), p(99));
    assert_eq!(trade.time(), 20);
    assert!(engine.book().validate().is_ok());
}

#[test]
fn invalid_prices_are_rejected_without_consuming_an_id() {
    let mut engine = engine();

    assert_eq!(engine.submit_limit_order(0, Side::Buy, 1, 0), Err(OrderError::ZeroPrice));
    assert_eq!(
        engine.submit_limit_order(p(1001), Side::Buy, 1, 0),
        Err(OrderError::PriceOutOfRange { price: p(1001), min: p(1), max: p(1000) })
    );
    assert_eq!(
        engine.submit_limit_order(p(5) + 1, Side::Buy, 1, 0),
        Err(OrderError::OffTick { price: p(5) + 1, tick_size: 100 })
    );
    assert_eq!(engine.submit_limit_order(p(5) + 100, Side::Buy, 1, 0), Ok(1));
}

#[test]
fn bad_config_is_rejected() {
    assert_eq!(Config::new(0, 1, 10), Err(ConfigError::ZeroTickSize));
    assert_eq!(Config::new(1, 0, 10), Err(ConfigError::InvalidRange { min: 0, max: 10 }));
    assert_eq!(Config::new(1, 11, 10), Err(ConfigError::InvalidRange { min: 11, max: 10 }));
}

#[test]
fn cancel_returns_open_quantity_and_removes_the_order() {
    let mut engine = engine();
    let id = engine.submit_limit_order(p(99), Side::Sell, 10, 0).unwrap();
    engine.submit_limit_order(p(99), Side::Buy, 4, 0).unwrap(); // fills 4 of 10

    assert_eq!(engine.cancel_order(id), Ok(6));
    assert_eq!(engine.book().best_ask(), None);
    assert!(engine.book().validate().is_ok());
}

#[test]
fn cancel_unknown_filled_or_repeated_is_an_error() {
    let mut engine = engine();
    let maker = engine.submit_limit_order(p(99), Side::Sell, 5, 0).unwrap();
    let resting = engine.submit_limit_order(p(98), Side::Buy, 5, 0).unwrap();
    engine.submit_limit_order(p(99), Side::Buy, 5, 0).unwrap(); // fills `maker`

    assert_eq!(engine.cancel_order(maker), Err(CancelError::UnknownOrder(maker)));
    assert_eq!(engine.cancel_order(777), Err(CancelError::UnknownOrder(777)));
    assert_eq!(engine.cancel_order(resting), Ok(5));
    assert_eq!(engine.cancel_order(resting), Err(CancelError::UnknownOrder(resting)));
}

#[test]
fn cancelled_order_no_longer_trades() {
    let mut engine = engine();
    let first = engine.submit_limit_order(p(99), Side::Sell, 5, 0).unwrap();
    let second = engine.submit_limit_order(p(99), Side::Sell, 5, 0).unwrap();
    engine.cancel_order(first).unwrap();
    engine.submit_limit_order(p(99), Side::Buy, 5, 0).unwrap();

    let trade = engine.trades().iter().next().unwrap();
    assert_eq!(trade.maker_id(), second);
    assert_eq!(engine.trades().len(), 1);
    assert!(engine.book().validate().is_ok());
}
