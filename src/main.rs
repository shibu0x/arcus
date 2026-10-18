use std::collections::BTreeMap;

type Price = u64;
const PRICE_MULTIPLIER: u64 = 10_000;

fn to_price(price: f64) -> Price {
    (price * PRICE_MULTIPLIER as f64).round() as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn add(&mut self, order:Order) {
        match order.side {
            Side::Buy => {
                self.bids.entry(order.price).or_default().push(order);
            }
            Side::Sell => {
                self.asks.entry(order.price).or_default().push(order);
            }
        }
    }

    fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    fn remove(&mut self, price: Price, side: Side) {
        match side {
            Side::Buy => {
                self.bids.remove_entry(&price);
            }
            Side::Sell => {
                self.asks.remove_entry(&price);
            }
        }
    }

    fn get_mut(&mut self, price: Price, side: Side) -> Option<&mut Vec<Order>> {
        match side {
            Side::Buy => self.bids.get_mut(&price),
            Side::Sell => self.asks.get_mut(&price),
        }
    }
}

fn main() {
    let mut orderbook = OrderBook::new();

    orderbook.add(Order { price: to_price(100.0534), side: Side::Buy, quantity: 10 });
    orderbook.add(Order { price: to_price(1100 as f64), side: Side::Sell, quantity: 10 });

    println!("orderbook asks: {:?}", orderbook.best_ask());
    println!("orderbook bids: {:?}", orderbook.best_bid());
}

fn matching_engine(mut order: Order, orderbook: &mut OrderBook) -> Result<(), ()> {
    match order.side {
        Side::Buy => match_buy(&mut order, orderbook),
        Side::Sell => match_sell(&mut order, orderbook)
    }
    
    if order.quantity > 0{
        orderbook.add(order);
    }
    
    Ok(())
}

fn match_buy(order: &mut Order, orderbook: &mut OrderBook) {
    while order.quantity > 0 {
        let Some(best_ask) = orderbook.best_ask() else {
            break;
        };
        
        if order.price < best_ask {
            break;
        }

        let mut level_empty = false;

        if let Some(orders) = orderbook.get_mut(best_ask, Side::Sell) {
            if let Some(resting_order) = orders.first_mut() {
                let traded = order.quantity.min(resting_order.quantity);

                order.quantity -= traded;
                resting_order.quantity -= traded;
            }

            if orders.first().is_some_and(|o| o.quantity == 0) {
                orders.remove(0);
            }

            level_empty = orders.is_empty();
        }
        
        if level_empty {
            orderbook.remove(best_ask, Side::Sell);
        }
    }
}

fn match_sell(order: &mut Order, orderbook: &mut OrderBook) {
    while order.quantity > 0 {
        let Some(best_bid) = orderbook.best_bid() else {
            break;
        };

        if order.price > best_bid {
            break;
        }

        let mut level_empty = false;

        if let Some(orders) = orderbook.get_mut(best_bid, Side::Buy) {
            if let Some(resting_order) = orders.first_mut() {
                let traded = order.quantity.min(resting_order.quantity);

                order.quantity -= traded;
                resting_order.quantity -= traded;
            }

            if orders.first().is_some_and(|o| o.quantity == 0) {
                orders.remove(0);
            }

            level_empty = orders.is_empty();
        }
        
        if level_empty {
            orderbook.remove(best_bid, Side::Buy);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buy_fully_fills_sell_order() {
        let mut book = OrderBook::new();

        // Existing resting ask: sell 5 at $99.
        book.add(Order {
            price: to_price(99.0),
            side: Side::Sell,
            quantity: 5,
        });

        // Incoming buy: buy 5 at $100.
        matching_engine(
            Order {
                price: to_price(100.0),
                side: Side::Buy,
                quantity: 5,
            },
            &mut book,
        )
        .unwrap();

        // The ask should be completely removed.
        assert_eq!(book.best_ask(), None);
        assert_eq!(book.best_bid(), None);
    }

    #[test]
    fn buy_partially_fills_sell_order() {
        let mut book = OrderBook::new();

        // Resting ask: sell 10 at $99.
        book.add(Order {
            price: to_price(99.0),
            side: Side::Sell,
            quantity: 10,
        });

        // Incoming buy: buy 4 at $100.
        matching_engine(
            Order {
                price: to_price(100.0),
                side: Side::Buy,
                quantity: 4,
            },
            &mut book,
        )
        .unwrap();

        // Six units should remain for sale at $99.
        let asks = book.get_mut(to_price(99.0), Side::Sell).unwrap();
        assert_eq!(asks[0].quantity, 6);
        assert_eq!(book.best_ask(), Some(to_price(99.0)));
    }

    #[test]
    fn buy_does_not_cross_above_its_limit() {
        let mut book = OrderBook::new();

        // Resting ask: sell at $110.
        book.add(Order {
            price: to_price(110.0),
            side: Side::Sell,
            quantity: 5,
        });

        // Incoming buy: maximum price $100.
        matching_engine(
            Order {
                price: to_price(100.0),
                side: Side::Buy,
                quantity: 3,
            },
            &mut book,
        )
        .unwrap();

        // Ask remains untouched.
        assert_eq!(book.best_ask(), Some(to_price(110.0)));
        assert_eq!(
            book.get_mut(to_price(110.0), Side::Sell).unwrap()[0].quantity,
            5
        );

        // The unfilled buy rests in the bids book.
        assert_eq!(book.best_bid(), Some(to_price(100.0)));
        assert_eq!(
            book.get_mut(to_price(100.0), Side::Buy).unwrap()[0].quantity,
            3
        );
    }

    #[test]
    fn sell_matches_best_bid() {
        let mut book = OrderBook::new();

        // Resting bid: buy 7 at $100.
        book.add(Order {
            price: to_price(100.0),
            side: Side::Buy,
            quantity: 7,
        });

        // Incoming sell: sell 3 at $99.
        matching_engine(
            Order {
                price: to_price(99.0),
                side: Side::Sell,
                quantity: 3,
            },
            &mut book,
        )
        .unwrap();

        // Four units remain on the bid.
        assert_eq!(
            book.get_mut(to_price(100.0), Side::Buy).unwrap()[0].quantity,
            4
        );
        assert_eq!(book.best_bid(), Some(to_price(100.0)));
    }
}