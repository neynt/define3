use std::collections::HashSet;

use Meaning;

#[derive(Debug, PartialEq)]
pub enum WikiContext {
    Heading1(String),
    Heading2(String),
    Heading3(String),
    Heading4(String),
    Heading5(String),
    Heading6(String),
}

use parse_wikitext::WikiContext::*;

impl WikiContext {
    pub fn precedence(&self) -> u32 {
        match self {
            &Heading1(_) => 1,
            &Heading2(_) => 2,
            &Heading3(_) => 3,
            &Heading4(_) => 4,
            &Heading5(_) => 5,
            &Heading6(_) => 6,
        }
    }

    pub fn text(&self) -> &String {
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

pub struct ContextStack {
    contexts: Vec<WikiContext>,
    pub language: Option<String>,
    pub part_of_speech: Option<String>,
}

impl ContextStack {
    pub fn apply(
        &mut self,
        context: WikiContext,
        languages: &HashSet<&str>,
        parts_of_speech: &HashSet<&str>,
    ) {
        let new_prec = context.precedence();
        // leave only lower-precedence contexts in the stack
        let contexts = &mut self.contexts;
        while contexts
            .last()
            .map_or(false, |c| c.precedence() >= new_prec)
        {
            match contexts.pop() {
                None => (),
                Some(context) => {
                    if self.language.as_ref().unwrap_or(&String::from("")) == context.text() {
                        self.language = None;
                    }
                    if self.part_of_speech.as_ref().unwrap_or(&String::from("")) == context.text() {
                        self.part_of_speech = None;
                    }
                }
            }
        }
        if languages.contains(context.text().as_str()) {
            self.language = Some(context.text().clone());
        }
        if parts_of_speech.contains(context.text().as_str()) {
            self.part_of_speech = Some(context.text().clone());
        }
        contexts.push(context);
    }

    pub fn new() -> ContextStack {
        ContextStack {
            contexts: Vec::new(),
            language: None,
            part_of_speech: None,
        }
    }
}

pub fn parse_wikitext(
    text: String,
    languages: &HashSet<&str>,
    parts_of_speech: &HashSet<&str>,
) -> Vec<Meaning> {
    let mut result: Vec<Meaning> = Vec::new();
    let mut context_stack: ContextStack = ContextStack::new();

    let stack_apply = |context_stack: &mut ContextStack, wiki_context: &dyn Fn(String) -> WikiContext, line: &str, slice: &Option<&str>| {
        slice.map_or_else(|| {
            println!("Could not parse line: {}", line);
        }, |slice| {
            context_stack.apply(
                wiki_context(slice.to_owned()),
                languages,
                parts_of_speech,
            );
        });
    };

    for line in text.lines() {
        if line.starts_with("======") && line.len() > 12 {
            stack_apply(&mut context_stack, &|x| Heading6(x), line, &line.get(6..line.len()-6));
        } else if line.starts_with("=====") && line.len() > 10 {
            stack_apply(&mut context_stack, &|x| Heading5(x), line, &line.get(5..line.len()-5));
        } else if line.starts_with("====") && line.len() > 8 {
            stack_apply(&mut context_stack, &|x| Heading4(x), line, &line.get(4..line.len()-4));
        } else if line.starts_with("===") && line.len() > 6 {
            stack_apply(&mut context_stack, &|x| Heading3(x), line, &line.get(3..line.len()-3));
        } else if line.starts_with("==") && line.len() > 4 {
            stack_apply(&mut context_stack, &|x| Heading2(x), line, &line.get(2..line.len()-2));
        } else if line.starts_with("=") && line.len() > 2 {
            stack_apply(&mut context_stack, &|x| Heading1(x), line, &line.get(1..line.len()-1));
        } else if line.starts_with("# ") {
            context_stack.language.as_ref().and_then(|language| {
                context_stack.part_of_speech.as_ref().map(|part_of_speech| {
                    result.push(Meaning {
                        language: language.clone(),
                        part_of_speech: part_of_speech.clone(),
                        definition: String::from(&line[2..]),
                    })
                })
            });
        }
    }
    result
}
