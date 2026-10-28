use crate::{
    order::{Order, Side},
    order_book::OrderBook,
    trade::{Trade, TradeHistory},
};

pub fn matching_engine(
    mut order: Order,
    orderbook: &mut OrderBook,
    trade_history: &mut TradeHistory,
) {
    match order.side() {
        Side::Buy => match_buy(&mut order, orderbook, trade_history),
        Side::Sell => match_sell(&mut order, orderbook, trade_history),
    }

    if order.quantity() > 0 {
        orderbook.add(order);
    }
}

pub fn match_buy(order: &mut Order, orderbook: &mut OrderBook, trade_history: &mut TradeHistory) {
    while order.quantity() > 0 {
        let Some(best_ask) = orderbook.best_ask() else {
            break;
        };

        if order.price() < best_ask {
            break;
        }

        if let Some(orders) = orderbook.get_mut(best_ask, Side::Sell) {
            if let Some(resting_order) = orders.first_mut() {
                let traded = order.quantity().min(resting_order.quantity());

                order.reduce_quantity(traded);
                resting_order.reduce_quantity(traded);

                let trade = Trade::new(best_ask, traded, order.side());

                trade_history.add(trade);
            }

            if orders.first().is_some_and(|o| o.quantity() == 0) {
                orders.remove(0);
            }
            if orders.is_empty() {
                orderbook.remove(best_ask, Side::Sell);
            }
        } else {
            orderbook.remove(best_ask, Side::Sell);
            continue;
        }
    }
}

pub fn match_sell(order: &mut Order, orderbook: &mut OrderBook, trade_history: &mut TradeHistory) {
    while order.quantity() > 0 {
        let Some(best_bid) = orderbook.best_bid() else {
            break;
        };

        if order.price() > best_bid {
            break;
        }

        if let Some(orders) = orderbook.get_mut(best_bid, Side::Buy) {
            if let Some(resting_order) = orders.first_mut() {
                let traded = order.quantity().min(resting_order.quantity());

                order.reduce_quantity(traded);
                resting_order.reduce_quantity(traded);

                let trade = Trade::new(best_bid, traded, order.side());

                trade_history.add(trade);
            }

            if orders.first().is_some_and(|o| o.quantity() == 0) {
                orders.remove(0);
            }

            if orders.is_empty() {
                orderbook.remove(best_bid, Side::Buy);
            }
        } else {
            orderbook.remove(best_bid, Side::Buy);
            continue;
        }
    }
}
