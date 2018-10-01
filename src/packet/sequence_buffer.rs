use std::io::Result;
use std::clone::Clone;

/// Collection for storing data fof any kind.
pub struct SequenceBuffer<T>  where T: Default + Clone + Send + Sync  {
    entries: Vec<T>,
    entry_sequences: Vec<u32>,
    sequence: u16,
    size: usize,
}

impl<T> SequenceBuffer<T> where T: Default + Clone + Send + Sync {
    /// Create collection with an specific capacity.
    pub fn with_capacity(size: usize) -> Self {
        let mut entries = Vec::with_capacity(size);
        let mut entry_sequences = Vec::with_capacity(size);

        entries.resize(size, T::default());
        entry_sequences.resize(size, 0xFFFF_FFFF);

        Self {
            sequence: 0,
            size,
            entries,
            entry_sequences,
        }
    }

    /// Get entry from collection by sequence number.
    pub fn get(&self, sequence: u16) -> Option<&T> {
        let index = self.index(sequence);
        if self.entry_sequences[index] != u32::from(sequence) {
            return None;
        }

        Some(&self.entries[index])
    }

    /// Get mutable entry from collection by sequence number.
    pub fn get_mut(&mut self, sequence: u16) -> Option<&mut T> {
        let index = self.index(sequence);

        if self.entry_sequences[index] != u32::from(sequence) {
            return None;
        }

        Some(&mut self.entries[index])
    }

    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    /// Insert new entry into the collection.
    pub fn insert(&mut self, data: T, sequence: u16) -> Result<&mut T> {
        let index = self.index(sequence);

        self.entries[index] = data;
        self.entry_sequences[index] = u32::from(sequence);

        Ok(&mut self.entries[index])
    }

    /// Remove entry from collection.
    pub fn remove(&mut self, sequence: u16) {
        // TODO: validity check
        let index = self.index(sequence);
        self.entries[index] = T::default();
        self.entry_sequences[index] = 0xFFFF_FFFF;
    }

    /// checks if an certain entry exists.
    pub fn exists(&self, sequence: u16) -> bool
    {
        let index = self.index(sequence);
        if self.entry_sequences[index] != u32::from(sequence) {
            return false;
        }

        return true;
    }

    pub fn sequence(&self) -> u16 {
        self.sequence
    }

    /// Get the lenght of the collection.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks if the collection is empty.
    pub fn is_empty(&self) -> bool { self.len() > 0 }

    /// Get the total capacity of the collection.
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// converts an sequence number to an index that could be used for the inner storage.
    fn index(&self, sequence: u16) -> usize {
        (sequence % self.entries.len() as u16) as usize
    }
}