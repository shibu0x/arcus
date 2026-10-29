use std::collections::BTreeMap;

use crate::{
    order::{Order, Side},
    price::Price,
};

pub struct OrderBook {
    bids: BTreeMap<Price, Vec<Order>>,
    asks: BTreeMap<Price, Vec<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, order: Order) {
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

    pub fn remove(&mut self, price: Price, side: Side) {
        match side {
            Side::Buy => {
                self.bids.remove_entry(&price);
            }
            Side::Sell => {
                self.asks.remove_entry(&price);
            }
        }
    }

    pub fn get_mut(&mut self, price: Price, side: Side) -> Option<&mut Vec<Order>> {
        match side {
            Side::Buy => self.bids.get_mut(&price),
            Side::Sell => self.asks.get_mut(&price),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        // 1. Validate bid levels
        for (price, orders) in &self.bids {
            if orders.is_empty() {
                return Err(format!("Empty bid level at price {price}"));
            }

            for order in orders {
                if order.quantity() == 0 {
                    return Err(format!("Zero-quantity order in bids at price {price}"));
                }

                if order.side() != Side::Buy {
                    return Err(format!("Non-buy order found in bids at price {price}"));
                }

                if order.price() != *price {
                    return Err(format!("Order price does not match bid level {price}"));
                }
            }
        }

        // 2. Validate ask levels
        for (price, orders) in &self.asks {
            if orders.is_empty() {
                return Err(format!("Empty ask level at price {price}"));
            }

            for order in orders {
                if order.quantity() == 0 {
                    return Err(format!("Zero-quantity order in asks at price {price}"));
                }

                if order.side() != Side::Sell {
                    return Err(format!("Non-sell order found in asks at price {price}"));
                }

                if order.price() != *price {
                    return Err(format!("Order price does not match ask level {price}"));
                }
            }
        }

        // 3. The book must not be crossed after matching
        if let (Some(best_bid), Some(best_ask)) = (self.best_bid(), self.best_ask()) {
            if best_bid >= best_ask {
                return Err(format!(
                    "Crossed book: best bid {best_bid} >= best ask {best_ask}"
                ));
            }
        }

        Ok(())
    }
}
