use std::{error::Error, fs, path::Path, thread, time::Duration};

use axum::Router;

mod templates;

const PUBLIC_DIR: &str = "public";
const MD: &str = "md";
const HTML: &str = "html";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    parse_markdown_files(Path::new(MD), Path::new(PUBLIC_DIR))?;
    
    tokio::task::spawn_blocking(move || {
       let mut hotwatch = hotwatch::Hotwatch::new().unwrap(); 
       hotwatch
           .watch(MD, |_| {
               println!("Detected change in md folder");
               parse_markdown_files(Path::new(MD), Path::new(PUBLIC_DIR)).unwrap();
           })
           .expect("Failed to watch for changes in md folder");
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

fn generate_site(files: Vec<String>, dest: &Path) -> Result<(), Box<dyn Error>> {
    let mut html = templates::HEADER.to_owned();
    let body = files
        .into_iter()
        .map(|file| {
            let file = file.trim_start_matches(dest.to_str().unwrap());
            let title = file.trim_start_matches("/").trim_end_matches(".html").trim_end_matches("index");
            format!(r#"<a href=".{}">{}</a>"#, file, title)
        })
        .collect::<Vec<String>>()
        .join("<br />\n");
    
    html.push_str(templates::create_body(&body).as_str());
    html.push_str(templates::FOOTER);

    let index_path = dest.join("index.html");
    fs::write(index_path, html)?;
    Ok(())
}

fn parse_markdown_files(src: &Path, dest: &Path) -> Result<(), Box<dyn Error>> {
    let mut generated_files: Vec<String> = Vec::new();
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
            parse_markdown_files(&path, &target_path)?;
            // the recursive call to parse_markdown_files() will generate an index.html in the
            // sub-directory
            let mut generated_file = target_path.to_owned().into_os_string().into_string().unwrap();
            generated_file.push_str("/index.html");
            generated_files.push(generated_file);
        } else {
            let markdown = fs::read_to_string(&path).unwrap();
            let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());

            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, parser);
            html.push_str(templates::create_body(&body).as_str());

            fs::create_dir_all(html_target.parent().unwrap())?;
            fs::write(&html_target, &html)?;
            generated_files.push(html_target.into_os_string().into_string().unwrap());
        }
        html.push_str(templates::FOOTER);
    }

    generate_site(generated_files, dest)?;
    Ok(())
}
