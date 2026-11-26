use crate::{
    matching::matching_engine,
    order::{Order, Side},
    order_book::OrderBook,
    price::{PRICE_MULTIPLIER, Price},
    trade::TradeHistory,
};

fn p(whole: u64) -> Price {
    whole * PRICE_MULTIPLIER
}

#[test]
fn buy_fully_fills_sell_order() {
    let mut book = OrderBook::new();
    let mut trade_history = TradeHistory::new();

    let existing_order = Order::new(1, p(99), Side::Sell, 5).unwrap();
    let incoming_order = Order::new(2, p(100), Side::Buy, 5).unwrap();

    book.add(existing_order).unwrap();
    matching_engine(incoming_order, &mut book, &mut trade_history, &mut 1, 0);
    let trade = trade_history.iter().next().unwrap();

    assert_eq!(book.best_ask(), None);
    assert_eq!(book.best_bid(), None);
    assert_eq!(trade_history.len(), 1);
    assert_eq!(trade.price(), p(99));
    assert_eq!(trade.quantity(), 5);
    assert_eq!(trade.side(), Side::Buy);
    assert!(
        book.validate().is_ok(),
        "Order book invariant violation: {:?}",
        book.validate()
    );
}

#[test]
fn buy_partially_fills_sell_order() {
    let mut book = OrderBook::new();
    let mut trade_history = TradeHistory::new();

    let existing_order = Order::new(3, p(99), Side::Sell, 10).unwrap();
    let incoming_order = Order::new(4, p(100), Side::Buy, 4).unwrap();

    book.add(existing_order).unwrap();
    matching_engine(incoming_order, &mut book, &mut trade_history, &mut 1, 0);
    let trade = trade_history.iter().next().unwrap();
    let asks = book.orders_at(p(99), Side::Sell);

    assert_eq!(asks[0].quantity(), 6);
    assert_eq!(book.best_ask(), Some(p(99)));
    assert_eq!(trade_history.len(), 1);
    assert_eq!(trade.price(), p(99));
    assert_eq!(trade.quantity(), 4);
    assert_eq!(trade.side(), Side::Buy);
    assert!(
        book.validate().is_ok(),
        "Order book invariant violation: {:?}",
        book.validate()
    );
}

#[test]
fn buy_does_not_cross_above_its_limit() {
    let mut book = OrderBook::new();
    let mut trade_history = TradeHistory::new();

    let existing_order = Order::new(5, p(110), Side::Sell, 5).unwrap();
    let incoming_order = Order::new(6, p(100), Side::Buy, 3).unwrap();

    book.add(existing_order).unwrap();

    matching_engine(incoming_order, &mut book, &mut trade_history, &mut 1, 0);

    assert_eq!(book.best_ask(), Some(p(110)));
    assert_eq!(
        book.orders_at(p(110), Side::Sell)[0].quantity(),
        5
    );
    assert_eq!(trade_history.len(), 0);
    assert_eq!(book.best_bid(), Some(p(100)));
    assert_eq!(
        book.orders_at(p(100), Side::Buy)[0].quantity(),
        3
    );
    assert!(
        book.validate().is_ok(),
        "Order book invariant violation: {:?}",
        book.validate()
    );
}

#[test]
fn sell_matches_best_bid() {
    let mut book = OrderBook::new();
    let mut trade_history = TradeHistory::new();

    let existing_order = Order::new(7, p(100), Side::Buy, 7).unwrap();
    let incoming_order = Order::new(8, p(99), Side::Sell, 3).unwrap();

    book.add(existing_order).unwrap();
    matching_engine(incoming_order, &mut book, &mut trade_history, &mut 1, 0);
    let trade = trade_history.iter().next().unwrap();

    assert_eq!(
        book.orders_at(p(100), Side::Buy)[0].quantity(),
        4
    );
    assert_eq!(trade_history.len(), 1);
    assert_eq!(book.best_bid(), Some(p(100)));
    assert_eq!(trade.price(), p(100));
    assert_eq!(trade.quantity(), 3);
    assert_eq!(trade.side(), Side::Sell);
    assert!(
        book.validate().is_ok(),
        "Order book invariant violation: {:?}",
        book.validate()
    );
}

#[test]
fn buy_matches_multiple_price_levels() {
    let mut book = OrderBook::new();
    let mut trade_history = TradeHistory::new();

    book.add(Order::new(9, p(99), Side::Sell, 3).unwrap()).unwrap();
    book.add(Order::new(10, p(100), Side::Sell, 4).unwrap()).unwrap();

    let incoming = Order::new(11, p(101), Side::Buy, 6).unwrap();
    matching_engine(incoming, &mut book, &mut trade_history, &mut 1, 0);

    let trades: Vec<_> = trade_history.iter().collect();

    assert_eq!(trades.len(), 2);

    assert_eq!(trades[0].price(), p(99));
    assert_eq!(trades[0].quantity(), 3);

    assert_eq!(trades[1].price(), p(100));
    assert_eq!(trades[1].quantity(), 3);

    assert_eq!(book.best_ask(), Some(p(100)));
    assert_eq!(
        book.orders_at(p(100), Side::Sell)[0].quantity(),
        1
    );
    assert_eq!(book.best_bid(), None);
    assert!(
        book.validate().is_ok(),
        "Order book invariant violation: {:?}",
        book.validate()
    );
}

#[test]
fn matches_orders_in_fifo_order_at_same_price() {
    let mut book = OrderBook::new();
    let mut trade_history = TradeHistory::new();

    book.add(Order::new(12, p(100), Side::Sell, 3).unwrap()).unwrap();

    book.add(Order::new(13, p(100), Side::Sell, 5).unwrap()).unwrap();

    let incoming = Order::new(14, p(100), Side::Buy, 3).unwrap();

    matching_engine(incoming, &mut book, &mut trade_history, &mut 1, 0);

    let trades: Vec<_> = trade_history.iter().collect();

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].price(), p(100));
    assert_eq!(trades[0].quantity(), 3);

    let remaining_orders = book.orders_at(p(100), Side::Sell);

    assert_eq!(remaining_orders.len(), 1);
    assert_eq!(remaining_orders[0].quantity(), 5);
    assert!(
        book.validate().is_ok(),
        "Order book invariant violation: {:?}",
        book.validate()
    );
}

#[test]
fn zero_quantity_order_is_rejected() {
    let result = Order::new(15, p(100), Side::Buy, 0);

    assert_eq!(result.unwrap_err(), crate::error::OrderError::ZeroQuantity);
}