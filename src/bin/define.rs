extern crate colored;
extern crate define3;
extern crate rusqlite;
extern crate textwrap;
extern crate regex;

use define3::Meaning;

use colored::*;
use regex::Regex;
use rusqlite::Connection;
use std::collections::BTreeMap;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} WORD/WORDS", &args[0]);
        return;
    }

    let conn = Connection::open(Path::new("/trove/data/enwikt-20180301.sqlite3")).unwrap();
    let mut stmt = conn.prepare(
        "SELECT language, part_of_speech, definition FROM words WHERE name = ?1",
    ).unwrap();
    let word_iter = stmt.query_map(&[&args[1]], |row| Meaning {
        language: row.get(0),
        part_of_speech: row.get(1),
        definition: row.get(2),
    }).unwrap();

    let mut langs: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

    for meaning in word_iter {
        let meaning = meaning.unwrap();
        langs
            .entry(meaning.language)
            .or_insert(BTreeMap::new())
            .entry(meaning.part_of_speech)
            .or_insert(Vec::new())
            .push(meaning.definition);
    }

    // TODO: This should be done when building the DB.
    // TODO: combine link REs into one
    let re_display_link = Regex::new(r"\[\[[^\]]*?\|(?P<text>.*?)\]\]").unwrap();
    let re_link = Regex::new(r"\[\[(?P<text>.*?)\]\]").unwrap();
    let re_html_comment = Regex::new(r"<!--.*?-->").unwrap();

    // This technically doesn't work if some jerk decided to format a single quote.
    let re_bold = Regex::new(r"'''(?P<text>[^']*?)'''").unwrap();
    let re_italic = Regex::new(r"''(?P<text>[^']*?)''").unwrap();

    let wrapper = textwrap::Wrapper::new(80)
        .initial_indent("    ")
        .subsequent_indent("      ");

    for (lang, poses) in &langs {
        println!("{}", lang.green().bold());
        for (pos, defns) in poses {
            println!("  {}", pos.white());
            for defn in defns {
                //let defn = re_link.replace_all(&defn, "\x1b[0;36m$x\x1b[0m");
                let defn = re_display_link.replace_all(&defn, "$text");
                let defn = re_link.replace_all(&defn, "$text");
                let defn = re_html_comment.replace_all(&defn, "");
                let defn = re_bold.replace_all(&defn, "$text");
                let defn = re_italic.replace_all(&defn, "$text");
                let defn = wrapper.fill(&defn);
                println!("{}", defn);
            }
        }
    }

    if langs.len() == 0usize {
        println!("No results found.");
    }
}
