extern crate rand;

use std::collections::HashMap;
use rand::{Rng, SeedableRng, StdRng};
use std::io::{BufRead, BufReader};


pub type WordTally = HashMap<String, usize>;
pub type WordFreq = HashMap<String, WordTally>;


pub enum SentAtom {
    Comma,
    SentBreak,
    Word(String)
}

pub fn weighted_choice<'a, 'b, R>( mut rnd: &'b mut R, counts: &'a WordTally ) -> &'a str
    where R: Rng + Sized {

    let mut acc = Vec::new();
    for (word, ct) in counts {
        for _ in 0 .. *ct {
            acc.push( word );
        }
    }
    rnd.choose(&acc).unwrap()
}

pub fn read_corpus( reader: &mut BufRead ) -> WordFreq {
    let mut buffer = String::new();
    let mut counts: WordFreq = HashMap::new();
    while reader.read_line(&mut buffer).unwrap() > 0 {
        let words = buffer.split_whitespace().collect::<Vec<&str>>();
        for i in (0 .. words.len() - 1) {
            let mut word_entry = counts.entry(words[i].to_string()).or_insert_with(HashMap::new);
            let mut assoc_e = word_entry.entry(words[i+1].to_string()).or_insert(0);
            *assoc_e += 1;
        }
    }
    counts
}

/*
pub fn generate_sentence( corpus: &WordFreq ) -> String {

}
 */

pub fn foo() {
    let mut rng = StdRng::from_seed(&[1,2,3,4]);
    
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTENCES: &'static [u8] = b"this is a sentence.
this is another sentence.";
    
    #[test]
    fn test_read_corpus() {
        let mut reader = BufReader::new(SENTENCES);
        let counts = read_corpus( &mut reader );
        println!("{:?}", &counts);
    }
}
