use std::collections::BTreeMap;

use crate::{order::{Order, Side}, price::Price};

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
}