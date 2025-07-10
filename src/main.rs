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

fn generate_site() {

}

fn parse_markdown_files(src: &Path, dest: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(MD).unwrap();
        let target_path = dest.join(relative_path);
        if path.is_dir() {
            fs::create_dir_all(&target_path)?;
            parse_markdown_files(&path, dest)?;
        } else if let Some(extension) = path.extension() {
            if extension == MD {
                let markdown = fs::read_to_string(&path).unwrap();
                let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());

                let mut html = templates::HEADER.to_string(); 
                let mut body = String::new();
                pulldown_cmark::html::push_html(&mut body, parser);
                html.push_str(templates::create_body(&body).as_str());
                html.push_str(templates::FOOTER);

                let html_target = target_path.with_extension(HTML);
                fs::create_dir_all(html_target.parent().unwrap())?;
                fs::write(html_target, html)?;
            } else if extension == HTML {
                fs::create_dir_all(target_path.parent().unwrap())?;
                fs::copy(&path, &target_path)?;
            }
        }
    }
    generate_site();
    Ok(())
}
