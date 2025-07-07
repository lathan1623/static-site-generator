use std::{fs, path::Path, error::Error};

fn main() -> std::io::Result<()> {
    let _ = fs::create_dir("./static");
    build_static_dir("md");
    Ok(())
}

/// read through the md directory, looking for .md files
/// we want to convert those .md files into html files, so they can eventually be linked to in
/// index.html
/// if we find a sub-directory in here. we still want to look through there for .md files too
/// index.html should not genereate links to those .md files directly. Instead it should create a
/// link to a page with the name of the directory. that page will then link to those .md files in
/// the folder
/// the goal of this function should be to convert all .md files into .html files. and move them to
/// the static directory.
fn build_static_dir(dir_path: &str) {
    let destination = Path::new("./static");
    let dir = fs::read_dir(dir_path).expect("Cannot find the {dir_path} directory");
    for entry in dir {
        let path = entry.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "md" {
                let markdown = fs::read_to_string(&path).unwrap();
                let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());
                let mut body = String::new();
                pulldown_cmark::html::push_html(&mut body, parser);
                let mut file_name = std::path::PathBuf::from("./static");
                file_name.push(&path.file_stem().unwrap());
                file_name.set_extension("html");
                let _ = fs::write(file_name, body);
            }
            else if extension == "html" {
                let name = path.file_name().unwrap();
                //TODO convert md to html, just move html directly
                let _ = fs::copy(&path, &destination.join(name));
            }
        } else {
            //TODO handle directories here with recursion

        }
    }
}
