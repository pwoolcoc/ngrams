extern crate ngrams;
extern crate rand;
extern crate flate2;

use ngrams::Ngrams;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use flate2::read::GzDecoder;

use rand::distributions::{Weighted, WeightedChoice, IndependentSample};

#[derive(Debug, Clone, PartialEq)]
struct Markov {
    data: HashMap<String, HashMap<String, u32>>,
}

fn extract_sentence(s: String) -> String {
    s.split('\t').nth(1).unwrap().into()
}

fn tokenize_sentence(sentence: String) -> Vec<String> {
    let tokens: Vec<String> = sentence.split(|c| {
        match c {
            '"' | ',' | ';' | ':' => true,
            a if a.is_whitespace() => true,
            _ => false
        }
    }).filter(|a| a.len() > 0).map(|a| a.trim().to_owned()).collect();
    tokens
}

impl Markov {
    fn new(fname: &'static str) -> Markov {
        let mut map: HashMap<String, HashMap<String, u32>> = HashMap::new();
        let file = BufReader::new(
                        GzDecoder::new(
                            File::open(fname).unwrap()).unwrap());
        let sentences = file.lines().map(|a| a.unwrap()).map(extract_sentence).map(tokenize_sentence);
        for tokenized in sentences {
            let grams = Ngrams::new(tokenized.into_iter(), 2);
            for gram in grams {
                let first = gram[0].clone();
                let second = gram[1].clone();
                let entry = map.entry(first).or_insert(HashMap::new());
                let entry = entry.entry(second).or_insert(0);
                *entry += 1;
            }
        }
        Markov {
            data: map
        }
    }

    fn random_word(&self, s: &str) -> String {
        match self.data.get(s) {
            Some(h) => {
                let mut choices = vec![];
                for (word, count) in h {
                    choices.push(Weighted { weight: *count, item: word });
                }
                let wc = WeightedChoice::new(&mut choices);
                let mut rng = rand::thread_rng();
                wc.ind_sample(&mut rng).clone()
            },
                None => "whoops...".to_owned()
        }
    }

    fn sentence_generator(&self) -> SentenceGenerator {
        SentenceGenerator {
            markov: self.clone(),
            state: "\u{2060}".to_owned(),
        }
    }
}

struct SentenceGenerator {
    markov: Markov,
    state: String,
}

impl Iterator for SentenceGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.ends_with('.') {
            return None;
        }
        self.state = self.markov.random_word(&self.state);
        Some(self.state.clone())
    }
}

fn main() {
    let chain = Markov::new("examples/eng_news_2005_1M-sentences.gz");
    for _ in 0..10 {
        println!("{:?}", chain.sentence_generator().collect::<Vec<_>>().join(" "));
    }
}
