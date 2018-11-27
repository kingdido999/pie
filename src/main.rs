#[macro_use]
extern crate tera;
#[macro_use]
extern crate serde_derive;

extern crate serde;

use pulldown_cmark::{html, Parser};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use tera::Context;

#[derive(Serialize)]
struct Post {
    file_name: String,
    title: String,
}

fn main() {
    let mut tera = compile_templates!("templates/**/*");
    tera.autoescape_on(vec![]);

    let mut post_list = vec![];
    let entries = fs::read_dir("posts").expect("Unable to read posts.");

    // Generate posts
    for entry in entries {
        let path = entry.unwrap().path();
        let mut file = File::open(&path).expect("Unable to open file.");
        let buf_reader = BufReader::new(&file);
        let title = extract_title_string(buf_reader);

        // Go back to the beginning of the file for the second reading
        file.seek(SeekFrom::Start(0)).unwrap();
        let html_buf = convert_markdown_to_html(file);

        let post = Post {
            file_name: format!("{}.html", path.file_stem().unwrap().to_str().unwrap()),
            title: title,
        };

        let mut context = Context::new();
        context.insert("content", &html_buf);
        let rendered_html = tera.render("layouts/post.html", &context).unwrap();

        let output_path = format!("dist/posts/{}", post.file_name);
        let mut write_buf = File::create(&output_path).expect("Unable to create file.");
        write_buf
            .write(rendered_html.as_bytes())
            .expect("Unable to write file.");
        post_list.push(post);
    }

    // Generate home page
    let mut context = Context::new();
    context.insert("post_list", &post_list);
    let rendered_html = tera.render("index.html", &context).unwrap();
    let mut write_buf = File::create("dist/index.html").expect("Unable to create file.");
    write_buf
        .write(rendered_html.as_bytes())
        .expect("Unable to write file.");
}

fn convert_markdown_to_html(mut file: File) -> String {
    let mut md_buf = String::new();
    file.read_to_string(&mut md_buf)
        .expect("Unable to read file");

    let parser = Parser::new(&md_buf);
    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);
    html_buf
}

fn extract_title_string<R: BufRead>(mut rdr: R) -> String {
    let mut first_line = String::new();
    rdr.read_line(&mut first_line).expect("Unable to read line");

    let last_hash = first_line
        .char_indices()
        .skip_while(|&(_, c)| c == '#')
        .next()
        .map_or(0, |(idx, _)| idx);

    first_line[last_hash..].trim().into()
}
