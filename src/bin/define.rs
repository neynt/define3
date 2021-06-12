extern crate colored;
extern crate define3;
extern crate getopts;
extern crate regex;
extern crate rusqlite;
extern crate textwrap;
extern crate nom;

use define3::Meaning;

use colored::*;
use getopts::Options;
use regex::{Captures, Regex};
use rusqlite::Connection;
use std::collections::BTreeMap;
use std::env;
use std::path::Path;

fn get_defns_by_lang(
    conn: &Connection,
    word: &str,
) -> Box<BTreeMap<String, BTreeMap<String, Vec<String>>>> {
    let mut stmt = conn.prepare(
        "SELECT language, part_of_speech, definition FROM words WHERE name = ?1",
    ).unwrap();
    let word_iter = stmt.query_map(&[&word], |row| Ok (Meaning {
        language: row.get(0).unwrap(),
        part_of_speech: row.get(1).unwrap(),
        definition: row.get(2).unwrap(),
    })).unwrap();

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
fn expand_template(conn: &Connection, args: &[&str]) -> String {
    fn get_template_content(conn: &Connection, name: &str) -> String {
        let result = conn.query_row(
            "SELECT content FROM templates WHERE name = ?1",
            &[&name],
            |row| row.get(0),
        );
        println!("{}", name);
        result.unwrap()
    }
    get_template_content(conn, args[0])
}

// For now, we just hardcode a couple common templates.
fn replace_template(_conn: &Connection, caps: &Captures) -> String {
    let s = caps.get(1).unwrap().as_str();
    let elems: Vec<&str> = s.split('|').collect();
    //match elems[0] {
    //    _ => expand_template(conn, &elems)
    //}
    match elems[0] {
        "," =>
            ",".to_owned(),
        "ngd" | "unsupported" | "non-gloss definition" =>
            elems[1].to_owned(),
        "alternative form of" =>
            format!("Alternative form of {}", elems[1]),
        "ja-romanization of" =>
            format!("Rōmaji transcription of {}", elems[1]),
        "sumti" =>
            format!("x{}", elems[1]),
        "ja-def" =>
            format!("{}:", elems[1]),
        "qualifier" =>
            format!("({})", elems[1]),
        "lb" =>
            format!("({})", elems[2]),
        "m" | "l" =>
            elems[2].to_owned(),
        _ => caps.get(0).unwrap().as_str().to_owned(),
    }
}

fn print_words<F>(langs: &BTreeMap<String, BTreeMap<String, Vec<String>>>, mut format: F)
where
    F: FnMut(&str) -> String,
{
    let textwrap_opts = textwrap::Options::new(80)
        .initial_indent("    ")
        .subsequent_indent("      ");

    for (lang, poses) in langs {
        println!("{}", lang.green().bold());
        for (pos, defns) in poses {
            println!("  {}", pos.white());
            for defn in defns {
                let defn = format(defn);
                let defn = textwrap::fill(&defn, &textwrap_opts);
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
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help text");
    opts.optflag("r", "raw", "don't expand wiki templates");
    opts.optopt("l", "language", "only print this language", "lang");
    let matches = opts.parse(&args[1..]).unwrap();
    if matches.opt_present("h") || matches.free.len() != 1 {
        let brief = format!("Usage: {} [options] WORD", args[0]);
        print!("{}", opts.usage(&brief));
        return;
    }

    // TODO: We currently support nested templates in a very bad way. We expand templates in
    // layers, most deeply nested first, and we do this by excluding curly braces in the regex.
    // Should eventually use a more legit parser (nom maybe?)
    let re_template = Regex::new(r"\{\{(?P<text>(?s:[^\{])*?)\}\}").unwrap();

    let mut sqlite_path = dirs::data_dir().unwrap();
    sqlite_path.push("define3");
    sqlite_path.push("define3.sqlite3");
    let conn = Connection::open(Path::new(&sqlite_path)).unwrap();

    let all_langs = *get_defns_by_lang(&conn, &matches.free[0]);
    let langs =
        match matches.opt_str("l") {
            None => all_langs,
            Some(lang) => {
                let mut result = BTreeMap::new();
                for &result_for_lang in all_langs.get(&lang).iter() {
                    result.insert(lang.clone(), result_for_lang.clone());
                }
                result
            },
        }
    ;
    print_words(&langs, |s| {
        let replace_template = |caps: &Captures| -> String { replace_template(&conn, caps) };
        let mut result = s.to_owned();
        if !matches.opt_present("r") {
            loop {
                let result_ = re_template.replace_all(&result, &replace_template).to_string();
                //println!("{}", result_);
                if result == result_ {
                    break
                }
                result = result_;
            }
        }
        result
    });
}
