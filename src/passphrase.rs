use lazy_static::lazy_static;
use rand::prelude::*;

const WORD_LIST: &'static str = include_str!("../res/wordlist_eff.txt");

lazy_static! {
    static ref WORDS: Vec<&'static str> = WORD_LIST.split("\n").collect();
}

pub fn gen_passphrase(num_words: usize) -> String {
    let mut rng = thread_rng();
    return WORDS
        .choose_multiple(&mut rng, num_words)
        .map(|&word| word)
        .collect::<Vec<&str>>()
        .join(&" ");
}
