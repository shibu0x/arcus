use crate::price::Price;

#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct Order {
    price: Price,
    side: Side,
    quantity: u32,
}

impl Order {
    pub fn new(
        price: Price,
        side: Side,
        quantity: u32,
    ) -> Result<Self, &'static str> {
        if quantity == 0 {
            return Err("order quantity must be greater than zero");
        }
    
        Ok(Self {
            price,
            side,
            quantity,
        })
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
        self.quantity -= amount;
    }
}
