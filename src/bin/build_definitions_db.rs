extern crate define3;
extern crate rusqlite;

use std::path::Path;
use std::collections::HashSet;

use rusqlite::{Connection, Transaction};

fn main() {
    // TODO: figure out list of languages automatically
    let languages: HashSet<&str> = [
        "English", "Japanese", "Chinese", "Korean", "Lojban", "German", "French"
    ].iter()
        .cloned()
        .collect();

    // TODO: figure out POS list automatically
    let parts_of_speech: HashSet<&str> = [
        "Noun",
        "Verb",
        "Adjective",
        "Adverb",
        "Definitions",
        "Infix",
        "Conjunction",
        "Romanization",
        "Interjection",
        "Proper noun",
        "Hanzi",
        "Kanji",
        "Hanja",
        "Brivla",
        "Gismu",
        "Rafsi",
        "Cmavo",
    ].iter()
        .cloned()
        .collect();

    let mut conn = Connection::open(Path::new("/trove/data/enwikt-20180301.sqlite3")).unwrap();
    let tx = Transaction::new(&mut conn, rusqlite::TransactionBehavior::Exclusive).unwrap();

    tx.execute("DROP TABLE IF EXISTS words", &[]).unwrap();
    tx.execute(
        "CREATE TABLE words (
             name           text not null,
             language       text not null,
             part_of_speech text not null,
             definition     text not null
         )",
        &[],
    ).unwrap();

    let mut count: u64 = 0;

    define3::parse_xml::extract_words(&languages, &parts_of_speech, |word| {
        count += 1;
        if count % 100000 == 0 {
            println!("{}", count);
            println!("{}", word.name);
            for meaning in &word.meanings {
                println!(
                    "  {} {}: {}",
                    meaning.language, meaning.part_of_speech, meaning.definition
                );
            }
        }
        for meaning in &word.meanings {
            tx.execute(
                "insert into words (name, language, part_of_speech, definition)
                 values (?1, ?2, ?3, ?4)",
                &[
                    &word.name,
                    &meaning.language,
                    &meaning.part_of_speech,
                    &meaning.definition,
                ],
            ).unwrap();
        }
    });

    tx.execute_batch(
        "create index words_name_idx on words(name);
         create index words_language_idx on words(language);
         create index words_part_of_speech_idx on words(part_of_speech);",
    ).unwrap();

    tx.commit().unwrap();
}
