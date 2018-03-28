#![deny(warnings)]
extern crate quick_xml;

use quick_xml::reader::Reader;
use quick_xml::events::Event;
use std::path::Path;
use std::io::BufRead;
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
enum WikiContext {
  Heading1(String),
  Heading2(String),
  Heading3(String),
  Heading4(String),
  Heading5(String),
  Heading6(String),
}

use WikiContext::*;

impl WikiContext {
  fn precedence(&self) -> u32 {
    match self {
      &Heading1(_) => 1,
      &Heading2(_) => 2,
      &Heading3(_) => 3,
      &Heading4(_) => 4,
      &Heading5(_) => 5,
      &Heading6(_) => 6,
    }
  }

  fn text(&self) -> &String {
    match self {
      &Heading1(ref x) => &x,
      &Heading2(ref x) => &x,
      &Heading3(ref x) => &x,
      &Heading4(ref x) => &x,
      &Heading5(ref x) => &x,
      &Heading6(ref x) => &x,
    }
  }
}

#[derive(Debug, Clone)]
struct Meaning {
  language: String,
  part_of_speech: String,
}

#[derive(Debug)]
struct Word {
  name: String,
  meanings: Vec<Meaning>,
}

fn parse_wikitext(text: String) -> Vec<Meaning> {
  let mut result: Vec<Meaning> = Vec::new();
  let mut contexts: Vec<WikiContext> = Vec::new();

  let languages: HashSet<&str> =
    ["English", "Japanese"].iter().cloned().collect();

  let parts_of_speech: HashSet<&str> =
    ["Noun", "Verb", "Adjective", "Romanization", "Interjection", "Proper noun"]
    .iter().cloned().collect();

  fn apply_context(contexts: &mut Vec<WikiContext>, context: WikiContext) {
    let new_prec = context.precedence();
    // leave only lower-precedence contexts in the stack
    while contexts.last().map_or(false, |c| c.precedence() >= new_prec) {
      contexts.pop();
    }
    contexts.push(context);
  };

  for line in text.lines() {
    if line.starts_with("======") {
      apply_context(&mut contexts, Heading6(String::from(&line[6..line.len()-6])));
    } else if line.starts_with("=====") {
      apply_context(&mut contexts, Heading5(String::from(&line[5..line.len()-5])));
    } else if line.starts_with("====") {
      apply_context(&mut contexts, Heading4(String::from(&line[4..line.len()-4])));
    } else if line.starts_with("===") {
      apply_context(&mut contexts, Heading3(String::from(&line[3..line.len()-3])));
    } else if line.starts_with("==") {
      apply_context(&mut contexts, Heading2(String::from(&line[2..line.len()-2])));
    } else if line.starts_with("=") {
      apply_context(&mut contexts, Heading1(String::from(&line[1..line.len()-1])));
    } else if line.starts_with("# ") {
      if contexts.len() >= 2
      && languages.contains(&contexts[contexts.len() - 2].text().as_str())
      && parts_of_speech.contains(&contexts[contexts.len() - 1].text().as_str()) {
        let language = contexts[contexts.len() - 2].text();
        let part_of_speech = contexts[contexts.len() - 1].text();
        result.push(Meaning {
          language: language.clone(),
          part_of_speech: part_of_speech.clone(),
        });
      }
    }
  }
  result
}

fn parse_revision<B: BufRead>(reader: &mut Reader<B>) -> Vec<Meaning> {
  let mut buf = Vec::new();
  let mut result = None;
  loop {
    match reader.read_event(&mut buf) {
      Ok(Event::Start(ref e)) if e.name() == b"text" => {
        let mut buf = Vec::new();
        match reader.read_event(&mut buf) {
          Ok(Event::Text(e)) => {
            let text = e.unescape_and_decode(&reader).unwrap();
            result = Some(parse_wikitext(text));
          },
          _ => (),
        }
      }
      Ok(Event::End(ref e)) if e.name() == b"revision" => break,
      _ => (),
    }
  }
  result.unwrap()
}

fn parse_page<B: BufRead>(mut reader: &mut Reader<B>) -> Box<Word> {
  let mut buf = Vec::new();
  let mut title = None;
  let mut meanings = None;
  loop {
    match reader.read_event(&mut buf) {
      Ok(Event::Start(ref e)) => {
        let mut buf = Vec::new();
        match e.name() {
          b"title" => {
            match reader.read_event(&mut buf) {
              Ok(Event::Text(e)) => title = Some(e.unescape_and_decode(&reader).unwrap().clone()),
              _ => (),
            }
          },
          b"revision" => {
            meanings = Some(parse_revision(&mut reader));
          },
          _ => (),
        }
      }
      Ok(Event::End(ref e)) if e.name() == b"page" => break,
      _ => (),
    }
  }
  Box::new(Word {
    name: title.unwrap(),
    meanings: meanings.unwrap(),
  })
}

fn main() {
  let mut reader =
    Reader::from_file(Path::new("/trove/data/enwiktionary-20180301-pages-articles.xml"))
    .unwrap();
  let mut buf = Vec::new();
  loop {
    match reader.read_event(&mut buf) {
      Ok(Event::Start(ref e)) => {
        match e.name() {
          b"page" => {
            let word = parse_page(&mut reader);
            println!("{:?}", word.name);
            for meaning in word.meanings {
              println!("{:?}", meaning);
            }
          }
          _ => (),
        }
      }
      Ok(Event::Eof) => break,
      Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
      _ => (),
    }
    buf.clear();
  }
}
