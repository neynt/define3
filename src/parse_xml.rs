extern crate quick_xml;

use parse_xml::quick_xml::reader::Reader;
use parse_xml::quick_xml::events::Event;

use std::path::Path;
use std::collections::HashSet;
use std::io::BufRead;

use parse_wikitext::parse_wikitext;

use Word;
use Meaning;

fn parse_revision<B: BufRead>(
    reader: &mut Reader<B>,
    languages: &HashSet<&str>,
    parts_of_speech: &HashSet<&str>,
) -> Option<Vec<Meaning>> {
    let mut buf = Vec::new();
    let mut result = None;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"text" => {
                let mut buf = Vec::new();
                match reader.read_event(&mut buf) {
                    Ok(Event::Text(e)) => {
                        let text = e.unescape_and_decode(&reader).unwrap();
                        result = Some(parse_wikitext(text, languages, parts_of_speech));
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

pub fn parse_page<B: BufRead>(
    mut reader: &mut Reader<B>,
    languages: &HashSet<&str>,
    parts_of_speech: &HashSet<&str>,
) -> Option<Box<Word>> {
    let mut buf = Vec::new();
    let mut title = None;
    let mut meanings = None;
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
                        meanings = parse_revision(&mut reader, languages, parts_of_speech);
                    }
                    _ => (),
                }
            }
            Ok(Event::End(ref e)) if e.name() == b"page" => break,
            _ => (),
        }
    }
    // and_then is a poor name for >>=
    title.and_then(|title| {
        meanings.map(|meanings| {
            Box::new(Word {
                name: title,
                meanings: meanings,
            })
        })
    })
}

pub fn extract_words<F>(languages: &HashSet<&str>, parts_of_speech: &HashSet<&str>, mut f: F)
where
    F: FnMut(Box<Word>) -> (),
{
    let mut buf = Vec::new();
    let mut reader = Reader::from_file(Path::new(
        "/trove/data/enwiktionary-20180301-pages-articles.xml",
    )).unwrap();
    'read_words: loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"page" => {
                    let word = parse_page(&mut reader, &languages, &parts_of_speech);
                    word.map(|word| f(word));
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
