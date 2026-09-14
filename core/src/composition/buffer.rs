use std::{
    fmt,
    ops::{Index, IndexMut},
};

/// An editable list of items with a cursor.
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Buffer<T> {
    items: Vec<T>,
    cursor: usize,
}

impl<T> Index<usize> for Buffer<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl<T> IndexMut<usize> for Buffer<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}

impl<T: fmt::Display> fmt::Display for Buffer<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            item.fmt(formatter)?;
        }

        Ok(())
    }
}

impl<T> Buffer<T> {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            items: Vec::new(),
            cursor: 0,
        }
    }

    /// Constructs a new, empty buffer with the specified initial capacity.
    #[inline]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            cursor: 0,
        }
    }

    #[inline]
    pub(crate) fn reserve(&mut self, additional: usize) {
        self.items.reserve(additional);
    }

    /// Resizes the buffer, cloning `value` when new items are needed.
    #[inline]
    pub(crate) fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.items.resize(new_len, value);
    }

    /// The total number of items currently in the buffer.
    #[inline]
    pub const fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the buffer holds no items.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The items currently in the buffer.
    #[inline]
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// The cursor position in the buffer.
    #[inline]
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    /// Inserts an item at the cursor and moves the cursor forward.
    #[inline]
    pub(crate) fn push(&mut self, item: T) {
        self.items.insert(self.cursor, item);
        self.cursor += 1;
    }

    /// Inserts items at the cursor and moves the cursor past them.
    #[inline]
    pub(crate) fn extend<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = T>,
    {
        let previous_len = self.items.len();

        self.items.splice(self.cursor..self.cursor, items);

        let inserted = self.items.len() - previous_len;
        self.cursor += inserted;
    }

    /// Removes the item before the cursor, if any.
    #[inline]
    pub(crate) fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.items.remove(self.cursor);
        }
    }

    /// Removes the item at the cursor, if any.
    #[inline]
    pub(crate) fn delete(&mut self) {
        if self.cursor < self.items.len() {
            self.items.remove(self.cursor);
        }
    }

    /// Moves the cursor one position left, clamped at the start.
    #[inline]
    pub(crate) const fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Moves the cursor one position right, clamped at the end.
    #[inline]
    pub(crate) fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.items.len());
    }

    /// Empties the buffer and resets the cursor.
    #[inline]
    pub(crate) fn clear(&mut self) {
        self.items.clear();
        self.cursor = 0;
    }
}
