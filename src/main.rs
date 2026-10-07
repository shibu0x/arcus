use std::collections::BTreeMap;

type Price = u64;
const PRICE_MULTIPLIER: u64 = 10_000;

fn to_price(price: f64) -> Price {
    (price * PRICE_MULTIPLIER as f64).round() as u64
}

#[derive(Debug)]
enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
struct Order {
    price: Price,
    side: Side,
    quantity: u32,
}

struct OrderBook {
    bids: BTreeMap<Price, Vec<Order>>,
    asks: BTreeMap<Price, Vec<Order>>,
}

impl OrderBook {
    fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    fn add(&mut self, order: Order) {
        match order.side {
            Side::Buy => {
                self.bids.entry(order.price).or_default().push(order);
            }
            Side::Sell => {
                self.asks.entry(order.price).or_default().push(order);
            }
        }
    }
}

fn main() {
    let mut orderbook = OrderBook::new();

    orderbook.add(Order { price: to_price(100.0534), side: Side::Buy, quantity: 10 });
    orderbook.add(Order { price: to_price(1100 as f64), side: Side::Sell, quantity: 10 });

    println!("orderbook bids: {:?}", orderbook.bids);
    println!("orderbook asks: {:?}", orderbook.asks);
}
