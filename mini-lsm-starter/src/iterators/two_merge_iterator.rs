#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use anyhow::{Ok, Result};

use super::StorageIterator;

/// Merges two iterators of different types into one. If the two iterators have the same key, only
/// produce the key once and prefer the entry from A.
pub struct TwoMergeIterator<A: StorageIterator, B: StorageIterator> {
    a: A,
    b: B,
    a_is_current: bool,
}

impl<
        A: 'static + StorageIterator,
        B: 'static + for<'a> StorageIterator<KeyType<'a> = A::KeyType<'a>>,
    > TwoMergeIterator<A, B>
{
    pub fn create(a: A, b: B) -> Result<Self> {
        let mut two_merged_iterator = Self {
            a_is_current: false,
            a,
            b,
        };

        two_merged_iterator.advance_b_for_duplicate()?;
        two_merged_iterator.a_is_current =
            Self::a_is_current(&two_merged_iterator.a, &two_merged_iterator.b);
        Ok((two_merged_iterator))
    }

    fn a_is_current(a: &A, b: &B) -> bool {
        if !a.is_valid() {
            return false;
        }

        if !b.is_valid() {
            return true;
        }

        a.key() < b.key()
    }

    fn advance_b_for_duplicate(&mut self) -> Result<()> {
        if self.a.is_valid() {
            if self.b.is_valid() && self.a.key() == self.b.key() {
                self.b.next()?;
            }
        }
        Ok(())
    }
}

impl<
        A: 'static + StorageIterator,
        B: 'static + for<'a> StorageIterator<KeyType<'a> = A::KeyType<'a>>,
    > StorageIterator for TwoMergeIterator<A, B>
{
    type KeyType<'a> = A::KeyType<'a>;

    fn key(&self) -> Self::KeyType<'_> {
        if self.a_is_current {
            self.a.key()
        } else {
            self.b.key()
        }
    }

    fn value(&self) -> &[u8] {
        if self.a_is_current {
            self.a.value()
        } else {
            self.b.value()
        }
    }

    fn is_valid(&self) -> bool {
        if self.a_is_current {
            self.a.is_valid()
        } else {
            self.b.is_valid()
        }
    }

    fn next(&mut self) -> Result<()> {
        if self.a_is_current {
            self.a.next()?;
        } else {
            self.b.next()?;
        }
        self.advance_b_for_duplicate()?;
        self.a_is_current = Self::a_is_current(&self.a, &self.b);
        Ok(())
    }
}
