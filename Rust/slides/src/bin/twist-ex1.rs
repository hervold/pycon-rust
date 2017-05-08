#[derive(Debug)]
enum Sentence {
    Period,
    Word { text: String, count: usize }
}

fn main() {
    let sentence = vec![ Sentence::Word { text: "Simple".to_string(), count: 1 },
                         Sentence::Word { text: "Rust".to_string(), count: 2 },
                         Sentence::Word { text: "demo".to_string(), count: 1 },
                         Sentence::Period ];
    println!("{:?}", &sentence);
}
