#![deny(warnings)]
extern crate quick_xml;

use quick_xml::reader::Reader;
use quick_xml::events::Event;
use std::path::Path;
use std::io::BufRead;

fn parse_revision<B: BufRead>(reader: &mut Reader<B>) -> Box<String> {
  let mut buf = Vec::new();
  let mut text = None;
  loop {
    match reader.read_event(&mut buf) {
      Ok(Event::Start(ref e)) if e.name() == b"text" => {
        let mut buf = Vec::new();
        match reader.read_event(&mut buf) {
          Ok(Event::Text(e)) => text = Some(e.unescape_and_decode(&reader).unwrap().clone()),
          _ => (),
        }
      }
      Ok(Event::End(ref e)) if e.name() == b"revision" => break,
      _ => (),
    }
  }
  Box::new(text.unwrap())
}

#[derive(Debug)]
struct Page {
  title: String,
  text: String,
}

fn parse_page<B: BufRead>(mut reader: &mut Reader<B>) -> Box<Page> {
  let mut buf = Vec::new();
  let mut title = None;
  let mut text = None;
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
            text = Some(parse_revision(&mut reader));
          },
          _ => (),
        }
      }
      Ok(Event::End(ref e)) if e.name() == b"page" => break,
      _ => (),
    }
  }
  Box::new(Page {
    title: title.unwrap(),
    text: *text.unwrap(),
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
            let page = parse_page(&mut reader);
            println!("{:?}", page.title);
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
