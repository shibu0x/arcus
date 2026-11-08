use std::{
    collections::{BTreeMap, HashMap},
    hash::{BuildHasherDefault, Hasher},
};

use crate::{
    error::{BookError, PoolError},
    level::Level,
    order::{Order, OrderId, Side},
    pool::{NIL, Pool, Slot},
    price::Price,
};

/// Order ids are assigned by the engine, never chosen by clients, so SipHash's
/// flood resistance buys nothing; one multiply spreads sequential ids well.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("order ids hash through write_u64");
    }

    fn write_u64(&mut self, n: u64) {
        self.0 = n.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

type IdMap = HashMap<OrderId, Slot, BuildHasherDefault<IdHasher>>;

const DEFAULT_CHUNK_BITS: u32 = 16; // 65,536 slots per pool chunk

/// What one fill took from the oldest resting order at a level.
pub(crate) struct Fill {
    pub maker_id: OrderId,
    pub quantity: u32,
}

/// Orders live in `pool`; each price is a `Level`, a queue of slots into it.
pub struct OrderBook {
    pool: Pool<Order>,
    /// Resting order id -> its pool slot, so cancel needs no search.
    index: IdMap,
    bids: BTreeMap<Price, Level>,
    asks: BTreeMap<Price, Level>,
}

impl OrderBook {
    pub(crate) fn new() -> Self {
        Self::with_pool(DEFAULT_CHUNK_BITS, None)
    }

    pub(crate) fn with_pool(chunk_bits: u32, max_chunks: Option<usize>) -> Self {
        Self {
            pool: Pool::new(chunk_bits, max_chunks),
            index: IdMap::default(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// True if another order can rest. Matching only frees slots, so checking
    /// before a match guarantees its remainder can be added afterwards.
    pub(crate) fn has_room(&self) -> bool {
        self.pool.has_room()
    }

    /// Rests `order` at the back of its price level.
    pub(crate) fn add(&mut self, order: Order) -> Result<(), PoolError> {
        let (id, side, price) = (order.id(), order.side(), order.price());
        let slot = self.pool.insert(order)?;
        let duplicate = self.index.insert(id, slot);
        debug_assert!(duplicate.is_none(), "duplicate order id {id}");

        let levels = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        levels.entry(price).or_insert_with(Level::new).push_back(&mut self.pool, slot);
        Ok(())
    }

    /// Fills up to `max_qty` against the oldest order at `price` on `side`,
    /// freeing it if exhausted and dropping the level if it empties.
    /// The level must exist (callers get `price` from `best_bid`/`best_ask`).
    pub(crate) fn fill_front(&mut self, side: Side, price: Price, max_qty: u32) -> Fill {
        let levels = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        let level = levels.get_mut(&price).expect("fill_front: no level at price");
        let slot = level.head().expect("fill_front: empty level");
        let maker = self.pool.get(slot).expect("fill_front: head is not live");
        let (maker_id, quantity) = (maker.id(), max_qty.min(maker.quantity()));

        level.reduce(&mut self.pool, slot, quantity);
        if self.pool.get(slot).is_some_and(|o| o.quantity() == 0) {
            level.pop_front(&mut self.pool);
            self.pool.remove(slot);
            self.index.remove(&maker_id);
        }
        if level.is_empty() {
            levels.remove(&price);
        }
        Fill { maker_id, quantity }
    }

    /// Removes a resting order from wherever it sits in its queue and returns
    /// it, or `None` if it is not resting.
    pub(crate) fn cancel(&mut self, id: OrderId) -> Option<Order> {
        let slot = self.index.remove(&id)?;
        let order = self.pool.get(slot).expect("cancel: indexed slot is not live");
        let (side, price) = (order.side(), order.price());

        let levels = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        let level = levels.get_mut(&price).expect("cancel: no level at price");
        level.unlink(&mut self.pool, slot);
        if level.is_empty() {
            levels.remove(&price);
        }
        self.pool.remove(slot)
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    /// Resting orders at `price`, oldest first.
    #[cfg(test)]
    pub(crate) fn orders_at(&self, price: Price, side: Side) -> Vec<&Order> {
        let levels = match side {
            Side::Buy => &self.bids,
            Side::Sell => &self.asks,
        };
        levels.get(&price).map_or_else(Vec::new, |level| {
            level.iter(&self.pool).filter_map(|slot| self.pool.get(slot)).collect()
        })
    }

    pub fn validate(&self) -> Result<(), BookError> {
        let resting = self.validate_side(Side::Buy)? + self.validate_side(Side::Sell)?;
        if resting != self.pool.len() {
            return Err(BookError::PoolMismatch { resting, in_pool: self.pool.len() });
        }
        if resting != self.index.len() {
            return Err(BookError::IndexSizeMismatch { indexed: self.index.len(), resting });
        }

        if let (Some(best_bid), Some(best_ask)) = (self.best_bid(), self.best_ask()) {
            if best_bid >= best_ask {
                return Err(BookError::CrossedBook { best_bid, best_ask });
            }
        }

        Ok(())
    }

    /// Checks every level on one side; returns how many orders rest there.
    fn validate_side(&self, side: Side) -> Result<usize, BookError> {
        let levels = match side {
            Side::Buy => &self.bids,
            Side::Sell => &self.asks,
        };
        let mut resting = 0;

        for (&price, level) in levels {
            if level.is_empty() {
                return Err(BookError::EmptyLevel { side, price });
            }

            let (mut prev, mut last_id) = (NIL, 0);
            let (mut count, mut total) = (0u32, 0u64);

            for slot in level.iter(&self.pool) {
                let order = self
                    .pool
                    .get(slot)
                    .ok_or(BookError::DeadSlot { side, price, slot })?;
                let id = order.id();

                if order.quantity() == 0 {
                    return Err(BookError::ZeroQuantity { side, price, id });
                }
                if order.side() != side {
                    return Err(BookError::WrongSide { side, price, id });
                }
                if order.price() != price {
                    return Err(BookError::PriceMismatch {
                        side,
                        level_price: price,
                        order_price: order.price(),
                        id,
                    });
                }
                if self.index.get(&id) != Some(&slot) {
                    return Err(BookError::IndexMismatch { id });
                }
                // a cycle would otherwise walk forever
                if order.prev != prev || count as usize >= self.pool.len() {
                    return Err(BookError::BrokenLink { side, price, id });
                }

                (prev, last_id) = (slot, id);
                count += 1;
                total += u64::from(order.quantity());
            }

            if level.tail() != Some(prev) {
                return Err(BookError::BrokenLink { side, price, id: last_id });
            }
            if count != level.count() {
                return Err(BookError::LevelCountMismatch { side, price, cached: level.count(), actual: count });
            }
            if total != level.total_qty() {
                return Err(BookError::LevelTotalMismatch { side, price, cached: level.total_qty(), actual: total });
            }
            resting += count as usize;
        }
        Ok(resting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(id: u64, price: Price, side: Side, qty: u32) -> Order {
        Order::new(id, price, side, qty).unwrap()
    }

    #[test]
    fn full_pool_refuses_new_orders_until_a_slot_frees() {
        let mut book = OrderBook::with_pool(1, Some(1)); // 2 slots
        book.add(order(1, 100, Side::Sell, 5)).unwrap();
        book.add(order(2, 101, Side::Sell, 5)).unwrap();

        assert!(!book.has_room());
        assert_eq!(book.add(order(3, 102, Side::Sell, 5)), Err(PoolError::Full));
        assert!(book.validate().is_ok());

        book.fill_front(Side::Sell, 100, 5);
        assert!(book.has_room());
        assert!(book.add(order(4, 102, Side::Sell, 5)).is_ok());
        assert!(book.validate().is_ok());
    }

    #[test]
    fn cancel_from_the_middle_keeps_fifo_for_the_rest() {
        let mut book = OrderBook::new();
        for id in 1..=3 {
            book.add(order(id, 100, Side::Sell, 5)).unwrap();
        }

        assert_eq!(book.cancel(2).map(|o| o.id()), Some(2));
        assert!(book.validate().is_ok());
        assert_eq!(book.fill_front(Side::Sell, 100, 5).maker_id, 1);
        assert_eq!(book.fill_front(Side::Sell, 100, 5).maker_id, 3);
        assert!(book.validate().is_ok());
    }

    #[test]
    fn cancel_head_and_tail() {
        let mut book = OrderBook::new();
        for id in 1..=3 {
            book.add(order(id, 100, Side::Buy, 5)).unwrap();
        }

        assert!(book.cancel(1).is_some()); // head
        assert!(book.cancel(3).is_some()); // tail
        assert!(book.validate().is_ok());
        assert_eq!(book.orders_at(100, Side::Buy)[0].id(), 2);
    }

    #[test]
    fn cancel_only_order_drops_level_and_frees_slot() {
        let mut book = OrderBook::with_pool(1, Some(1)); // 2 slots
        book.add(order(1, 100, Side::Sell, 5)).unwrap();
        book.add(order(2, 101, Side::Sell, 5)).unwrap();

        assert!(book.cancel(1).is_some());
        assert_eq!(book.best_ask(), Some(101));
        assert!(book.has_room());
        assert!(book.add(order(3, 100, Side::Sell, 5)).is_ok());
        assert!(book.validate().is_ok());
    }

    #[test]
    fn filled_or_unknown_orders_cannot_be_cancelled() {
        let mut book = OrderBook::new();
        book.add(order(1, 100, Side::Sell, 5)).unwrap();
        book.fill_front(Side::Sell, 100, 5); // order 1 fully filled

        assert!(book.cancel(1).is_none());
        assert!(book.cancel(99).is_none());
        assert!(book.validate().is_ok());
    }

    #[test]
    fn fill_front_partial_then_full_keeps_book_valid() {
        let mut book = OrderBook::new();
        book.add(order(1, 100, Side::Sell, 5)).unwrap();
        book.add(order(2, 100, Side::Sell, 3)).unwrap();

        let fill = book.fill_front(Side::Sell, 100, 2);
        assert_eq!((fill.maker_id, fill.quantity), (1, 2));
        assert!(book.validate().is_ok());

        let fill = book.fill_front(Side::Sell, 100, 10); // takes the rest of order 1 only
        assert_eq!((fill.maker_id, fill.quantity), (1, 3));
        assert_eq!(book.orders_at(100, Side::Sell)[0].id(), 2);

        book.fill_front(Side::Sell, 100, 10);
        assert_eq!(book.best_ask(), None); // empty level dropped
        assert!(book.validate().is_ok());
    }
}
