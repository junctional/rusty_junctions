//! Light-weight BigUint implementation to allow `Controller` to keep track of the
//! last time a particular Join Pattern has been fired.
//!
//! Note that there are publicly available crates, but given the state of stability
//! and the relative lack of features actually required by the `Controller`, a
//! dependency on those appears unwise and unnecessary.

use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::convert::From;
use std::default::Default;
use std::vec::Vec;

/// Type alias for unsigned integer type used. Makes switching trivial.
///
/// All that is required for the unsigned integer type is that is provides
/// a `max_value` and a `min_value` function.
type Uint = u64;

/// Minimum value of used unsigned integer type.
const UINT_MIN: Uint = Uint::min_value();

/// Maximum value of used unsigned integer type.
const UINT_MAX: Uint = Uint::max_value();

/// Constant for the value of a carry variable when no carry occurs.
const NO_CARRY: Uint = 0;

/// Constant for the value of a carry variable when a carry occurs.
const CARRY: Uint = 1;

/// Incrementable, dynamically resizing counter.
///
/// Effectively a light version of a BigUint, with only the ability to increment
/// and compare to other `Counter` instances.
///
/// In this implementation, the most significant part of the counter value is kept
/// at the highest index, i.e. the counter value is stored in a Little-Endian way.
///
/// The counter value is stored in a Little-Endian way to keep the cost of growing
/// the internal representation using `Vec` small. Pushing a value at the back of
/// a `Vec` may require resizing, but never moving existing elements.
#[derive(Clone, Eq, Debug)]
pub(crate) struct Counter {
    digits: Vec<Uint>,
}

impl Counter {
    /// Increment the `Counter`.
    ///
    /// Increments the `Counter` and dynamically grows it if all of its digits
    /// have reached their maximum values.
    pub(crate) fn increment(&mut self) {
        // Increment the lowest digit by a normal carry.
        let mut carry = CARRY;

        for e in &mut self.digits {
            if *e < UINT_MAX {
                *e += carry;
                carry = NO_CARRY;

                // Since we don't carry, we don't need to look at
                // other digits.
                break;
            } else {
                *e = UINT_MIN;
                carry = CARRY;
            }
        }

        // If we carried in the last digit, we need to add a new with the
        // carried value.
        if carry == CARRY {
            self.digits.push(carry);
        }
    }
}

impl Default for Counter {
    /// Create a new `Counter`, initialized to the lowest possible value.
    ///
    /// By default, the parts within the `Counter` are always initialized
    /// with capacity 1. This is because a priori, it is unlikely that
    /// any given `Counter` exceeds the maximum possible value and so we aim
    /// to conserve space initially.
    fn default() -> Self {
        let mut digits = Vec::with_capacity(1);
        digits.push(UINT_MIN);

        Counter { digits }
    }
}

impl From<Vec<Uint>> for Counter {
    /// Create `Counter` from a vector of values.
    ///
    /// Note that this function mainly exists to be able to initialize a
    /// `Counter` from any number and then perform tests on its handling
    /// of an overflow situation.
    fn from(digits: Vec<Uint>) -> Self {
        Counter { digits }
    }
}

impl PartialEq for Counter {
    /// Return true if the two `Counter`s are equal.
    ///
    /// Two `Counter`s are equal if and only if they are of the same length and
    /// each of the digits are equal. Otherwise, they are not equal.
    fn eq(&self, other: &Counter) -> bool {
        if self.digits.len() != other.digits.len() {
            false
        } else {
            self.digits
                .iter()
                .zip(other.digits.iter())
                .all(|(a, b)| a == b)
        }
    }
}

impl Ord for Counter {
    /// Compare the two `Counter`s and return a full `Ordering`.
    ///
    /// The rules for ordering are as follows:
    /// 1. If the first `Counter`'s digits are fewer, it is less, since the
    /// more digits the greater the counter value.
    /// 2. If the first `Counter`'s digits are more, it is greater, since the
    /// more digits the greater the counter value.
    /// 3. If both `Counter`s have an equal number of digits, then compare each
    /// digit from left (lowest index) to right (greatest index) and choose the
    /// last Ordering that is not `Ordering::Equal`. If there is `None`, then
    /// the two `Counter`s are equal.
    fn cmp(&self, other: &Self) -> Ordering {
        if self.digits.len() < other.digits.len() {
            Ordering::Less
        } else if self.digits.len() > other.digits.len() {
            Ordering::Greater
        } else {
            let comparison = self
                .digits
                .iter()
                .zip(other.digits.iter())
                .map(|(a, b)| a.cmp(b))
                .filter(|&v| v != Ordering::Equal)
                .last();

            match comparison {
                Some(cmp) => cmp,
                None => Ordering::Equal,
            }
        }
    }
}

