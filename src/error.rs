use crate::{order::{OrderId, Side}, pool::Slot, price::Price};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OrderError {
    #[error("order quantity must be greater than zero")]
    ZeroQuantity,

    #[error("order price must be greater than zero")]
    ZeroPrice,

    #[error("price {price} is outside the allowed range {min}..={max}")]
    PriceOutOfRange { price: Price, min: Price, max: Price },

    #[error("price {price} is not a multiple of the tick size {tick_size}")]
    OffTick { price: Price, tick_size: Price },

    #[error("book is at capacity, no room for another resting order")]
    BookFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ConfigError {
    #[error("tick size must be greater than zero")]
    ZeroTickSize,

    #[error("price range {min}..={max} is invalid (min must be > 0 and <= max)")]
    InvalidRange { min: Price, max: Price },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BookError {
    #[error("empty {side:?} level at price {price}")]
    EmptyLevel { side: Side, price: Price },

    #[error("zero-quantity order {id} in {side:?} level at price {price}")]
    ZeroQuantity { side: Side, price: Price, id: OrderId },

    #[error("order {id} on wrong side in {side:?} level at price {price}")]
    WrongSide { side: Side, price: Price, id: OrderId },

    #[error("order {id} priced {order_price} sits in {side:?} level {level_price}")]
    PriceMismatch { side: Side, level_price: Price, order_price: Price, id: OrderId },

    #[error("crossed book: best bid {best_bid} >= best ask {best_ask}")]
    CrossedBook { best_bid: Price, best_ask: Price },

    #[error("{side:?} level at price {price} points at dead slot {slot}")]
    DeadSlot { side: Side, price: Price, slot: Slot },

    #[error("broken prev/next links at order {id} in {side:?} level at price {price}")]
    BrokenLink { side: Side, price: Price, id: OrderId },

    #[error("{side:?} level at price {price}: cached total {cached} != actual {actual}")]
    LevelTotalMismatch { side: Side, price: Price, cached: u64, actual: u64 },

    #[error("{side:?} level at price {price}: cached count {cached} != actual {actual}")]
    LevelCountMismatch { side: Side, price: Price, cached: u32, actual: u32 },

    #[error("order {id} is missing from the id index or maps to another slot")]
    IndexMismatch { id: OrderId },

    #[error("id index holds {indexed} entries but {resting} orders rest")]
    IndexSizeMismatch { indexed: usize, resting: usize },

    #[error("{resting} orders sit in levels but the pool holds {in_pool}")]
    PoolMismatch { resting: usize, in_pool: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PoolError {
    #[error("pool is full")]
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CancelError {
    #[error("order {0} is not resting (unknown, filled or already cancelled)")]
    UnknownOrder(OrderId),
}
