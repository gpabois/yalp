use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use itertools::Itertools;

/// A fixed-size array
#[derive(Debug)]
pub struct Array<const N: usize, T> {
    pub cursor: usize,
    pub data: [T; N],
}

impl<const N: usize, T> PartialEq for Array<N, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        !self.iter().zip(other.iter()).any(|(a, b)| *a != *b)
    }
}

impl<const N: usize, T> Hash for Array<N, T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|e| e.hash(state));
    }
}

impl<const N: usize, T> Eq for Array<N, T> where T: Eq {}

impl<const N: usize, T> Clone for Array<N, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<const N: usize, T> std::fmt::Display for Array<N, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if N > 1 {
            write!(f, "({})", self.iter().map(ToString::to_string).join(", "))
        } else {
            write!(f, "{}", self.iter().map(ToString::to_string).join(", "))
        }
    }
}

impl<const N: usize, T> FromIterator<T> for Array<N, T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let mut arr = Self::default();

        for item in iter.into_iter() {
            if arr.is_full() {
                break;
            }

            arr.push(item);
        }

        arr
    }
}

impl<const N: usize, T> Default for Array<N, T> {
    fn default() -> Self {
        Self {
            cursor: 0,
            data: unsafe { std::mem::zeroed() },
        }
    }
}

impl<const N: usize, T> Deref for Array<N, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data[0..self.cursor]
    }
}

impl<const N: usize, T> DerefMut for Array<N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data[0..self.cursor]
    }
}

impl<const N: usize, T> Array<N, T> {
    /// Push a new element in the array
    /// # Panics
    /// This method panics if the array is full.
    pub fn push(&mut self, value: T) {
        if self.cursor >= N {
            panic!("fixed-size array is full");
        }
        self.data[self.cursor] = value;
        self.cursor += 1;
    }

    /// The current length of the array
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.cursor
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.len() == N
    }
}

#[cfg(test)]
mod tests {
    use super::Array;

    #[test]
    fn test_001_collect() {
        let array: Array<3, i32> = [1, 2, 3, 4, 5].into_iter().collect();

        let values: Vec<i32> = array.iter().cloned().collect();
        let expected_values = vec![1, 2, 3];

        assert_eq!(values, expected_values);
    }
}

