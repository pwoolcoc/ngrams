use std::fmt;
use std::collections::{VecDeque};

pub trait Pad {
  fn symbol() -> Self;

  fn len(n: u32) -> u32 { n - 1 }
}

impl<'a> Pad for &'a str {
  fn symbol() -> Self {
    "\u{2060}"
  }
}

impl Pad for String {
  fn symbol() -> Self {
    "\u{2060}".to_string()
  }
}

impl Pad for Vec<u8> {
  fn symbol() -> Self {
    "\u{2060}".to_string().into()
  }
}

pub struct Padded<'a, T: 'a + Pad + fmt::Debug + Clone> {
  source: Box<Iterator<Item=T> + 'a>,
  len: u32,
  symbol: T,
  remaining: u32,
  end: bool,
}

impl<'a, T: 'a + Pad + fmt::Debug + Clone> Padded<'a, T> {
  pub fn new<U: 'a + Iterator<Item=T>>(source: U, n: u32) -> Padded<'a, T>{
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

      if !self.end { // then this is the first time
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

pub struct Ngrams<'a, T: 'a + Pad + fmt::Debug + Clone> {
  source: Box<Iterator<Item=T> + 'a>,
  num: u32,
  memory: VecDeque<T>,
}

impl<'a, T: 'a + Pad + fmt::Debug + Clone> Ngrams<'a, T> {
  pub fn new<V: 'a + Iterator<Item=T>>(source: V, num: u32) -> Ngrams<'a, T> {
    Ngrams {
      source: Box::new(Padded::new(source, num)),
      num: num,
      memory: VecDeque::with_capacity(num - 1 as usize),
    }
  }

  pub fn pad_len(mut self, pad: u32) -> Ngrams<'a, T> {
    self.source = Box::new(Padded::new(self.source, pad));
    self
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
      None => return None,
      Some(n) => {

        let mut result = Vec::with_capacity(self.num as usize);

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

#[cfg(test)]
mod tests {

use super::{Ngrams, Pad};
use std::string::ToString;

#[test]
fn test_words() {
  let seq = "one two three four".split(' ');
  let result: Vec<_> = Ngrams::new(seq, 2).collect();
  assert_eq!(
    result,
    vec![
      vec!["one", "two"],
      vec!["two", "three"],
      vec!["three", "four"],
    ]
  );
}

#[test]
fn test_chars() {
  let seq = "test string".chars().map(|c| c.to_string());
  let result: Vec<_> = Ngrams::new(seq, 4).collect();
  assert_eq!(
    result,
    vec![
      vec!["t", "e", "s", "t"],
      vec!["e", "s", "t", " "],
      vec!["s", "t", " ", "s"],
      vec!["t", " ", "s", "t"],
      vec![" ", "s", "t", "r"],
      vec!["s", "t", "r", "i"],
      vec!["t", "r", "i", "n"],
      vec!["r", "i", "n", "g"],
    ]
  );
}

/*
#[test]
fn test_words_wth_pad_left() {
  let seq = "one two three four".split(' ');
  let result: Vec<_> = Ngrams::new(seq, 2)
                              .with_pad(Pad::Left("PAD"))
                              .collect();
  assert_eq!(
    result,
    vec![
      vec!["PAD", "one"],
      vec!["one", "two"],
      vec!["two", "three"],
      vec!["three", "four"],
    ]
  );
}

#[test]
fn test_words_with_pad_right() {
  let seq = "one two three four".split(' ');
  let result: Vec<_> = Ngrams::new(seq, 2)
                              .with_pad(Pad::Right("PAD"))
                              .collect();
  assert_eq!(
    result,
    vec![
      vec!["one", "two"],
      vec!["two", "three"],
      vec!["three", "four"],
      vec!["four", "PAD"],
    ]
  );
}

#[test]
fn test_words_with_pad_both() {
  let seq = "one two three four".split(' ');
  let result: Vec<_> = Ngrams::new(seq, 2)
                              .pad_len(1)
                              .collect();
  assert_eq!(
    result,
    vec![
      vec!["PAD", "one"],
      vec!["one", "two"],
      vec!["two", "three"],
      vec!["three", "four"],
      vec!["four", "PAD"],
    ]
  );
}
*/
}

