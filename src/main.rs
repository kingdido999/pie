#[macro_use]
extern crate tera;

use pulldown_cmark::{html, Parser};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use tera::Context;

fn main() -> io::Result<()> {
    let mut f = File::open("posts/issue-1.md")?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    let parser = Parser::new(&buffer);

    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);

    let mut tera = compile_templates!("templates/**/*");
    tera.autoescape_on(vec![]);

    let mut context = Context::new();
    context.insert("post", &html_buf);
    let rendered_html = tera.render("index.html", &context).unwrap();

    let mut write_buf = File::create("dist/posts/issue-1.html")?;
    write_buf.write(rendered_html.as_bytes())?;

    Ok(())
}
