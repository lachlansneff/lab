use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write;
use std::iter;

use mdbook::book::{Book, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{Event, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;

#[derive(Default)]
pub struct TableOfContents;

fn build_toc(headers: impl IntoIterator<Item=(u32, String, String)>, dest: &mut String) -> impl Iterator<Item=Event> {
    let mut headers = headers.into_iter().peekable();

    let mut last_lower = match headers.peek() {
        Some((lvl, _, _)) => *lvl,
        None => 0,
    };

    let normalized = headers.map(|(level, slug, name)| {
        let level = match (last_lower + 1).cmp(&level) {
            Ordering::Less => last_lower + 1,
            _ => {
                last_lower = level;
                level
            }
        };
        (level, slug, name)
    });

    for (level, slug, name) in normalized {
        let width = 2 * (level - 1) as usize;
        writeln!(dest, "{1:0$}* [{name}](#{slug})", width, "", name=name, slug=slug).unwrap();
    }
    
    iter::once(Event::Html("<div id=\"toc\"><p class=\"toc-title\">Contents:</p>\n".into()))
        .chain(Parser::new(dest))
        .chain(iter::once(Event::Html("</div>\n".into())))
}

fn generate_toc(chapter: &mut Chapter) {
    let mut events = Parser::new(&chapter.content);

    let mut headers = vec![];
    let mut id_counter = HashMap::new();
    while let Some(event) = events.next() {
        if let Event::Start(Tag::Heading(level)) = event {
            let mut heading = String::new();

            for event in (&mut events).take_while(|event| !matches!(event, Event::End(Tag::Heading(_)))) {
                match event {
                    Event::Text(header) => {
                        write!(heading, "{}", header).unwrap();
                    }
                    Event::Code(code) => {
                        write!(heading, "`{}`", code).unwrap();
                    }
                    _ => {}
                }
            }

            let mut slug = mdbook::utils::normalize_id(&heading);
            {
                let count = id_counter.entry(slug.clone()).or_insert(0);

                if *count > 0 {
                    write!(slug, "-{}", count).unwrap();
                }
                *count += 1;
            }

            headers.push((level, slug, heading));
        }
    }

    let mut temp = String::new();
    let toc = build_toc(headers, &mut temp);
    let mut toc_str = String::new();

    cmark(toc, &mut toc_str, None).expect("failed to regenerate markdown");

    // let toc_str = format!("{}\n{}", toc_str, chapter.content);

    chapter.content = format!("{}\n{}", toc_str, chapter.content);

    // let old_content = mem::replace(&mut chapter.content, String::new());

    // let new_events = toc.chain(Parser::new(&old_content));

    // cmark(new_events, &mut chapter.content, None).expect("failed to regenerate markdown");
}

impl Preprocessor for TableOfContents {
    fn name(&self) -> &str {
        todo!()
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> mdbook::errors::Result<Book> {
        book.for_each_mut(|item| {
            match item {
                mdbook::BookItem::Chapter(chapter) => generate_toc(chapter),
                _ => {},
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}
