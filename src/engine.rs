use crate::{
    config::Config,
    error::{CancelError, OrderError},
    matching::matching_engine,
    order::{Order, OrderId, Side},
    order_book::OrderBook,
    price::Price,
    trade::{Timestamp, TradeHistory, TradeId},
};

pub struct Engine {
    config: Config,
    book: OrderBook,
    trades: TradeHistory,
    next_order_id: OrderId,
    next_trade_id: TradeId,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        Self {
            config,
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

    pub fn submit_limit_order(
        &mut self,
        price: Price,
        side: Side,
        qty: u32,
        timestamp: Timestamp,
    ) -> Result<OrderId, OrderError> {
        self.config.check_price(price)?;

        let id = self.next_order_id;
        let order = Order::new(id, price, side, qty)?;
        if !self.book.has_room() {
            return Err(OrderError::BookFull);
        }
        self.next_order_id += 1;

        matching_engine(order, &mut self.book, &mut self.trades, &mut self.next_trade_id, timestamp);
        Ok(id)
    }

    /// Cancels a resting order and returns the quantity that was still open.
    pub fn cancel_order(&mut self, id: OrderId) -> Result<u32, CancelError> {
        self.book
            .cancel(id)
            .map(|order| order.quantity())
            .ok_or(CancelError::UnknownOrder(id))
    }
}
