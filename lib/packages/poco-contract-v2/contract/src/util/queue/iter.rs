use std::{iter::FusedIterator, ops::Range};

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};

use super::CircularQueue;

const ERR_INDEX_OUT_OF_BOUNDS: &str = "Index out of bounds";

#[derive(Debug)]
pub struct Iter<'a, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    queue: &'a CircularQueue<T>,
    range: Range<u64>,
}

impl<'a, T> Iter<'a, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    pub(super) fn new(queue: &'a CircularQueue<T>) -> Self {
        Self {
            queue,
            range: Range {
                start: queue.offset,
                end: queue.offset + queue.len(),
            },
        }
    }

    fn remaining(&self) -> usize {
        self.queue.len() as usize
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        <Self as Iterator>::nth(self, 0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining();
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.remaining()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let idx = self.range.nth(n)?;
        Some(
            self.queue
                .get(idx)
                .unwrap_or_else(|| near_sdk::env::panic_str(ERR_INDEX_OUT_OF_BOUNDS)),
        )
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> where T: BorshSerialize + BorshDeserialize {}

impl<'a, T> FusedIterator for Iter<'a, T> where T: BorshSerialize + BorshDeserialize {}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        <Self as DoubleEndedIterator>::nth_back(self, 0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let idx = self.range.nth_back(n)?;
        Some(
            self.queue
                .get(idx)
                .unwrap_or_else(|| near_sdk::env::panic_str(ERR_INDEX_OUT_OF_BOUNDS)),
        )
    }
}

#[derive(Debug)]
pub struct IterMut<'a, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    queue: &'a mut CircularQueue<T>,
    range: Range<u64>,
}

impl<'a, T> IterMut<'a, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    pub(crate) fn new(queue: &'a mut CircularQueue<T>) -> Self {
        let offset = queue.offset;
        let end = offset + queue.len();
        Self {
            queue,
            range: Range { start: offset, end },
        }
    }

    fn remaining(&self) -> usize {
        self.queue.len() as usize
    }
}

impl<'a, T> IterMut<'a, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn get_mut<'b>(&'b mut self, at: u64) -> Option<&'a mut T> {
        self.queue
            .get_mut(at)
            .map(|value| unsafe { &mut *(value as *mut T) })
    }
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        <Self as Iterator>::nth(self, 0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining();
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.remaining()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let idx = self.range.nth(n)?;
        Some(
            self.get_mut(idx)
                .unwrap_or_else(|| near_sdk::env::panic_str(ERR_INDEX_OUT_OF_BOUNDS)),
        )
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> where T: BorshSerialize + BorshDeserialize {}

impl<'a, T> FusedIterator for IterMut<'a, T> where T: BorshSerialize + BorshDeserialize {}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        <Self as DoubleEndedIterator>::nth_back(self, 0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let idx = self.range.nth_back(n)?;
        Some(
            self.get_mut(idx)
                .unwrap_or_else(|| near_sdk::env::panic_str(ERR_INDEX_OUT_OF_BOUNDS)),
        )
    }
}
