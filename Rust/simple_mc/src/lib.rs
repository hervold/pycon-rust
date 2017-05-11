extern crate rand;

use std::collections::HashMap;
use rand::{Rng, SeedableRng, StdRng};
use std::io::{BufRead, BufReader};
use std::fs::File;

// for C/Python compatibility
use std::ffi::{CStr, CString};
use std::os::raw::{ c_char, c_void };

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SentAtom {
    Comma,
    SentBreak,
    Word(String)
}

impl SentAtom {
    pub fn is_not_word(&self) -> bool {
        match self {
            &SentAtom::Word(_) => false,
            _ => true
        }
    }
}

pub type WordTally = HashMap<SentAtom, usize>;
pub type WordFreq = HashMap<SentAtom, WordTally>;

// wrap the Rng and HashMap in one package to pass to Python
pub struct LibState<R: Rng + Sized>{
    rng: R,
    counts: WordFreq
}


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

    let mut next = weighted_choice(&mut rnd, corpus.get( first ).unwrap());

    while next != &SentAtom::SentBreak {
        let mut seed = next.clone();
        // we don't treat Comma's like regular words
        let word = match next {
            &SentAtom::Comma => {

                // add comma to most recent word
                let last = accum.pop().unwrap();
                let mut last_plus_comma = last.clone();
                last_plus_comma.push(',');

                // now grab an alternative next word
                seed = weighted_choice(&mut rnd, corpus.get(&SentAtom::Word(last.clone())).unwrap());
                while seed.is_not_word() {
                    seed = weighted_choice(&mut rnd, corpus.get(&SentAtom::Word(last.clone())).unwrap())
                }
                last_plus_comma
            },
            &SentAtom::Word(ref w) => w.to_string(),
            _ => panic!(),
        };
        accum.push( word );
        next = weighted_choice(&mut rnd, corpus.get(seed).unwrap());
    }
    accum.join(" ")
}

#[no_mangle]
pub extern fn read_corpus_file( _fname: *const c_char ) -> *const c_void {
    let fname =
        unsafe {
            CStr::from_ptr(_fname).to_string_lossy().into_owned()
        };

    let mut reader = BufReader::new( File::open(fname).unwrap() );

    let counts = read_corpus(&mut reader);
    let rng = StdRng::from_seed(&[1,2,3,4]);
    let state = Box::new( LibState { rng, counts } );
    Box::into_raw(state) as *mut c_void
}

#[no_mangle]
pub extern fn ext_generate_sentence( _state: *const c_void ) -> *const c_char {

    let mut state = unsafe {
        Box::from_raw(_state as *mut LibState<StdRng>)
    };
    // ugly - https://github.com/rust-lang/rust/issues/30564
    let tmp = *state;
    let (mut rng, counts) = match tmp {
        LibState { rng, counts } => (rng, counts)
    };
    CString::new( generate_sentence(&mut rng, &counts) ).unwrap().into_raw()
}

#[no_mangle]
pub extern fn release_str( somestr: *mut c_char ) {
    unsafe { CString::from_raw(somestr); }
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
