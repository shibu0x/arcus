use std::collections::BTreeMap;

use crate::{
    error::BookError,
    order::{Order, Side},
    price::Price,
};

pub struct OrderBook {
    bids: BTreeMap<Price, Vec<Order>>,
    asks: BTreeMap<Price, Vec<Order>>,
}

impl OrderBook {
    pub(crate) fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub(crate) fn add(&mut self, order: Order) {
        match order.side() {
            Side::Buy => {
                self.bids.entry(order.price()).or_default().push(order);
            }
            Side::Sell => {
                self.asks.entry(order.price()).or_default().push(order);
            }
        }
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    pub(crate) fn remove(&mut self, price: Price, side: Side) {
        match side {
            Side::Buy => {
                self.bids.remove_entry(&price);
            }
            Side::Sell => {
                self.asks.remove_entry(&price);
            }
        }
    }

    pub(crate) fn get_mut(&mut self, price: Price, side: Side) -> Option<&mut Vec<Order>> {
        match side {
            Side::Buy => self.bids.get_mut(&price),
            Side::Sell => self.asks.get_mut(&price),
        }
    }

    pub fn validate(&self) -> Result<(), BookError> {
        Self::validate_side(&self.bids, Side::Buy)?;
        Self::validate_side(&self.asks, Side::Sell)?;

        if let (Some(best_bid), Some(best_ask)) = (self.best_bid(), self.best_ask()) {
            if best_bid >= best_ask {
                return Err(BookError::CrossedBook { best_bid, best_ask });
            }
        }

        Ok(())
    }

    fn validate_side(levels: &BTreeMap<Price, Vec<Order>>, side: Side) -> Result<(), BookError> {
        for (&price, orders) in levels {
            if orders.is_empty() {
                return Err(BookError::EmptyLevel { side, price });
            }

            for order in orders {
                let id = order.id();
                if order.quantity() == 0 {
                    return Err(BookError::ZeroQuantity { side, price, id });
                }
                if order.side() != side {
                    return Err(BookError::WrongSide { side, price, id });
                }
                if order.price() != price {
                    return Err(BookError::PriceMismatch {
                        side,
                        level_price: price,
                        order_price: order.price(),
                        id,
                    });
                }
            }
        }
        Ok(())
    }
}
