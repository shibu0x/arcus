use std::time::SystemTime;

use crate::{order::{OrderId, Side}, price::Price};

pub type TradeId = u64;

pub struct Trade {
    id: TradeId,
    price: Price,
    quantity: u32,
    side: Side,
    time: SystemTime,
    maker_id: OrderId,
    taker_id: OrderId,
}

impl Trade {
    pub fn new(
        id: TradeId,
        price: Price,
        quantity: u32,
        side: Side,
        maker_id: OrderId,
        taker_id: OrderId,
    ) -> Self {
        Self {
            id,
            price,
            quantity,
            side,
            time: SystemTime::now(),
            maker_id,
            taker_id
        }
    }

    pub fn id(&self) -> TradeId {
        self.id
    }

    pub fn maker_id(&self) -> OrderId {
        self.maker_id
    }

    pub fn taker_id(&self) -> OrderId {
        self.taker_id
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
