use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use mdbook_core::book::{Book, BookItem};
use mdbook_core::errors::Error;
use regex::Regex;
use std::io;
use serde_json;

pub struct DrawioPreprocessor;

impl DrawioPreprocessor {
    pub fn new() -> DrawioPreprocessor {
        DrawioPreprocessor
    }

    fn convert_github_to_raw(&self, github_url: &str) -> String {
        github_url
            .replace("github.com", "raw.githubusercontent.com")
            .replace("/blob/", "/")
    }

    fn create_iframe(&self, url: &str) -> String {
        let raw_url = self.convert_github_to_raw(url);
        format!(
            r#"<iframe class="drawio-viewer" src="https://viewer.diagrams.net/?highlight=0000ff&edit=_blank&layers=1&nav=1&title=diagram&url={}"></iframe>"#,
            raw_url
        )
    }
}

impl Preprocessor for DrawioPreprocessor {
    fn name(&self) -> &str {
        "drawio"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let regex = Regex::new(r"@drawio\{(https://github\.com/[^\s}]+\.drawio)\}").unwrap();
        
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapter.content = regex.replace_all(&chapter.content, |caps: &regex::Captures| {
                    self.create_iframe(&caps[1])
                }).to_string();
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool, Error> {
        Ok(renderer == "html")
    }
}

fn main() -> Result<(), Error> {
    let preprocessor = DrawioPreprocessor::new();

    if let Some(arg) = std::env::args().nth(1) {
        if arg == "supports" {
            let renderer = std::env::args().nth(2).unwrap_or_default();
            if preprocessor.supports_renderer(&renderer)? {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }
    }

    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;
    let processed_book = preprocessor.run(&ctx, book)?;
    
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}