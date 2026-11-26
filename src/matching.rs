use crate::{
    order::{Order, Side},
    order_book::OrderBook,
    trade::{Timestamp, Trade, TradeHistory, TradeId},
};

/// Matches `order` against the book, then rests any remainder. The caller must
/// have checked `orderbook.has_room()`: matching only frees slots, so the
/// remainder always fits.
pub fn matching_engine(
    mut order: Order,
    orderbook: &mut OrderBook,
    trade_history: &mut TradeHistory,
    next_trade_id: &mut TradeId,
    timestamp: Timestamp,
) {
    match order.side() {
        Side::Buy => match_buy(&mut order, orderbook, trade_history, next_trade_id, timestamp),
        Side::Sell => match_sell(&mut order, orderbook, trade_history, next_trade_id, timestamp),
    }

    if order.quantity() > 0 {
        orderbook.add(order).expect("room was checked before matching");
    }
}

pub fn match_buy(
    order: &mut Order,
    orderbook: &mut OrderBook,
    trade_history: &mut TradeHistory,
    next_trade_id: &mut TradeId,
    timestamp: Timestamp,
) {
    while order.quantity() > 0 {
        let Some(best_ask) = orderbook.best_ask() else {
            break;
        };

        if order.price() < best_ask {
            break;
        }

        let fill = orderbook.fill_front(Side::Sell, best_ask, order.quantity());
        order.reduce_quantity(fill.quantity);

        trade_history.add(Trade::new(
            *next_trade_id,
            best_ask,
            fill.quantity,
            order.side(),
            fill.maker_id,
            order.id(),
            timestamp,
        ));
        *next_trade_id += 1;
    }
}

pub fn match_sell(
    order: &mut Order,
    orderbook: &mut OrderBook,
    trade_history: &mut TradeHistory,
    next_trade_id: &mut TradeId,
    timestamp: Timestamp,
) {
    while order.quantity() > 0 {
        let Some(best_bid) = orderbook.best_bid() else {
            break;
        };

        if order.price() > best_bid {
            break;
        }

        let fill = orderbook.fill_front(Side::Buy, best_bid, order.quantity());
        order.reduce_quantity(fill.quantity);

        trade_history.add(Trade::new(
            *next_trade_id,
            best_bid,
            fill.quantity,
            order.side(),
            fill.maker_id,
            order.id(),
            timestamp,
        ));
        *next_trade_id += 1;
    }
}

#[cfg(test)]
mod tests;
