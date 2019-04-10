extern crate quick_xml;

use parse_xml::quick_xml::reader::Reader;
use parse_xml::quick_xml::events::Event;

use std::path::Path;
use std::io::BufRead;

use Page;

fn parse_revision<B: BufRead>(reader: &mut Reader<B>) -> Option<String> {
    let mut buf = Vec::new();
    let mut result = None;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"text" => {
                let mut buf = Vec::new();
                match reader.read_event(&mut buf) {
                    Ok(Event::Text(e)) => {
                        let text = e.unescape_and_decode(&reader).unwrap();
                        result = Some(text);
                    }
                    _ => (),
                }
            }
            Ok(Event::End(ref e)) if e.name() == b"revision" => break,
            _ => (),
        }
    }
    result
}

pub fn parse_page<B: BufRead>(mut reader: &mut Reader<B>) -> Option<Page> {
    let mut buf = Vec::new();
    let mut title = None;
    let mut content = None;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let mut buf = Vec::new();
                match e.name() {
                    b"title" => match reader.read_event(&mut buf) {
                        Ok(Event::Text(e)) => {
                            title = Some(e.unescape_and_decode(&reader).unwrap().clone())
                        }
                        _ => (),
                    },
                    b"revision" => {
                        content = parse_revision(&mut reader);
                    }
                    _ => (),
                }
            }
            Ok(Event::End(ref e)) if e.name() == b"page" => break,
            _ => (),
        }
    }
    // and_then is a poor name for >>=
    title.and_then(|title| content.map(|content| Page { title, content }))
}

pub fn for_pages<F>(filename: &str, mut f: F)
where
    F: FnMut(Page) -> (),
{
    let mut buf = Vec::new();
    // TODO: I really shouldn't hardcode paths like this
    let mut reader = Reader::from_file(Path::new(filename)).unwrap();
    'read_words: loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"page" => {
                    let page = parse_page(&mut reader);
                    page.map(|page| f(page));
                }
                _ => (),
            },
            Ok(Event::Eof) => break 'read_words,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
        buf.clear();
    }
}
