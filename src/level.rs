use crate::{
    order::Order,
    pool::{NIL, Pool, Slot},
};

/// FIFO queue of the orders resting at one price. It owns no orders: it is
/// head/tail bookkeeping over a doubly linked list threaded through the pool
/// (`Order::prev` / `Order::next`).
///
/// Every order's quantity change must go through `reduce`, otherwise
/// `total_qty` drifts from the real sum.
pub(crate) struct Level {
    head: Slot, // oldest, matched first
    tail: Slot, // newest
    total_qty: u64,
    count: u32,
}

impl Level {
    pub fn new() -> Self {
        Self { head: NIL, tail: NIL, total_qty: 0, count: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.head == NIL
    }

    pub fn head(&self) -> Option<Slot> {
        (self.head != NIL).then_some(self.head)
    }

    pub fn tail(&self) -> Option<Slot> {
        (self.tail != NIL).then_some(self.tail)
    }

    pub fn total_qty(&self) -> u64 {
        self.total_qty
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    /// Queues the order at `slot` behind everything already resting here.
    pub fn push_back(&mut self, pool: &mut Pool<Order>, slot: Slot) {
        let order = pool.get_mut(slot).expect("push_back: slot is not live");
        order.prev = self.tail;
        order.next = NIL;
        let qty = order.quantity();

        if self.tail == NIL {
            self.head = slot;
        } else {
            pool.get_mut(self.tail).expect("push_back: tail is not live").next = slot;
        }
        self.tail = slot;
        self.total_qty += u64::from(qty);
        self.count += 1;
    }

    /// Detaches the oldest order and returns its slot; the caller frees it
    /// from the pool.
    pub fn pop_front(&mut self, pool: &mut Pool<Order>) -> Option<Slot> {
        let slot = self.head()?;
        self.unlink(pool, slot);
        Some(slot)
    }

    /// Detaches the order at `slot` from anywhere in the queue. `slot` must
    /// currently be queued in *this* level.
    pub fn unlink(&mut self, pool: &mut Pool<Order>, slot: Slot) {
        let order = pool.get_mut(slot).expect("unlink: slot is not live");
        let (prev, next, qty) = (order.prev, order.next, order.quantity());
        order.prev = NIL;
        order.next = NIL;

        if prev == NIL {
            self.head = next;
        } else {
            pool.get_mut(prev).expect("unlink: prev is not live").next = next;
        }
        if next == NIL {
            self.tail = prev;
        } else {
            pool.get_mut(next).expect("unlink: next is not live").prev = prev;
        }
        self.total_qty -= u64::from(qty);
        self.count -= 1;
    }

    /// Fills `amount` off the order at `slot`, keeping `total_qty` in step.
    pub fn reduce(&mut self, pool: &mut Pool<Order>, slot: Slot, amount: u32) {
        pool.get_mut(slot).expect("reduce: slot is not live").reduce_quantity(amount);
        self.total_qty -= u64::from(amount);
    }

    /// Slots oldest to newest.
    pub fn iter<'a>(&self, pool: &'a Pool<Order>) -> impl Iterator<Item = Slot> + 'a {
        std::iter::successors(self.head(), move |&slot| {
            let next = pool.get(slot)?.next;
            (next != NIL).then_some(next)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::Side;

    fn rest(pool: &mut Pool<Order>, qty: u32) -> Slot {
        let id = pool.len() as u64 + 1;
        pool.insert(Order::new(id, 100, Side::Buy, qty).unwrap()).unwrap()
    }

    /// Forward and backward walks agree, and the cached totals are the real ones.
    fn check(level: &Level, pool: &Pool<Order>) {
        let fwd: Vec<Slot> = level.iter(pool).collect();

        let mut back = Vec::new();
        let mut cur = level.tail().unwrap_or(NIL);
        while cur != NIL {
            back.push(cur);
            cur = pool.get(cur).unwrap().prev;
        }
        back.reverse();

        assert_eq!(fwd, back, "prev/next links disagree");
        assert_eq!(level.count() as usize, fwd.len());
        let sum: u64 = fwd.iter().map(|&s| u64::from(pool.get(s).unwrap().quantity())).sum();
        assert_eq!(level.total_qty(), sum);
        assert_eq!(level.is_empty(), fwd.is_empty());
        if let Some(&first) = fwd.first() {
            assert_eq!(pool.get(first).unwrap().prev, NIL);
        }
        if let Some(&last) = fwd.last() {
            assert_eq!(pool.get(last).unwrap().next, NIL);
        }
    }

    // 4 slots per chunk
    fn setup(qtys: &[u32]) -> (Pool<Order>, Level, Vec<Slot>) {
        let mut pool = Pool::new(2, None);
        let mut level = Level::new();
        let slots = qtys
            .iter()
            .map(|&q| {
                let s = rest(&mut pool, q);
                level.push_back(&mut pool, s);
                s
            })
            .collect();
        (pool, level, slots)
    }

    #[test]
    fn empty_level() {
        let (mut pool, mut level, _) = setup(&[]);
        assert!(level.is_empty());
        assert_eq!(level.pop_front(&mut pool), None);
        check(&level, &pool);
    }

    #[test]
    fn pops_in_fifo_order() {
        let (mut pool, mut level, slots) = setup(&[1, 2, 3]);
        check(&level, &pool);
        assert_eq!(level.total_qty(), 6);

        for &expected in &slots {
            assert_eq!(level.pop_front(&mut pool), Some(expected));
            check(&level, &pool);
        }
        assert!(level.is_empty());
        assert_eq!(level.total_qty(), 0);
    }

    #[test]
    fn unlink_head() {
        let (mut pool, mut level, s) = setup(&[1, 2, 3]);
        level.unlink(&mut pool, s[0]);
        check(&level, &pool);
        assert_eq!(level.iter(&pool).collect::<Vec<_>>(), vec![s[1], s[2]]);
    }

    #[test]
    fn unlink_middle() {
        let (mut pool, mut level, s) = setup(&[1, 2, 3]);
        level.unlink(&mut pool, s[1]);
        check(&level, &pool);
        assert_eq!(level.iter(&pool).collect::<Vec<_>>(), vec![s[0], s[2]]);
        assert_eq!(level.total_qty(), 4);
    }

    #[test]
    fn unlink_tail_then_push_back_still_links() {
        let (mut pool, mut level, s) = setup(&[1, 2, 3]);
        level.unlink(&mut pool, s[2]);
        check(&level, &pool);

        let new = rest(&mut pool, 9);
        level.push_back(&mut pool, new);
        check(&level, &pool);
        assert_eq!(level.iter(&pool).collect::<Vec<_>>(), vec![s[0], s[1], new]);
    }

    #[test]
    fn unlink_only_order_resets_head_and_tail() {
        let (mut pool, mut level, s) = setup(&[5]);
        level.unlink(&mut pool, s[0]);
        check(&level, &pool);
        assert!(level.is_empty());

        let again = rest(&mut pool, 2);
        level.push_back(&mut pool, again);
        check(&level, &pool);
    }

    #[test]
    fn unlinked_order_has_cleared_links() {
        let (mut pool, mut level, s) = setup(&[1, 2, 3]);
        level.unlink(&mut pool, s[1]);
        let order = pool.get(s[1]).unwrap();
        assert_eq!((order.prev, order.next), (NIL, NIL));
    }

    #[test]
    fn reduce_keeps_total_in_step() {
        let (mut pool, mut level, s) = setup(&[10, 4]);
        level.reduce(&mut pool, s[0], 3);
        check(&level, &pool);
        assert_eq!(level.total_qty(), 11);

        // fully filled head is popped with nothing left to subtract
        level.reduce(&mut pool, s[0], 7);
        assert_eq!(level.pop_front(&mut pool), Some(s[0]));
        check(&level, &pool);
        assert_eq!(level.total_qty(), 4);
    }
}
