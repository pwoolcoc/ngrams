//! This is a just awful, awful implementation of a markov chain generator, but
//! it serves the purpose, which is to use ngrams to generate subsequences of words
//! in a corpus

extern crate ngrams;
extern crate rand;
extern crate flate2;
extern crate curl;

use ngrams::Ngrams;
use std::collections::HashMap;
use std::io::{Cursor, BufRead, BufReader};
use flate2::read::GzDecoder;
use curl::easy::Easy;

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
    fn new(url: &'static str) -> Markov {
        let mut map: HashMap<String, HashMap<String, u32>> = HashMap::new();
        let mut handle = Easy::new();
        let mut data = Vec::new();
        handle.url(url).unwrap();
        {
            let mut transfer = handle.transfer();
            transfer.write_function(|new_data| {
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
            }).unwrap();
            transfer.perform().unwrap();
        }
        let file = BufReader::new(
                        GzDecoder::new(Cursor::new(data)
                            ).unwrap());
        println!("extracting sentences...");
        let sentences = file.lines().map(|a| a.unwrap()).map(extract_sentence).map(tokenize_sentence);
        print!("Building map of ngrams...");
        for tokenized in sentences {
            let grams = Ngrams::new(tokenized.into_iter(), 2).pad();
            for gram in grams {
                let first = gram[0].clone();
                let second = gram[1].clone();
                let entry = map.entry(first).or_insert(HashMap::new());
                let entry = entry.entry(second).or_insert(0);
                *entry += 1;
            }
        }
        println!("Done!");
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
    let url = "https://gitlab.com/pwoolcoc/ngrams/raw/master/examples/eng_news_2005_1M-sentences.gz";
    println!("Generating markov chain from input data\n\n\t{}\n\nThis is gonna take a while...", url);
    let chain = Markov::new(url);
    for _ in 0..10 {
        println!("{:?}", chain.sentence_generator().collect::<Vec<_>>().join(" "));
    }
}
