use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use base64::{encode}; 
use sha2::{Sha256, Digest}; 

// --- Example user database ---
// (In a real application, use a more secure method like bcrypt)
let mut users: HashMap<String, String> = HashMap::new(); 
users.insert("admin".to_string(), encode(Sha256::digest("password".as_bytes()))); 

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

    // --- Authentication Check ---
    let is_authenticated = if path.starts_with("/protected") { 
        check_authentication(&request)
    } else {
        true // Public routes don't require authentication
    };

    let (status_line, content_type, contents) = match (method, path, is_authenticated) {
        ("GET", "/", _) => {
            let filepath = public_path.join("index.html");
            let contents = read_file(&filepath);
            ("HTTP/1.1 200 OK", "text/html", contents)
        }
        ("GET", path, true) => {
            let filepath = public_path.join(path);
            if filepath.is_file() {
                let contents = read_file(&filepath);
                let content_type = get_content_type(&filepath);
                ("HTTP/1.1 200 OK", content_type, contents)
            } else {
                ("HTTP/1.1 404 NOT FOUND", "text/html", "<h1>404 Not Found</h1>".to_string())
            }
        }
        ("POST", "/upload", true) => {
            let content_length: usize = get_header_value(&request, "Content-Length")
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            let boundary = get_header_value(&request, "Content-Type")
                .and_then(|header| header.split("boundary=").nth(1))
                .unwrap_or("");

            if content_length > 0 && !boundary.is_empty() {
                let mut file_data = vec![0; content_length];
                stream.read_exact(&mut file_data).unwrap();

                let (filename, file_content) = parse_multipart_data(&file_data, boundary);
                let filepath = public_path.join(filename);
                let mut file = File::create(filepath).unwrap();
                file.write_all(&file_content).unwrap();

                ("HTTP/1.1 201 Created", "text/plain", "File uploaded successfully".to_string())
            } else {
                ("HTTP/1.1 400 Bad Request", "text/plain", "Invalid request".to_string())
            }
        },
        (_, _, false) => ("HTTP/1.1 401 Unauthorized", "text/plain", "Authentication required".to_string()),
        _ => ("HTTP/1.1 400 Bad Request", "text/html", "<h1>400 Bad Request</h1>".to_string()),
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
        // ... add more content types
    ]);

    content_types.get(extension).unwrap_or(&"text/plain")
}

fn check_authentication(request: &str) -> bool {
    let auth_header = get_header_value(request, "Authorization");
    if let Some(auth_header) = auth_header {
        if auth_header.starts_with("Basic ") {
            let base64_credentials = auth_header.split(" ").nth(1).unwrap();
            if let Ok(decoded_credentials) = base64::decode(base64_credentials) {
                if let Ok(credentials) = String::from_utf8(decoded_credentials) {
                    let mut parts = credentials.split(":");
                    let username = parts.next().unwrap_or("");
                    let password = parts.next().unwrap_or("");

                    if let Some(&hashed_password) = users.get(username) {
                        // Note: In a real application, use a more secure comparison method
                        // that avoids timing attacks.
                        return hashed_password == encode(Sha256::digest(password.as_bytes()));
                    }
                }
            }
        }
    }
    false 
}

fn get_header_value(request: &str, header_name: &str) -> Option<&str> {
    request
        .lines()
        .find(|line| line.starts_with(header_name))
        .and_then(|line| line.split(": ").nth(1))
}

// --- Multipart data parsing (simplified example, you may want to use a crate) ---
fn parse_multipart_data(data: &[u8], boundary: &str) -> (String, Vec<u8>) {
    let boundary_bytes = format!("\r\n--{}\r\n", boundary).as_bytes();
    let mut filename = String::new();
    let mut file_content = Vec::new();

    let mut parts = data.split(|&b| boundary_bytes.contains(&b));
    parts.next(); // Skip the preamble

    while let Some(part) = parts.next() {
        if let Some(header_end) = find_subsequence(part, b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&part[..header_end]);
            for line in headers.lines() {
                if line.starts_with("Content-Disposition: form-data; name=\"file\"; filename=\"") {
                    filename = line
                        .split("filename=\"")
                        .nth(1)
                        .unwrap()
                        .trim_end_matches('"')
                        .to_string();
                }
            }
            file_content.extend_from_slice(&part[header_end + 4..]);
        }
    }

    (filename, file_content)
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
