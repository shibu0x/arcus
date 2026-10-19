use arcus::{
    matching::matching_engine,
    order::{Order, Side},
    order_book::OrderBook,
    trade::TradeHistory,
    price::to_price,
};

fn main() {
    let mut book = OrderBook::new();
    let mut history = TradeHistory::new();

    book.add(Order::new(to_price(99.0), Side::Sell, 5).unwrap());

    matching_engine(
        Order::new(to_price(100.0), Side::Buy, 5).unwrap(),
        &mut book,
        &mut history,
    );

    for trade in history.iter() {
        println!(
            "Trade: price={}, quantity={}, side={:?}",
            trade.price(),
            trade.quantity(),
            trade.side()
        );
    }
}