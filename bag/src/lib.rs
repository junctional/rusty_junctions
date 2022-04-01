//! Arbitrary key-value bag collection.
//!
//! Provides a new collection struct that can hold an arbitrary number
//! of values for a given key and allows to retrieve them in FIFO order.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// Arbitrary key-value bag.
///
/// Stores a `HashMap` of keys to `VecDeque` of values, which allows it to
/// store and arbitrary amount of values for any given key and retrieve
/// them in FIFO order.
pub struct Bag<K, V> {
    items: HashMap<K, VecDeque<V>>,
}

impl<K, V> Bag<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Bag<K, V> {
        Bag {
            items: HashMap::new(),
        }
    }

    /// Add new key-value pair.
    ///
    /// If the given key already exists, the given value is added last in
    /// a `VecDeque` for the given key. Otherwise, the new key and value
    /// are inserted.
    pub fn add(&mut self, key: K, item: V) {
        match self.items.get_mut(&key) {
            Some(queue) => {
                queue.push_back(item);
            }
            None => {
                let mut queue = VecDeque::new();
                queue.push_back(item);

                self.items.insert(key, queue);
            }
        }
    }

    /// Retrieve the first value available for the given key, if possible.
    ///
    /// Retrieve `Some` of the least recently added value for the given key
    /// if there is at least one available, otherwise return `None`.
    pub fn retrieve(&mut self, key: &K) -> Option<V> {
        self.items.get_mut(key)?.pop_front()
    }

    /// Return true if there are values for the given key.
    pub fn contains_items(&self, key: &K) -> bool {
        self.items.get(key).map_or(false, |q| !q.is_empty())
    }

    /// Return the number of values stored for the given key.
    pub fn count_items(&self, key: &K) -> usize {
        self.items.get(key).map_or(0, |q| q.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        // When:
        let _bag: Bag<usize, char> = Bag::new();
    }

    #[test]
    fn test_retrieving_non_existent() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        let actual = bag.retrieve(&42);

        // Then:
        assert!(actual.is_none());
    }

    #[test]
    fn test_retrieving_existing() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');

        let actual = bag.retrieve(&217);

        // Then:
        assert_eq!('O', actual.unwrap());
    }

    #[test]
    fn test_retrieving_first() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');
        bag.add(217, 'v');

        let actual = bag.retrieve(&217);

        // Then:
        assert_eq!('O', actual.unwrap());
    }

    #[test]
    fn test_retrieving_all_fifo_order() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');
        bag.add(217, 'v');

        let first = bag.retrieve(&217);
        let second = bag.retrieve(&217);

        // Then:
        assert_eq!('O', first.unwrap());
        assert_eq!('v', second.unwrap());
    }

    #[test]
    fn test_assigned_correct_key() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');
        bag.add(237, 'v');

        let first = bag.retrieve(&217);
        let second = bag.retrieve(&217);

        // Then:
        assert_eq!('O', first.unwrap());
        assert!(second.is_none());
    }

    #[test]
    fn test_contains_items_with_no_item() {
        // Given:
        let bag: Bag<usize, char> = Bag::new();

        // Then:
        assert!(!bag.contains_items(&217));
    }

    #[test]
    fn test_contains_items_with_single_item() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');

        // Then:
        assert!(bag.contains_items(&217));
    }

    #[test]
    fn test_contains_items_with_multiple_items() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');
        bag.add(217, 'v');
        bag.add(217, 'e');

        // Then:
        assert!(bag.contains_items(&217));
    }

    #[test]
    fn test_contains_items_with_unknown_key() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');

        // Then:
        assert!(!bag.contains_items(&42));
    }

    #[test]
    fn test_contains_items_after_add_retrieve() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');
        let _v = bag.retrieve(&217);

        // Then:
        assert!(!bag.contains_items(&217));
    }

    #[test]
    fn test_count_items_with_no_item() {
        // Given:
        let bag: Bag<usize, char> = Bag::new();

        // Then:
        assert_eq!(0, bag.count_items(&217));
    }

    #[test]
    fn test_count_items_with_single_item() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');

        // Then:
        assert_eq!(1, bag.count_items(&217));
    }

    #[test]
    fn test_count_items_with_multiple_items() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');
        bag.add(217, 'v');
        bag.add(217, 'e');

        // Then:
        assert_eq!(3, bag.count_items(&217));
    }

    #[test]
    fn test_count_items_with_unknown_key() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');

        // Then:
        assert_eq!(0, bag.count_items(&42));
    }

    #[test]
    fn test_count_items_after_add_retrieve() {
        // Given:
        let mut bag: Bag<usize, char> = Bag::new();

        // When:
        bag.add(217, 'O');
        let _v = bag.retrieve(&217);

        // Then:
        assert_eq!(0, bag.count_items(&217));
    }
}
