use std::{error::Error, fs, path::Path, thread, time::Duration};

use anyhow::Context;
use axum::Router;

mod templates;

const PUBLIC_DIR: &str = "public";
const CONTENT: &str = "content";
const HTML: &str = "html";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    build_html(Path::new(CONTENT), Path::new(PUBLIC_DIR))?;
    
    tokio::task::spawn_blocking(move || {
       let mut hotwatch = hotwatch::Hotwatch::new().unwrap(); 
       hotwatch
           .watch(CONTENT, |_| {
               println!("Detected change in content folder");
               build_html(Path::new(CONTENT), Path::new(PUBLIC_DIR)).unwrap();
           })
           .expect("Failed to watch for changes in content folder");
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });
    
    let service = tower_http::services::ServeDir::new(PUBLIC_DIR);
    let app = Router::new().fallback_service(service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

fn generate_links(files: &[String], dest: &Path) -> Result<String, Box<dyn Error>> {
    let links = files
        .iter()
        .map(|file| {
            let file = file.trim_start_matches(dest.to_str().unwrap());
            let title = file.trim_start_matches("/").trim_end_matches(".html").trim_end_matches("index");
            format!(r#"<a href=".{}">{}</a>"#, file, title)
        })
        .collect::<Vec<String>>()
        .join("<br/>\n");
    
    Ok(links)
}

fn generate_default_index_html(html_files: &[String], dest: &Path) -> Result<String, Box<dyn Error>> {
    let mut html = templates::HEADER.to_owned();
    let body = generate_links(html_files, dest)?;
    
    html.push_str(templates::create_body(&body).as_str());
    html.push_str(templates::FOOTER);
    
    Ok(html)
}

fn build_index_html(src: &Path, dest: &Path, html_files: &[String]) -> Result<(), Box<dyn Error>> {
    let index_path = src.join("index.html");
    
    if !index_path.exists() {
        let default_html = generate_default_index_html(html_files, dest)?;
        fs::write(&index_path, default_html)?;
    }
    
    let custom_html = fs::read_to_string(&index_path)?;
    let links = generate_links(html_files, dest)?;
    let final_html = custom_html.replace("{{LINKS}}", &links);
    
    let dest_index_path = dest.join("index.html");
    fs::write(dest_index_path, final_html)?;
    
    Ok(())
}

fn build_html(src: &Path, dest: &Path) -> Result<(), Box<dyn Error>> {
    let mut html_files: Vec<String> = Vec::new();
    if fs::exists(dest)? {
        fs::remove_dir_all(dest)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.file_name().unwrap();
        let target_path = dest.join(relative_path);

        let mut html = String::from(templates::HEADER);
        let html_target = target_path.with_extension(HTML);
        let path_name = path.file_stem().unwrap().to_str().unwrap(); 
        html.push_str(&templates::create_title(path_name));

        if path.is_dir() {
            fs::create_dir_all(&target_path)?;
            build_html(&path, &target_path)?;
            // the recursive call to parse_markdown_files() will generate an index.html in the
            // sub-directory
            let mut generated_file = target_path.to_owned().into_os_string().into_string().unwrap();
            generated_file.push_str("/index.html");
            html_files.push(generated_file);
        } else {
            let extension = path.extension().context("Failed to parse extension from file path")?.to_str().unwrap();
            match extension {
                "css" => {
                    fs::copy(&path, dest.join(path.file_name().unwrap()))?;
                },
                "html" => {
                    continue;
                },
                _ => {
                    let markdown = fs::read_to_string(&path).unwrap();
                    let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());
        
                    let mut body = String::new();
                    pulldown_cmark::html::push_html(&mut body, parser);
                    html.push_str(templates::create_body(&body).as_str());
                    html.push_str(templates::FOOTER);
        
                    fs::create_dir_all(html_target.parent().unwrap())?;
                    fs::write(&html_target, &html)?;
                    html_files.push(html_target.into_os_string().into_string().unwrap());
                }
            }
        }
    }

    build_index_html(src, dest, &html_files)?;
    Ok(())
}
