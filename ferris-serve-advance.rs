use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: cargo run <port> <directory>");
        std::process::exit(1);
    }

    let port = &args[1];
    let public_path = Path::new(&args[2]);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    println!("Server listening on port {}", port);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let public_path = public_path.to_owned(); 
        thread::spawn(move || {
            handle_connection(stream, &public_path);
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, public_path: &Path) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer);
    let (method, path, _) = parse_request(&request);

    let (status_line, content_type, contents) = match (method, path) {
        ("GET", "/") => {
            let filepath = public_path.join("index.html");
            let contents = read_file(&filepath);
            ("HTTP/1.1 200 OK", "text/html", contents)
        }
        ("GET", path) => {
            let filepath = public_path.join(path);
            if filepath.is_file() {
                let contents = read_file(&filepath);
                let content_type = get_content_type(&filepath);
                ("HTTP/1.1 200 OK", content_type, contents)
            } else {
                let filepath = public_path.join("404.html");
                let contents = read_file(&filepath);
                ("HTTP/1.1 404 NOT FOUND", "text/html", contents)
            }
        }
        _ => ("HTTP/1.1 400 BAD REQUEST", "text/html", "<h1>400 Bad Request</h1>".to_string()),
    };

    let response = format!(
        "{}Content-Type: {}\r\n\r\n{}",
        status_line, content_type, contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn parse_request(request: &str) -> (&str, &str, &str) {
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");
    let version = parts.next().unwrap_or("");
    (method, path, version)
}

fn read_file(filepath: &Path) -> String {
    fs::read_to_string(filepath).unwrap_or_else(|_| {
        format!("Error reading file: {}", filepath.to_str().unwrap())
    })
}

fn get_content_type(filepath: &Path) -> &str {
    let extension = filepath.extension().and_then(|s| s.to_str()).unwrap_or("");
    let content_types = HashMap::from([
        ("html", "text/html"),
        ("css", "text/css"),
        ("js", "text/javascript"),
        ("png", "image/png"),
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("gif", "image/gif"),
        // ... add more content types as needed
    ]);

    content_types.get(extension).unwrap_or(&"text/plain")
}
