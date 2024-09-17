use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let public_path = Path::new("public");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, public_path);
    }
}

fn handle_connection(mut stream: TcpStream, public_path: &Path) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get_index = b"GET / HTTP/1.1\r\n";
    let get_file = b"GET /";

    let (status_line, filename) = if buffer.starts_with(get_index) {
        ("HTTP/1.1 200 OK\r\n\r\n", public_path.join("index.html")) 
    } else if buffer.starts_with(get_file) {
        let path = String::from_utf8_lossy(&buffer[5..]).split_whitespace().next().unwrap();
        let filepath = public_path.join(path);
        if filepath.is_file() {
            ("HTTP/1.1 200 OK\r\n\r\n", filepath) 
        } else {
            ("HTTP/1.1 404 NOT FOUND\r\n\r\n", public_path.join("404.html")) 
        }
    } else {
        ("HTTP/1.1 400 BAD REQUEST\r\n\r\n", public_path.join("400.html")) 
    };

    let contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        format!("Error reading file") 
    });

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
