use crate::{error::OrderError, price::Price};

pub type OrderId =  u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct Order {
    id: OrderId,
    price: Price,
    side: Side,
    quantity: u32,
}

impl Order {
    pub(crate) fn new(
        id: OrderId,
        price: Price,
        side: Side,
        quantity: u32,
    ) -> Result<Self, OrderError> {
        if quantity == 0 {
            return Err(OrderError::ZeroQuantity);
        }
    
        Ok(Self {
            id,
            price,
            side,
            quantity,
        })
    }
    
    pub fn id(&self) -> OrderId {
        self.id
    }

    pub fn price(&self) -> Price {
        self.price
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub(crate) fn reduce_quantity(&mut self, amount: u32) {
        self.quantity = self.quantity.checked_sub(amount).unwrap();
    }
}
