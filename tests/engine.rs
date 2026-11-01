use arcus::{engine::Engine, error::OrderError, order::Side, price::to_price};

#[test]
fn ids_are_sequential_and_rejects_do_not_consume_one() {
    let mut engine = Engine::new();

    assert_eq!(engine.submit_limit_order(to_price(99.0), Side::Sell, 5), Ok(1));
    assert_eq!(
        engine.submit_limit_order(to_price(99.0), Side::Sell, 0),
        Err(OrderError::ZeroQuantity)
    );
    assert_eq!(engine.submit_limit_order(to_price(98.0), Side::Sell, 5), Ok(2));
}

#[test]
fn trade_records_maker_and_taker_ids() {
    let mut engine = Engine::new();

    let maker = engine.submit_limit_order(to_price(99.0), Side::Sell, 5).unwrap();
    let taker = engine.submit_limit_order(to_price(100.0), Side::Buy, 5).unwrap();

    let trade = engine.trades().iter().next().unwrap();
    assert_eq!(trade.id(), 1);
    assert_eq!(trade.maker_id(), maker);
    assert_eq!(trade.taker_id(), taker);
    assert_eq!(trade.price(), to_price(99.0));
    assert!(engine.book().validate().is_ok());
}