impl PartialOrd for Counter {
    /// Compare the two `Counter`s and return a partial `Ordering`.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod creation {
        use super::*;

        #[test]
        fn test_create_with_default() {
            // When:
            let _a = Counter::default();
        }

        #[test]
        fn test_default_is_initial_digit_value() {
            // Given:
            let actual = Counter::default();

            // Then:
            let expected = Counter::from(vec![UINT_MIN]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_create_with_from_single_digit() {
            // When:
            let _a = Counter::from(vec![1729]);
        }

        #[test]
        fn test_create_with_from_multiple_digits() {
            // When:
            let _a = Counter::from(vec![1729, 42, 69]);
        }
    }

    mod increment {
        use super::*;

        #[test]
        fn test_increment_initial_value() {
            // Given:
            let mut actual = Counter::from(vec![UINT_MIN]);

            // When:
            actual.increment();

            // Then:
            let expected = Counter::from(vec![UINT_MIN + 1]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_increment_one_below_max_value() {
            // Given:
            let mut actual = Counter::from(vec![UINT_MAX - 1]);

            // When:
            actual.increment();

            // Then:
            let expected = Counter::from(vec![UINT_MAX]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_increment_one_digit_overflow() {
            // Given:
            let mut actual = Counter::from(vec![UINT_MAX]);

            // When:
            actual.increment();

            // Then:
            let expected = Counter::from(vec![UINT_MIN, UINT_MIN + 1]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_increment_one_digit_no_overflow() {
            // Given:
            let mut actual = Counter::from(vec![1729]);

            // When:
            actual.increment();

            // Then:
            let expected = Counter::from(vec![1730]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_increment_two_digit_overflow() {
            // Given:
            let mut actual = Counter::from(vec![UINT_MAX, UINT_MAX]);

            // When:
            actual.increment();

            // Then:
            let expected = Counter::from(vec![UINT_MIN, UINT_MIN, UINT_MIN + 1]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_increment_one_below_max_value_two_digits() {
            // Given:
            let mut actual = Counter::from(vec![UINT_MAX - 1, 42]);

            // When:
            actual.increment();

            // Then:
            let expected = Counter::from(vec![UINT_MAX, 42]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_increment_max_value_two_digits() {
            // Given:
            let mut actual = Counter::from(vec![UINT_MAX, 42]);

            // When:
            actual.increment();

            // Then:
            let expected = Counter::from(vec![UINT_MIN, 43]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_increment_two_digits_no_overflow() {
            // Given:
            let mut actual = Counter::from(vec![UINT_MAX - 1, UINT_MAX]);

            // When:
            actual.increment();

            // Then:
            let expected = Counter::from(vec![UINT_MAX, UINT_MAX]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_increment_three_digits_overflow() {
            // Given:
            let mut actual = Counter::from(vec![UINT_MAX, UINT_MAX, 42]);

            // When:
            actual.increment();

            // Then:
            let expected = Counter::from(vec![UINT_MIN, UINT_MIN, 43]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_increment_three_digits_no_overflow() {
            // Given:
            let mut actual = Counter::from(vec![1729, UINT_MAX, 42]);

            // When:
            actual.increment();

            // Then:
            let expected = Counter::from(vec![1730, UINT_MAX, 42]);
            assert_eq!(expected, actual);
        }
    }

    mod ordering {
        use super::*;

        #[test]
        fn test_eq_same() {
            // Given:
            let a = Counter::from(vec![1, 2, 3]);
            let b = Counter::from(vec![1, 2, 3]);

            // Then:
            assert_eq!(a, b);
        }

        #[test]
        fn test_neq_same_length() {
            // Given:
            let a = Counter::from(vec![1, 2, 4]);
            let b = Counter::from(vec![1, 2, 3]);

            // Then:
            assert!(a != b);
        }

        #[test]
        fn test_neq_differnt_length() {
            // Given:
            let a = Counter::from(vec![1, 2]);
            let b = Counter::from(vec![1, 2, 3]);

            // Then:
            assert!(a != b);
        }

        #[test]
        fn test_lt_same_num_digits_true() {
            // Given:
            let a = Counter::from(vec![9, 0]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(a < b);
        }

        #[test]
        fn test_lt_same_num_digits_false() {
            // Given:
            let a = Counter::from(vec![9, 0]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(!(b < a));
        }

        #[test]
        fn test_lt_diff_num_digits_true() {
            // Given:
            let a = Counter::from(vec![9]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(a < b);
        }

        #[test]
        fn test_lt_diff_num_digits_false() {
            // Given:
            let a = Counter::from(vec![9]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(!(b < a));
        }

        #[test]
        fn test_leq_same_num_digits_eq_true() {
            // Given:
            let a = Counter::from(vec![0, 1]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(a <= b);
        }

        #[test]
        fn test_leq_same_num_digits_lt_true() {
            // Given:
            let a = Counter::from(vec![0, 1]);
            let b = Counter::from(vec![0, 2]);

            // Then:
            assert!(a <= b);
        }

        #[test]
        fn test_leq_same_num_digits_false() {
            // Given:
            let a = Counter::from(vec![0, 2]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(!(a <= b));
        }

        #[test]
        fn test_leq_diff_num_digits_true() {
            // Given:
            let a = Counter::from(vec![9]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(a <= b);
        }

        #[test]
        fn test_leq_diff_num_digits_false() {
            // Given:
            let a = Counter::from(vec![9]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(!(b <= a));
        }

        #[test]
        fn test_gt_same_num_digits_true() {
            // Given:
            let a = Counter::from(vec![0, 2]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(a > b);
        }

        #[test]
        fn test_gt_same_num_digits_false() {
            // Given:
            let a = Counter::from(vec![0, 1]);
            let b = Counter::from(vec![0, 2]);

            // Then:
            assert!(!(a > b));
        }

        #[test]
        fn test_gt_diff_num_digits_true() {
            // Given:
            let a = Counter::from(vec![9]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(b > a);
        }

        #[test]
        fn test_gt_diff_num_digits_false() {
            // Given:
            let a = Counter::from(vec![9]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(!(a > b));
        }

        #[test]
        fn test_geq_same_num_digits_eq_true() {
            // Given:
            let a = Counter::from(vec![0, 1]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(b >= a);
        }

        #[test]
        fn test_geq_same_num_digits_gt_true() {
            // Given:
            let a = Counter::from(vec![0, 1]);
            let b = Counter::from(vec![0, 2]);

            // Then:
            assert!(b >= a);
        }

        #[test]
        fn test_geq_same_num_digits_false() {
            // Given:
            let a = Counter::from(vec![0, 2]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(!(b >= a));
        }

        #[test]
        fn test_geq_diff_num_digits_true() {
            // Given:
            let a = Counter::from(vec![9]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(b >= a);
        }

        #[test]
        fn test_geq_diff_num_digits_false() {
            // Given:
            let a = Counter::from(vec![9]);
            let b = Counter::from(vec![0, 1]);

            // Then:
            assert!(!(a >= b));
        }
    }

    /// Verify that `eq` is an equivalence relation.
    mod eq_relation {
        use super::*;

        #[test]
        fn test_reflexifity() {
            // Given:
            let a = Counter::from(vec![1729]);

            // a == a
            assert!(a == a);
        }

        #[test]
        fn test_symmetry() {
            // Given
            let a = Counter::from(vec![1, 2, 3]);
            let b = Counter::from(vec![1, 2, 3]);

            // a == b implies b == a
            assert!((a != b) || (b == a));
        }

        #[test]
        fn test_transitivity() {
            // Given
            let a = Counter::from(vec![1, 2, 3]);
            let b = Counter::from(vec![1, 2, 3]);
            let c = Counter::from(vec![1, 2, 3]);

            // a == b and b == c implies a == c
            assert!((a != b) || (b != c) || (a == c));
        }
    }
}
