//! Ngrams
//!
//! Produce ngram sequences from inputs, and calculate the similarity
//! between two items using their ngram similarity.

#![deny(missing_docs,
       missing_debug_implementations, missing_copy_implementations,
       trivial_casts, trivial_numeric_casts,
       unsafe_code,
       unstable_features,
       unused_import_braces, unused_qualifications)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![cfg_attr(feature = "dev", deny(clippy))]


use std::fmt;
use std::collections::VecDeque;

const WORD_SEP: &'static str = "\u{2060}";

/// Main data type, implements the logic on splitting and grouping n-grams
pub struct Ngrams<'a, T: 'a + Pad + fmt::Debug + Clone> {
    source: Box<Iterator<Item = T> + 'a>,
    num: usize,
    memory: VecDeque<T>,
}

impl<'a, T: 'a + Pad + fmt::Debug + Clone> fmt::Debug for Ngrams<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ngrams(tokens, N)")
    }
}

impl<'a, T: 'a + Pad + fmt::Debug + Clone> Ngrams<'a, T> {
    /// The source for the `Ngrams` is expected to be pre-tokenized, this library
    /// does not make any decisions regarding how your input should be tokenized.
    pub fn new<V: 'a + Iterator<Item = T>>(source: V, n: usize) -> Ngrams<'a, T> {
        Ngrams {
            source: Box::new(Padded::new(source, n)),
            num: n,
            memory: VecDeque::with_capacity((n - 1)),
        }
    }

}

impl<'a, T: 'a + Pad + fmt::Debug + Clone> Iterator for Ngrams<'a, T> {
  type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Fill the memory
        while self.memory.len() < self.memory.capacity() {
            // Can I unwrap here? I need to make sure that
            // .next() can't return None before .memory is full
            let a = self.source.next().unwrap();
            self.memory.push_back(a);
        }

        let next_item = self.source.next();

        match next_item {
            None => None,
            Some(n) => {

                let mut result = Vec::with_capacity(self.num);

                for elem in &self.memory {
                    result.push(elem.clone());
                }

                result.push(n.clone());

                let _ = self.memory.pop_front();
                self.memory.push_back(n.clone());

                Some(result)
            }
        }
    }
}

/// Implement this so `ngrams` knows how to pad the beginning and end of your input.
///
/// There are default implementations for `&str`, `String`, and `Vec<u8>`
pub trait Pad {
    /// The item returned from this method will be used to pad the beginning and end of each n-gram
    fn symbol() -> Self;

    /// Specifies how many characters of padding to add. Defaults to N - 1
    fn len(n: usize) -> usize {
        n - 1
    }
}

impl<'a> Pad for &'a str {
    fn symbol() -> Self {
        WORD_SEP
    }
}

impl Pad for String {
    fn symbol() -> Self {
        WORD_SEP.to_owned()
    }
}

impl Pad for Vec<u8> {
    fn symbol() -> Self {
        WORD_SEP.to_owned().into()
    }
}

impl Pad for char {
    fn symbol() -> Self {
        WORD_SEP.chars().next().unwrap()
    }
}

struct Padded<'a, T: 'a + Pad + fmt::Debug + Clone> {
    source: Box<Iterator<Item = T> + 'a>,
    len: usize,
    symbol: T,
    remaining: usize,
    end: bool,
}

impl<'a, T: 'a + Pad + fmt::Debug + Clone> Padded<'a, T> {
    fn new<U: 'a + Iterator<Item = T>>(source: U, n: usize) -> Padded<'a, T> {
        let l = T::len(n);
        Padded {
            source: Box::new(source),
            len: l,
            symbol: T::symbol(),
            remaining: l,
            end: false,
        }
    }
}

impl<'a, T: 'a + Pad + fmt::Debug + Clone> Iterator for Padded<'a, T> {
  type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining > 0 {
            self.remaining = self.remaining - 1;
            return Some(self.symbol.clone());
        }

        let result = self.source.next();

        if result.is_none() {

            if !self.end {
                // then this is the first time
                // we have seen this return None.
                self.end = true;
                self.remaining = self.len;
            }

            if self.remaining > 0 {
                self.remaining = self.remaining - 1;
                return Some(self.symbol.clone());
            }

        }

        result
    }
}

#[cfg(test)]
mod tests {

    use super::Ngrams;
    use std::string::ToString;

    #[test]
    fn test_words() {
        let seq = "one two three four".split(' ');
        let result: Vec<_> = Ngrams::new(seq, 2).collect();
        assert_eq!(result,
                   vec![
                vec!["\u{2060}", "one"],
                vec!["one", "two"],
                vec!["two", "three"],
                vec!["three", "four"],
                vec!["four", "\u{2060}"],
            ]);
    }

    #[test]
    fn test_chars() {
        let seq = "test string".chars().map(|c| c.to_string());
        let result: Vec<_> = Ngrams::new(seq, 4).collect();
        assert_eq!(result,
                   vec![
                vec!["\u{2060}", "\u{2060}", "\u{2060}", "t"],
                vec!["\u{2060}", "\u{2060}", "t", "e"],
                vec!["\u{2060}", "t", "e", "s"],
                vec!["t", "e", "s", "t"],
                vec!["e", "s", "t", " "],
                vec!["s", "t", " ", "s"],
                vec!["t", " ", "s", "t"],
                vec![" ", "s", "t", "r"],
                vec!["s", "t", "r", "i"],
                vec!["t", "r", "i", "n"],
                vec!["r", "i", "n", "g"],
                vec!["i", "n", "g", "\u{2060}"],
                vec!["n", "g", "\u{2060}", "\u{2060}"],
                vec!["g", "\u{2060}", "\u{2060}", "\u{2060}"],
            ]);
    }
}
