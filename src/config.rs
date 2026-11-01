use crate::{
    error::{ConfigError, OrderError},
    price::Price,
};

/// Per-market price rules, fixed for the life of an `Engine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    tick_size: Price,
    min_price: Price,
    max_price: Price,
}

impl Config {
    pub fn new(tick_size: Price, min_price: Price, max_price: Price) -> Result<Self, ConfigError> {
        if tick_size == 0 {
            return Err(ConfigError::ZeroTickSize);
        }
        if min_price == 0 || min_price > max_price {
            return Err(ConfigError::InvalidRange { min: min_price, max: max_price });
        }
        Ok(Self { tick_size, min_price, max_price })
    }

    pub(crate) fn check_price(&self, price: Price) -> Result<(), OrderError> {
        if price == 0 {
            return Err(OrderError::ZeroPrice);
        }
        if price < self.min_price || price > self.max_price {
            return Err(OrderError::PriceOutOfRange {
                price,
                min: self.min_price,
                max: self.max_price,
            });
        }
        if price % self.tick_size != 0 {
            return Err(OrderError::OffTick { price, tick_size: self.tick_size });
        }
        Ok(())
    }
}
