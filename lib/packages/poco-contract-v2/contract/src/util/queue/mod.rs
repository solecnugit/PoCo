use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::Vector;
use near_sdk::IntoStorageKey;
use std::fmt::Debug;

mod iter;

#[derive(Debug)]
pub struct CircularQueue<T>
where
    T: BorshDeserialize + BorshSerialize,
{
    buf: Vector<T>,
    head: u64,
    tail: u64,
    offset: u64,
    length: u64,
}

impl<T> CircularQueue<T>
where
    T: BorshDeserialize + BorshSerialize,
{
    pub fn new<S: IntoStorageKey>(length: u64, prefix: S) -> Self {
        let length = length + 1;

        Self {
            buf: Vector::new(prefix),
            head: 0,
            tail: 0,
            offset: 0,
            length,
        }
    }

    pub fn cap(&self) -> u64 {
        self.length
    }

    pub fn capacity(&self) -> u64 {
        self.length - 1
    }

    pub fn len(&self) -> u64 {
        (self.tail + self.cap() - self.head) % self.cap()
    }

    pub fn total_count(&self) -> u64 {
        self.offset + self.len()
    }

    pub fn is_full(&self) -> bool {
        (self.tail + 1) % self.length == self.head
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn remove_front(&mut self) {
        if self.is_empty() {
            return;
        }

        self.head = (self.head + 1) % self.length;
        self.offset += 1;
    }

    pub fn remove_back(&mut self) {
        if self.is_empty() {
            return;
        }

        self.tail = (self.tail + self.length - 1) % self.length;
    }

    pub fn push_back(&mut self, element: T) {
        if self.is_full() {
            self.head = (self.head + 1) % self.length;
            self.offset += 1;
        }

        if self.buf.len() < self.length as u32 {
            self.buf.push(element);
        } else {
            self.buf.replace(self.tail as u32, element);
        }

        self.tail = (self.tail + 1) % self.length;
    }

    pub fn get(&self, index: u64) -> Option<&T> {
        if index < self.offset || index >= self.offset + self.len() {
            return None;
        }

        let index = (self.head + index - self.offset) % self.length;

        self.buf.get(index as u32)
    }

    pub fn get_mut(&mut self, index: u64) -> Option<&mut T> {
        if index < self.offset || index >= self.offset + self.len() {
            return None;
        }

        let index = (self.head + index - self.offset) % self.length;

        self.buf.get_mut(index as u32)
    }

    pub fn replace(&mut self, index: u64, element: T) -> Option<T> {
        if index < self.offset || index >= self.offset + self.len() {
            return None;
        }

        let index = (self.head + index - self.offset) % self.length;

        Some(self.buf.replace(index as u32, element))
    }

    pub fn iter(&self) -> iter::Iter<T> {
        iter::Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> iter::IterMut<T> {
        iter::IterMut::new(self)
    }
}

impl<T> CircularQueue<T>
where
    T: BorshDeserialize + BorshSerialize + Clone,
{
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let index = self.head;
        self.head = (self.head + 1) % self.length;
        self.buf.get(index as u32).cloned()
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.tail = (self.tail + self.length - 1) % self.length;
        self.buf.get(self.tail as u32).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};
    use std::collections::VecDeque;

    #[test]
    fn test_push_back() {
        let range = 1000;
        let length = 100;

        let mut queue = CircularQueue::new(length, b"test_push_back_and_pop".to_vec());
        let mut baseline = VecDeque::new();

        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);

        for i in 0..range {
            let value = rng.gen::<u64>();

            queue.push_back(value);
            baseline.push_back(value);

            assert_eq!(queue.get(i), baseline.get(i as usize));
        }

        for i in 0..range - length {
            assert_eq!(queue.get(i), None);
        }

        for i in range..range + length {
            assert_eq!(queue.get(i), None);
        }
    }

    #[test]
    fn test_get_and_get_mut() {
        let mut queue = CircularQueue::new(3, b"test_get_and_get_mut".to_vec());

        queue.push_back(1);
        queue.push_back(2);
        queue.push_back(3);
        queue.push_back(4);

        assert_eq!(queue.get(0), None);
        assert_eq!(queue.get(1), Some(&2));
        assert_eq!(queue.get(2), Some(&3));
        assert_eq!(queue.get(3), Some(&4));
        assert_eq!(queue.get(4), None);

        assert_eq!(queue.get_mut(0), None);
        assert_eq!(queue.get_mut(1), Some(&mut 2));
        assert_eq!(queue.get_mut(2), Some(&mut 3));
        assert_eq!(queue.get_mut(3), Some(&mut 4));
        assert_eq!(queue.get_mut(4), None);
    }

    #[test]
    fn test_set() {
        let mut queue = CircularQueue::new(3, b"test_set".to_vec());

        queue.push_back(1);
        queue.push_back(2);
        queue.push_back(3);
        queue.push_back(4);

        assert_eq!(queue.replace(0, 5), None);
        assert_eq!(queue.replace(1, 5), Some(2));
        assert_eq!(queue.replace(2, 5), Some(3));
        assert_eq!(queue.replace(3, 5), Some(4));
        assert_eq!(queue.replace(4, 5), None);

        assert_eq!(queue.get(0), None);
        assert_eq!(queue.get(1), Some(&5));
        assert_eq!(queue.get(2), Some(&5));
        assert_eq!(queue.get(3), Some(&5));
        assert_eq!(queue.get(4), None);
    }

    #[test]
    fn test_iter() {
        let mut queue = CircularQueue::new(3, b"test_iter".to_vec());

        queue.push_back(1);
        queue.push_back(2);
        queue.push_back(3);
        queue.push_back(4);

        let mut iter = queue.iter();
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_mut() {
        let mut queue = CircularQueue::new(3, b"test_iter_mut".to_vec());

        queue.push_back(1);
        queue.push_back(2);
        queue.push_back(3);
        queue.push_back(4);

        for i in queue.iter_mut() {
            *i += 1;
        }

        assert_eq!(queue.get(0), None);
        assert_eq!(queue.get(1), Some(&3));
        assert_eq!(queue.get(2), Some(&4));
        assert_eq!(queue.get(3), Some(&5));
        assert_eq!(queue.get(4), None);
    }
}
