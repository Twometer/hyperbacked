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

#[cfg(test)]
mod test {
    use super::gen_passphrase;

    #[test]
    fn test_word_count() {
        let passphrase = gen_passphrase(4);
        assert_eq!(4, passphrase.split(" ").count());
    }

    #[test]
    fn test_different_results() {
        let passphrase1 = gen_passphrase(4);
        let passphrase2 = gen_passphrase(4);
        assert_ne!(passphrase1, passphrase2);
    }
}
