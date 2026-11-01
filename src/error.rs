use crate::{order::{OrderId, Side}, price::Price};
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
}
