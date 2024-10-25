use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use std::fs;
use std::path::Path;

fn process_content(content: &str) -> String {
    if content.starts_with('[') {
        content
            .lines()
            .map(|line| line.trim_start())
            .filter(|line| !line.starts_with("//"))
            .collect::<Vec<&str>>()
            .join(" ")
    } else {
        content.to_string()
    }
}

#[get("/{filename:.*}")]
async fn serve_file(filename: actix_web::web::Path<String>) -> impl Responder {
    let path = Path::new("web").join(filename.as_str());

    if let Ok(content) = fs::read(&path) {
        if let Ok(content_str) = String::from_utf8(content.clone()) {
            let processed_content = process_content(&content_str);
            println!("Ok: {} - {}", filename.as_str(), processed_content);
            HttpResponse::Ok().body(processed_content)
        } else {
            println!("Ok (binary): {}", filename.as_str());
            HttpResponse::Ok().body(content)
        }
    } else {
        println!("Not found: {}", filename.as_str());
        HttpResponse::NotFound().body("File not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at http://localhost:8081");

    HttpServer::new(|| App::new().service(serve_file))
        .bind("0.0.0.0:8081")?
        .run()
        .await
}
