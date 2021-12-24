//! Collection for construction an inverted index from arbitrary keys to
//! arbitrary values.

use std::collections::{HashMap, LinkedList};
use std::hash::Hash;

/// Inverted index collection using a `HashMap` to `LinkedList`s.
///
/// For each key inserted, an arbitrary number of values is stored as a
/// `LinkedList`, making this collection appropritate for iterating over
/// all values associated to a single key.
pub struct InvertedIndex<K, V> {
    look_up_table: HashMap<K, LinkedList<V>>,
}

impl<K, V> InvertedIndex<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> InvertedIndex<K, V> {
        InvertedIndex {
            look_up_table: HashMap::new(),
        }
    }

    /// Insert a single value for the given key.
    ///
    /// If the key does not yet exist in the collection, it is added with
    /// the value. Otherwise, the given value is added as the last value
    /// in the list of values associated with the given key.
    pub fn insert_single(&mut self, key: K, value: V) {
        match self.look_up_table.get_mut(&key) {
            Some(values) => values.push_back(value),
            None => {
                let mut values = LinkedList::new();
                values.push_back(value);

                self.look_up_table.insert(key, values);
            }
        }
    }

    /// Insert multiple values for the given key.
    ///
    /// If the key does not yet exist in the collection, it is added with
    /// the values, in order. Otherwise, the given values are added in
    /// order at the end of the list of values already associated with the
    /// given key.
    pub fn insert_multiple(&mut self, key: K, values: impl IntoIterator<Item = V>) {
        match self.look_up_table.get_mut(&key) {
            Some(stored_values) => {
                values.into_iter().for_each(|v| stored_values.push_back(v));
            }
            None => {
                self.look_up_table.insert(key, values.into_iter().collect());
            }
        }
    }

    /// Retrieve an immutable reference to the first value for the given key.
    ///
    /// Retrieve an immutable reference to the first value added for the
    /// given key, if the key is available in the collection. Otherwise,
    /// return `None`.
    pub fn peek_first(&self, key: &K) -> Option<&V> {
        self.look_up_table.get(key)?.front()
    }

    /// Retrieve an immutable reference to all values for the given key.
    ///
    /// Retrieve an immutable reference to all values, in order of
    /// insertion, for the given key, if the key is available in the
    /// collection. Otherwise, return `None`.
    pub fn peek_all(&self, key: &K) -> Option<&LinkedList<V>> {
        self.look_up_table.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_matching {
        ( $x:expr, $y:expr ) => {{
            let are_matching = $x.iter().zip(&$y).fold(true, |acc, (a, b)| acc && a == b);
            assert!(are_matching);
        }};
    }

    #[test]
    fn test_new() {
        // When:
        let _index: InvertedIndex<char, i32> = InvertedIndex::new();
    }

    #[test]
    fn test_insert_single_peek_first() {
        // Given:
        let mut index: InvertedIndex<char, i32> = InvertedIndex::new();

        // When:
        index.insert_single('A', 65);
        let actual = index.peek_first(&'A');

        // Then:
        assert_eq!(65, *actual.unwrap());
    }

    #[test]
    fn test_new_key_insert_multiple_peek_first() {
        // Given:
        let mut index: InvertedIndex<char, i32> = InvertedIndex::new();

        // When:
        index.insert_multiple('A', vec![65, 66, 67]);
        let actual = index.peek_first(&'A');

        // Given:
        assert_eq!(65, *actual.unwrap());
    }

    #[test]
    fn test_new_key_insert_multiple_peek_all() {
        // Given:
        let mut index: InvertedIndex<char, i32> = InvertedIndex::new();

        // When:
        index.insert_multiple('A', vec![65, 66, 67]);
        let actual = index.peek_all(&'A');

        // Given:
        assert_matching!([65, 66, 67], *actual.unwrap());
    }

    #[test]
    fn test_existing_key_insert_multiple_peek_first() {
        // Given:
        let mut index: InvertedIndex<char, i32> = InvertedIndex::new();

        // When:
        index.insert_single('A', 64);
        index.insert_multiple('A', vec![65, 66, 67]);
        let actual = index.peek_first(&'A');

        // Given:
        assert_eq!(64, *actual.unwrap());
    }

    #[test]
    fn test_existing_key_insert_multiple_peek_all() {
        // Given:
        let mut index: InvertedIndex<char, i32> = InvertedIndex::new();

        // When:
        index.insert_single('A', 64);
        index.insert_multiple('A', vec![65, 66, 67]);
        let actual = index.peek_all(&'A');

        // Given:
        assert_matching!([64, 65, 66, 67], *actual.unwrap());
    }

    #[test]
    fn test_multiple_same_key_insert_single_peek_all() {
        // Given:
        let mut index: InvertedIndex<char, i32> = InvertedIndex::new();

        // When:
        index.insert_single('A', 65);
        index.insert_single('A', 66);
        index.insert_single('A', 67);
        let actual = index.peek_all(&'A');

        // Then:
        assert_matching!([65, 66, 67], *actual.unwrap());
    }

    #[test]
    fn test_multiple_different_key_insert_single_peek_all() {
        // Given:
        let mut index: InvertedIndex<char, i32> = InvertedIndex::new();

        // When:
        index.insert_single('A', 65);
        index.insert_single('B', 66);
        index.insert_single('C', 67);
        let actual = index.peek_all(&'A');

        // Then:
        assert_matching!([65], *actual.unwrap());
    }
}
