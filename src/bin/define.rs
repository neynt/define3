extern crate colored;
extern crate define3;
extern crate getopts;
extern crate regex;
extern crate rusqlite;
extern crate textwrap;
extern crate nom;

use define3::Meaning;

use colored::*;
use regex::{Captures, Regex};
use rusqlite::Connection;
use std::collections::BTreeMap;
use std::env;
use std::path::Path;

fn get_word_defs(
    conn: &Connection,
    word: &str,
) -> Box<BTreeMap<String, BTreeMap<String, Vec<String>>>> {
    let mut stmt = conn.prepare(
        "SELECT language, part_of_speech, definition FROM words WHERE name = ?1",
    ).unwrap();
    let word_iter = stmt.query_map(&[&word], |row| Meaning {
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
    Box::new(langs)
}

// TODO: Actually expand templates. This is very hard because Wikitext templates have a bunch of
// functions and often call out into Lua code.
// https://www.mediawiki.org/wiki/Help:Extension:ParserFunctions
// https://www.mediawiki.org/wiki/Extension:Scribunto
fn _expand_template(conn: &Connection, args: &[&str]) -> String {
    fn get_template_content(conn: &Connection, name: &str) -> String {
        let result = conn.query_row(
            "SELECT content FROM templates WHERE name = ?1",
            &[&name],
            |row| row.get(0),
        );
        result.unwrap()
    }
    get_template_content(conn, args[0])
}

// For now, we just special-case a couple common templates.
// (or maybe not even that; turns out nested templates are a thing)
fn replace_template(_conn: &Connection, caps: &Captures) -> String {
    let s = caps.get(1).unwrap().as_str();
    let elems: Vec<&str> = s.split('|').collect();
    //expand_template(conn, &elems)
    match elems[0] {
        "," => ",".to_owned(),
        "ngd" | "unsupported" | "non-gloss definition" => elems[1].to_owned(),
        "alternative form of" => format!("Alternative form of {}", elems[1]),
        "ja-romanization of" => format!("Rōmaji transcription of {}", elems[1]),
        "sumti" => format!("x{}", elems[1]),
        "ja-def" => format!("{}:", elems[1]),
        "lb" => format!("({})", elems[2]),
        "m" | "l" => elems[2].to_owned(),
        _ => format!("{{{{{}}}}}", elems.join("|")),
    }
}

fn print_words<F>(langs: &BTreeMap<String, BTreeMap<String, Vec<String>>>, mut format: F)
where
    F: FnMut(&str) -> String,
{
    let wrapper = textwrap::Wrapper::new(80)
        .initial_indent("    ")
        .subsequent_indent("      ");

    for (lang, poses) in langs {
        println!("{}", lang.green().bold());
        for (pos, defns) in poses {
            println!("  {}", pos.white());
            for defn in defns {
                let defn = format(defn);
                let defn = wrapper.fill(&defn);
                println!("{}", defn);
            }
        }
    }

    if langs.len() == 0usize {
        println!("No results found.");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // TODO: this shit don't support nested templates, which are unfortunately a thing. we might
    // need a more legit parsing solution
    let re_template = Regex::new(r"\{\{(?P<text>(?s:[^\{])*?)\}\}").unwrap();

    if args.len() < 2 {
        println!("Usage: {} WORD/WORDS", &args[0]);
        return;
    }

    let conn = Connection::open(Path::new("/trove/data/enwikt-20180301.sqlite3")).unwrap();
    let langs = *get_word_defs(&conn, &args[1]);
    print_words(&langs, |s| {
        let replace_template_ = |caps: &Captures| -> String { replace_template(&conn, caps) };
        let mut result = s.to_owned();
        loop {
            let result_ = re_template.replace_all(&result, &replace_template_).to_string();
            if result == result_ {
                break
            }
            result = result_;
        }
        result
    });
}
