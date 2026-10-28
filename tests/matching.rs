use arcus::{
    matching::matching_engine,
    order::{Order, Side},
    order_book::OrderBook,
    price::to_price,
    trade::TradeHistory,
};

#[test]
fn buy_fully_fills_sell_order() {
    let mut book = OrderBook::new();
    let mut trade_history = TradeHistory::new();

    let existing_order = Order::new(to_price(99.0), Side::Sell, 5).unwrap();
    let incoming_order = Order::new(to_price(100.0), Side::Buy, 5).unwrap();

    book.add(existing_order);
    matching_engine(incoming_order, &mut book, &mut trade_history);
    let trade = trade_history.iter().next().unwrap();

    assert_eq!(book.best_ask(), None);
    assert_eq!(book.best_bid(), None);
    assert_eq!(trade_history.len(), 1);
    assert_eq!(trade.price(), to_price(99.0));
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

    let existing_order = Order::new(to_price(99.0), Side::Sell, 10).unwrap();
    let incoming_order = Order::new(to_price(100.0), Side::Buy, 4).unwrap();

    book.add(existing_order);
    matching_engine(incoming_order, &mut book, &mut trade_history);
    let trade = trade_history.iter().next().unwrap();
    let asks = book.get_mut(to_price(99.0), Side::Sell).unwrap();

    assert_eq!(asks[0].quantity(), 6);
    assert_eq!(book.best_ask(), Some(to_price(99.0)));
    assert_eq!(trade_history.len(), 1);
    assert_eq!(trade.price(), to_price(99.0));
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

    let existing_order = Order::new(to_price(110.0), Side::Sell, 5).unwrap();
    let incoming_order = Order::new(to_price(100.0), Side::Buy, 3).unwrap();

    book.add(existing_order);

    matching_engine(incoming_order, &mut book, &mut trade_history);

    assert_eq!(book.best_ask(), Some(to_price(110.0)));
    assert_eq!(
        book.get_mut(to_price(110.0), Side::Sell).unwrap()[0].quantity(),
        5
    );
    assert_eq!(trade_history.len(), 0);
    assert_eq!(book.best_bid(), Some(to_price(100.0)));
    assert_eq!(
        book.get_mut(to_price(100.0), Side::Buy).unwrap()[0].quantity(),
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

    let existing_order = Order::new(to_price(100.0), Side::Buy, 7).unwrap();
    let incoming_order = Order::new(to_price(99.0), Side::Sell, 3).unwrap();

    book.add(existing_order);
    matching_engine(incoming_order, &mut book, &mut trade_history);
    let trade = trade_history.iter().next().unwrap();

    assert_eq!(
        book.get_mut(to_price(100.0), Side::Buy).unwrap()[0].quantity(),
        4
    );
    assert_eq!(trade_history.len(), 1);
    assert_eq!(book.best_bid(), Some(to_price(100.0)));
    assert_eq!(trade.price(), to_price(100.0));
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

    book.add(Order::new(to_price(99.0), Side::Sell, 3).unwrap());
    book.add(Order::new(to_price(100.0), Side::Sell, 4).unwrap());

    let incoming = Order::new(to_price(101.0), Side::Buy, 6).unwrap();
    matching_engine(incoming, &mut book, &mut trade_history);

    let trades: Vec<_> = trade_history.iter().collect();

    assert_eq!(trades.len(), 2);

    assert_eq!(trades[0].price(), to_price(99.0));
    assert_eq!(trades[0].quantity(), 3);

    assert_eq!(trades[1].price(), to_price(100.0));
    assert_eq!(trades[1].quantity(), 3);

    assert_eq!(book.best_ask(), Some(to_price(100.0)));
    assert_eq!(
        book.get_mut(to_price(100.0), Side::Sell).unwrap()[0].quantity(),
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

    book.add(Order::new(to_price(100.0), Side::Sell, 3).unwrap());

    book.add(Order::new(to_price(100.0), Side::Sell, 5).unwrap());

    let incoming = Order::new(to_price(100.0), Side::Buy, 3).unwrap();

    matching_engine(incoming, &mut book, &mut trade_history);

    let trades: Vec<_> = trade_history.iter().collect();

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].price(), to_price(100.0));
    assert_eq!(trades[0].quantity(), 3);

    let remaining_orders = book.get_mut(to_price(100.0), Side::Sell).unwrap();

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
    let result = Order::new(to_price(100.0), Side::Buy, 0);

    assert!(result.is_err());
}