pub type Price = u64;
pub const PRICE_MULTIPLIER: u64 = 10_000;

pub fn to_price(price: f64) -> Price {
    (price * PRICE_MULTIPLIER as f64).round() as u64
}