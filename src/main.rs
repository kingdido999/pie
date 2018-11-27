#[macro_use]
extern crate tera;

use pulldown_cmark::{html, Parser};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::path::Path;
use tera::Context;

fn main() -> Result<()> {
    let mut tera = compile_templates!("templates/**/*");
    tera.autoescape_on(vec![]);

    for entry in fs::read_dir("posts")? {
        let path = entry?.path();
        let html_buf = convert_markdown_to_html(&path)?;

        let mut context = Context::new();
        context.insert("content", &html_buf);
        let rendered_html = tera.render("layouts/post.html", &context).unwrap();

        let output_path = format!(
            "dist/posts/{}.html",
            &path.file_stem().unwrap().to_str().unwrap()
        );
        let mut write_buf = File::create(&output_path)?;
        write_buf.write(rendered_html.as_bytes())?;
    }

    Ok(())
}

fn convert_markdown_to_html<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut f = File::open(path)?;
    let mut md_buf = String::new();
    f.read_to_string(&mut md_buf)?;

    let parser = Parser::new(&md_buf);
    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);

    Ok(html_buf)
}
