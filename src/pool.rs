use crate::error::PoolError;

pub type Slot = u32;

pub(crate) const NIL: Slot = Slot::MAX;

enum Entry<T> {
    Free { next: Slot },
    Occupied(T),
}

pub struct Pool<T> {
    chunks: Vec<Box<[Entry<T>]>>,
    chunk_bits: u32,
    free_head: Slot,
    len: usize,
    max_chunks: Option<usize>,
}

impl<T> Pool<T> {
    pub fn new(chunk_bits: u32, max_chunks: Option<usize>) -> Self {
        assert!(
            matches!(chunk_bits, 1..=24),
            "chunks bit should be in range of 1..=24"
        );
        Self {
            chunks: Vec::new(),
            chunk_bits,
            free_head: NIL,
            len: 0,
            max_chunks,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.chunks.len() << self.chunk_bits
    }

    /// True if `insert` would succeed right now.
    pub fn has_room(&self) -> bool {
        self.free_head != NIL
            || (self.max_chunks.is_none_or(|max| self.chunks.len() < max)
                && (self.chunks.len() + 1) << self.chunk_bits <= NIL as usize)
    }

    pub fn insert(&mut self, value: T) -> Result<Slot, PoolError> {
        if self.free_head == NIL {
            self.grow()?;
        }

        let slot = self.free_head;
        let (chunk, idx) = self.locate(slot);
        let entry = &mut self.chunks[chunk][idx];
        match std::mem::replace(entry, Entry::Occupied(value)) {
            Entry::Free { next } => self.free_head = next,
            Entry::Occupied(_) => unreachable!("free list points at an occupied slot"),
        }
        self.len += 1;
        Ok(slot)
    }

    pub fn remove(&mut self, slot: Slot) -> Option<T> {
        let (chunk, idx) = self.locate(slot);
        let entry = self.chunks.get_mut(chunk)?.get_mut(idx)?;
        if matches!(entry, Entry::Free { .. }) {
            return None;
        }

        let old = std::mem::replace(
            entry,
            Entry::Free {
                next: self.free_head,
            },
        );
        self.free_head = slot;
        self.len -= 1;
        match old {
            Entry::Occupied(value) => Some(value),
            Entry::Free { .. } => unreachable!(),
        }
    }

    pub fn get(&self, slot: Slot) -> Option<&T> {
        let (chunk, idx) = self.locate(slot);
        match self.chunks.get(chunk)?.get(idx)? {
            Entry::Occupied(value) => Some(value),
            Entry::Free { .. } => None,
        }
    }

    pub fn get_mut(&mut self, slot: Slot) -> Option<&mut T> {
        let (chunk, idx) = self.locate(slot);
        match self.chunks.get_mut(chunk)?.get_mut(idx)? {
            Entry::Occupied(value) => Some(value),
            Entry::Free { .. } => None,
        }
    }

    fn locate(&self, slot: Slot) -> (usize, usize) {
        let mask = (1usize << self.chunk_bits) - 1;
        ((slot as usize) >> self.chunk_bits, (slot as usize) & mask)
    }

    fn grow(&mut self) -> Result<(), PoolError> {
        if self.max_chunks.is_some_and(|max| self.chunks.len() >= max){
            return Err(PoolError::Full)
        }

        let size = 1usize << self.chunk_bits;
        let base = self.chunks.len() << self.chunk_bits;
        if base + size > NIL as usize {
            return Err(PoolError::Full);
        }

        let chunk = (0..size)
            .map(|i| Entry::Free {
                next: if i + 1 < size {
                    (base + i + 1) as Slot
                } else {
                    NIL
                },
            })
            .collect();
        self.chunks.push(chunk);
        self.free_head = base as Slot;
        Ok(())
    }
}
