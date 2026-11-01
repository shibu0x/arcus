use crate::{
    error::OrderError,
    matching::matching_engine,
    order::{Order, OrderId, Side},
    order_book::OrderBook,
    price::Price,
    trade::{TradeHistory, TradeId},
};

pub struct Engine {
    book: OrderBook,
    trades: TradeHistory,
    next_order_id: OrderId,
    next_trade_id: TradeId,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            book: OrderBook::new(),
            trades: TradeHistory::new(),
            next_order_id: 1,
            next_trade_id: 1,
        }
    }

    pub fn book(&self) -> &OrderBook {
        &self.book
    }

    pub fn trades(&self) -> &TradeHistory {
        &self.trades
    }

    pub fn submit_limit_order(&mut self, price: Price, side: Side, qty: u32) -> Result<OrderId, OrderError> {
        let id = self.next_order_id;
        let order = Order::new(id, price, side, qty)?;
        self.next_order_id += 1;

        matching_engine(order, &mut self.book, &mut self.trades, &mut self.next_trade_id);
        Ok(id)
    }
}
