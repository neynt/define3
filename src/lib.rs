pub mod parse_xml;
pub mod parse_wikitext;

#[derive(Debug, Clone)]
pub struct Meaning {
    pub language: String,
    pub part_of_speech: String,
    pub definition: String,
}

#[derive(Debug)]
pub struct Word {
    pub name: String,
    pub meanings: Vec<Meaning>,
}
