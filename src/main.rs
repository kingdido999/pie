extern crate glob;
extern crate serde;
extern crate chrono;

#[macro_use]
extern crate serde_derive;

use glob::glob;
use pulldown_cmark::{html, Parser};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Result, SeekFrom};
use std::path::Path;
use tera::{Context, Tera};
use chrono::prelude::*;

#[derive(Serialize, Eq, Ord, PartialEq, PartialOrd)]
struct Item {
    url: String,
    title: String,
    created_at: String,
    updated_at: String,
}

fn main() -> Result<()> {
    let tera = setup_template_engine();
    let input_dir_str = "docs";
    let output_dir_str = "dist";
    let output_dir = Path::new(output_dir_str);

    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }

    fs::create_dir(output_dir)?;
    let mut item_list = vec![];

    // Generate pages
    for entry in glob(&format!("{}/**/*.md", input_dir_str)).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let mut file = File::open(&path)?;
                let buf_reader = BufReader::new(&file);
                let title = extract_title_string(buf_reader);

                // Go back to the beginning of the file for the second reading
                file.seek(SeekFrom::Start(0))?;
                let html_buf = convert_markdown_to_html(file);

                let metadata = fs::metadata(&path)?;
                let created_at = metadata.created()?;
                let updated_at = metadata.modified()?;
		let created_at = DateTime::<Utc>::from(created_at)
                    .format("%Y-%m-%d %H:%M")
                    .to_string();
                let updated_at = DateTime::<Utc>::from(updated_at)
                    .format("%Y-%m-%d %H:%M")
                    .to_string();

                let mut context = Context::new();
                context.insert("content", &html_buf);
                context.insert("created_at", &created_at);
                context.insert("updated_at", &updated_at);
                let rendered_html = tera.render("page.html", &context).unwrap();

                let output_path_str = path
                    .to_str()
                    .unwrap()
                    .replace(input_dir_str, output_dir_str)
                    .replace(".md", ".html");
                let output_path = Path::new(&output_path_str);
                let parent_dir = output_path.parent().unwrap();

                if parent_dir.exists() == false {
                    fs::create_dir_all(parent_dir)?;
                }

                let mut write_buf = File::create(&output_path)?;
                write_buf.write(rendered_html.as_bytes())?;
                let url = output_path_str.replace(output_dir_str, "");

                let item = Item {
                    title: title,
                    url: url,
                    created_at: created_at,
                    updated_at: updated_at,
                };

                item_list.push(item);
            }
            Err(e) => println!("{:?}", e),
        }
    }
    
    item_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Generate home page
    let mut context = Context::new();
    context.insert("item_list", &item_list);
    let rendered_html = tera.render("index.html", &context).unwrap();
    let mut write_buf = File::create(format!("{}/index.html", output_dir_str))?;
    write_buf.write(rendered_html.as_bytes())?;

    // Copy assets
    let css = include_str!("theme/assets/css/style.css");
    fs::create_dir_all("dist/assets/css")?;
    fs::write("dist/assets/css/style.css", css)?;

    Ok(())
}

fn setup_template_engine() -> Tera {
    let mut tera = Tera::default();
    tera.add_raw_template("base.html", include_str!("theme/base.html"))
        .unwrap();
    tera.add_raw_template("page.html", include_str!("theme/page.html"))
        .unwrap();
    tera.add_raw_template("index.html", include_str!("theme/index.html"))
        .unwrap();
    tera.autoescape_on(vec![]);
    tera
}

fn convert_markdown_to_html(mut file: File) -> String {
    let mut md_buf = String::new();
    file.read_to_string(&mut md_buf).unwrap();

    let parser = Parser::new(&md_buf);
    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);
    html_buf
}

fn extract_title_string<R: BufRead>(mut rdr: R) -> String {
    let mut first_line = String::new();
    rdr.read_line(&mut first_line).unwrap();

    let last_hash = first_line
        .char_indices()
        .skip_while(|&(_, c)| c == '#')
        .next()
        .map_or(0, |(idx, _)| idx);

    first_line[last_hash..].trim().into()
}
