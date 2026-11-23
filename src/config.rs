use crate::{
    error::{ConfigError, OrderError},
    order_book::DEFAULT_CHUNK_BITS,
    price::Price,
};

/// Per-market price rules, fixed for the life of an `Engine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    tick_size: Price,
    min_price: Price,
    max_price: Price,
    chunk_bits: u32,
    max_chunks: Option<usize>,
    expected_orders: usize,
}

impl Config {
    pub fn new(tick_size: Price, min_price: Price, max_price: Price) -> Result<Self, ConfigError> {
        if tick_size == 0 {
            return Err(ConfigError::ZeroTickSize);
        }
        if min_price == 0 || min_price > max_price {
            return Err(ConfigError::InvalidRange { min: min_price, max: max_price });
        }
        Ok(Self {
            tick_size,
            min_price,
            max_price,
            chunk_bits: DEFAULT_CHUNK_BITS,
            max_chunks: None,
            expected_orders: 0,
        })
    }

    pub fn with_pool(mut self, chunk_bits: u32, max_chunks: Option<usize>) -> Result<Self, ConfigError> {
        if !(1..=24).contains(&chunk_bits) {
            return Err(ConfigError::InvalidChunkBits(chunk_bits));
        }
        self.chunk_bits = chunk_bits;
        self.max_chunks = max_chunks;
        Ok(self)
    }

    pub fn with_expected_orders(mut self, expected_orders: usize) -> Self {
        self.expected_orders = expected_orders;
        self
    }

    pub(crate) fn expected_orders(&self) -> usize {
        self.expected_orders
    }

    pub(crate) fn chunk_bits(&self) -> u32 {
        self.chunk_bits
    }

    pub(crate) fn max_chunks(&self) -> Option<usize> {
        self.max_chunks
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
