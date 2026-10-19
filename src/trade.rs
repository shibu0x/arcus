use std::time::SystemTime;

use crate::{order::Side, price::Price};

pub struct Trade {
    price: Price,
    quantity: u32,
    side: Side,
    time: SystemTime,
}

impl Trade {
    pub fn new(price: Price, quantity: u32, side: Side) -> Self {
        Self {
            price,
            quantity,
            side,
            time: SystemTime::now(),
        }
    }
    
    pub fn price(&self) -> Price {
        self.price
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn time(&self) -> SystemTime {
        self.time
    }
}

pub struct TradeHistory {
    trades: Vec<Trade>,
}

impl TradeHistory {
    pub fn new() -> Self {
        Self { trades: Vec::new() }
    }

    pub fn add(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    pub fn len(&self) -> usize {
        self.trades.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trades.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Trade> {
        self.trades.iter()
    }
}
