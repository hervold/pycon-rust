extern crate rand;

use std::collections::HashMap;
use rand::{Rng, SeedableRng, StdRng};
use std::io::{BufRead, BufReader};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SentAtom {
    Comma,
    SentBreak,
    Word(String)
}

pub type WordTally = HashMap<SentAtom, usize>;
pub type WordFreq = HashMap<SentAtom, WordTally>;


pub fn weighted_choice<'a, 'b, R>( mut rnd: &'b mut R, counts: &'a WordTally )
                                   -> &'a SentAtom
    where R: Rng + Sized {

    let mut acc = Vec::new();
    for (word, ct) in counts {
        for _ in 0 .. *ct {
            acc.push( word );
        }
    }
    rnd.choose(&acc).unwrap()
}

pub fn trimcomma<'a>( word: &'a str ) -> &'a str {
    if word.ends_with(",") {
        &word[ .. word.len() - 1 ]
    } else {
        word
    }
}

pub fn read_corpus( reader: &mut BufRead ) -> WordFreq {
    let mut counts: WordFreq = HashMap::new();
    for input in reader.lines() {
        if let Ok(line) = input {
            let words = line.split_whitespace().collect::<Vec<&str>>();
            for i in 0 .. words.len() - 1 {

                if words[i].ends_with(",") {
                    let mut word_entry = counts
                        .entry(SentAtom::Word(trimcomma(words[i]).to_string()))
                        .or_insert_with(HashMap::new);

                    // tally comma
                    {
                        let mut assoc_e = word_entry
                            .entry(SentAtom::Comma)
                            .or_insert(0);
                        *assoc_e += 1;
                    }

                    // tally next word
                    {
                        let mut assoc_e = word_entry
                            .entry(SentAtom::Word(trimcomma(words[i+1]).to_string()))
                            .or_insert(0);
                        *assoc_e += 1;
                    }

                } else {
                    let mut word_entry = counts
                        .entry(SentAtom::Word(words[i].to_string()))
                        .or_insert_with(HashMap::new);

                    // tally next word
                    let mut assoc_e = word_entry
                        .entry(SentAtom::Word(trimcomma(words[i+1]).to_string()))
                        .or_insert(0);
                    *assoc_e += 1;
                }
            }
            // tally the end-of-sentence
            let mut word_entry = counts
                .entry(SentAtom::Word(words[words.len()-1].to_string()))
                .or_insert_with(HashMap::new);
            let mut assoc_e = word_entry
                .entry(SentAtom::SentBreak)
                .or_insert(0);
            *assoc_e += 1;
        }
    }
    counts
}


pub fn generate_sentence<'a, R>( mut rnd: &'a mut R, corpus: &WordFreq ) -> String
    where R: Rng + Sized {

    let keys = corpus.keys().collect::<Vec<&SentAtom>>();

    let first = rnd.choose(&keys).unwrap();
    let mut accum: Vec<String> = vec![ match first {
        &&SentAtom::Word(ref w) => w.to_string(),
        _ => panic!()
    } ];

    let mut next = weighted_choice(&mut rnd, corpus.get( first ).unwrap() );
    while next != &SentAtom::SentBreak {
        let word = match next {
            &SentAtom::Comma => {
                let mut last = accum.pop().unwrap();
                last.push(',');
                last
            },
            &SentAtom::Word(ref w) => w.to_string(),
            _ => panic!(),
        };
        accum.push( word );
        next = weighted_choice(&mut rnd, corpus.get(next).unwrap() );
    }
    accum.join(" ")
}


#[cfg(test)]
mod tests {
    use super::*;

    const TINY_SENT: &'static [u8] = b"hello world";
    const SENTENCES: &'static [u8] = b"this is a sentence
this is another sentence
not this, though";

    #[test]
    fn test_trimcomma() {
        assert_eq!( trimcomma("abc,"), "abc");
    }

    #[test]
    #[ignore]
    fn simple_read_corpus() {
        let mut rdr1 = BufReader::new(TINY_SENT);
        let tiny_cts = read_corpus( &mut rdr1 );

        let mut world1 = HashMap::new();
        world1.insert( SentAtom::Word("world".to_string()), 1 );

        let mut world2 = HashMap::new();
        world2.insert( SentAtom::SentBreak, 1 );

        let mut ref_map = HashMap::new();
        ref_map.insert( SentAtom::Word("hello".to_string()), world1 );
        ref_map.insert( SentAtom::Word("world".to_string()), world2 );

        assert_eq!( tiny_cts, ref_map );
    }

    #[test]
    fn test_corpus_counts() {
        let mut reader = BufReader::new(SENTENCES);
        let counts = read_corpus( &mut reader );

        assert_eq!( counts
                    .get( &SentAtom::Word("this".to_string()) )
                    .unwrap()
                    .get( &SentAtom::Word("is".to_string()) )
                    .unwrap(),
                    &2 );
    }

    #[test]
    fn test_generate_sentence() {
        let mut reader = BufReader::new(SENTENCES);
        let counts = read_corpus( &mut reader );
        let mut rng = StdRng::from_seed(&[1,2,3,4]);
        for _ in 0 .. 5 {
            println!("{}.", generate_sentence(&mut rng, &counts));
        }
    }
}
