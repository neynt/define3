pub mod parse_xml;
pub mod parse_wikitext;

#[derive(Debug)]
pub struct Meaning {
    pub language: String,
    pub part_of_speech: String,
    pub definition: String,
}

#[derive(Debug)]
pub struct Page {
    pub title: String,
    pub content: String,
}

#[derive(Debug)]
pub struct Word {
    pub name: String,
    pub meanings: Vec<Meaning>,
}

#[derive(Debug)]
pub struct Template {
    pub name: String,
    pub content: String,
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub src: String,
}

#[derive(Debug)]
pub enum PageContent {
    Word(Word),
    Template(Template),
    Module(Module),
}
