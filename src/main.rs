use std::{fs, path::Path};

mod templates;

fn main() -> std::io::Result<()> {
    let _ = fs::create_dir("public");
    parse_markdown_files(Path::new("md"), Path::new("public"))?;
    Ok(())
}

fn parse_markdown_files(src: &Path, dest: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix("md").unwrap();
        let target_path = dest.join(relative_path);
        if path.is_dir() {
            fs::create_dir_all(&target_path)?;
            parse_markdown_files(&path, dest)?;
        } else if let Some(extension) = path.extension() {
            if extension == "md" {
                let markdown = fs::read_to_string(&path).unwrap();
                let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());

                let mut html = templates::HEADER.to_string(); 
                let mut body = String::new();
                pulldown_cmark::html::push_html(&mut body, parser);
                html.push_str(templates::create_body(&body).as_str());
                html.push_str(templates::FOOTER);

                let html_target = target_path.with_extension("html");
                fs::create_dir_all(html_target.parent().unwrap())?;
                fs::write(html_target, html)?;
            } else if extension == "html" {
                fs::create_dir_all(target_path.parent().unwrap())?;
                fs::copy(&path, &target_path)?;
            }
        }
    }
    Ok(())
}
