# N-grams

[![Build Status](https://travis-ci.org/pwoolcoc/ngrams.svg)](https://travis-ci.org/pwoolcoc/ngrams)
[![Coverage Status](https://coveralls.io/repos/pwoolcoc/ngrams/badge.svg?branch=master&service=github)](https://coveralls.io/github/pwoolcoc/ngrams?branch=master)
[![](https://meritbadge.herokuapp.com/ngrams)](https://crates.io/crates/ngrams)

[Documentation](https://pwoolcoc.github.io/ngrams)

This crate takes a sequence of tokens and generates an n-gram for it.
For more information about n-grams, check wikipedia: https://en.wikipedia.org/wiki/N-gram

## Usage

Probably the easiest way to use it is to use the iterator adaptor. If
your tokens are strings (&str, String, char, or Vec<u8>), you don't have
to do anything other than generate the token stream:

```rust
use ngrams::Ngram;
let grams: Vec<_> = "one two three".split(' ').ngrams(2).collect();
// => vec![
//        vec!["\u{2060}", "one"],
//        vec!["one", "two"],
//        vec!["two", "three"],
//        vec!["three", "\u{2060}"],
//    ]
```

(re: the "\u{2060}": We use the unicode `WORD JOINER` symbol as padding on the beginning and
end of the token stream.)

If your token type isn't one of the listed types, you can still use the
iterator adaptor by implementing the `ngram::Pad` trait for your type.
