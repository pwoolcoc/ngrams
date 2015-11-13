use std::fmt;
use std::collections::{VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum Pad<T: fmt::Debug + Clone> {
  Left(T),
  Right(T),
  Both(T),
}

pub struct PadWrapper<'a, T: 'a + fmt::Debug + Clone> {
  source: Box<Iterator<Item=T> + 'a>,
  pad: Pad<T>,
  start: bool,
  end: bool,
}

impl<'a, T: 'a + fmt::Debug + Clone> PadWrapper<'a, T> {
  pub fn new(source: Box<Iterator<Item=T> + 'a>, pad: Pad<T>) -> PadWrapper<'a, T>{
    PadWrapper {
      source: source,
      pad: pad,
      start: true,
      end: false,
    }
  }
}

impl<'a, T: 'a + fmt::Debug + Clone> Iterator for PadWrapper<'a, T> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    if self.start {
      match &self.pad {
        &Pad::Left(ref e) | &Pad::Both(ref e) => {
          self.start = false;
          return Some(e.clone());
        },
        _ => { }
      }
    }

    let result = self.source.next();

    if result.is_none() {
      if !self.end { // then this is the first time
                     // we have seen this return None.
        self.end = true;
        match &self.pad {
          &Pad::Right(ref e) | &Pad::Both(ref e) => {
            return Some(e.clone());
          },
          _ => { }
        }
      } else {
        return None;
      }
    }

    result
  }
}

pub struct Ngrams<'a, T: 'a + fmt::Debug + Clone> {
  source: Box<Iterator<Item=T> + 'a>,
  num: usize,
  memory: VecDeque<T>,
}

impl<'a, T: 'a + fmt::Debug + Clone> Ngrams<'a, T> {
  pub fn new<V: 'a + Iterator<Item=T>>(source: V, num: usize) -> Ngrams<'a, T> {
    Ngrams {
      source: Box::new(source),
      num: num,
      memory: VecDeque::with_capacity(num - 1),
    }
  }

  pub fn with_pad(mut self, pad: Pad<T>) -> Ngrams<'a, T> {
    self.source = Box::new(PadWrapper::new(self.source, pad));
    self
  }
}

impl<'a, T: 'a + fmt::Debug + Clone> Iterator for Ngrams<'a, T> {
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
                              .with_pad(Pad::Both("PAD"))
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
}

