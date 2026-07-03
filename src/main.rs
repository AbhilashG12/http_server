use std::io;
use std::net::{TcpStream,TcpListener};
pub mod http;
use http::response::HttpResponse;
use http::*;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("🚀 Server running on http://127.0.0.1:8080");
    println!("Press Ctrl+C to stop\n");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let req = request::parse_request(&mut stream)?;
    let res = match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") | ("GET", "/hello") => {
            HttpResponse::ok("Hello, World!")
                .content_type("text/plain")
        }
        ("GET", "/html") => {
            let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Welcome</title>
    <style>
        body { font-family: Arial; max-width: 600px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
    </style>
</head>
<body>
    <h1>🌟 Welcome to My HTTP Server!</h1>
    <p>This is a HTML response from Phase 2.</p>
    <p>Try these endpoints:</p>
    <ul>
        <li><a href="/">/</a> - Plain text</li>
        <li><a href="/json">/json</a> - JSON response</li>
        <li><a href="/notfound">/notfound</a> - 404 error</li>
    </ul>
</body>
</html>"#;
            HttpResponse::ok(html).html()
        }
        ("GET", "/json") => {
            let json = r#"{"message": "Hello, JSON!", "version": "1.0"}"#;
            HttpResponse::ok(json).json()
        }
        ("POST", "/api/data") => {
            let response_body = format!(
                r#"{{"received": true, "body": "{}"}}"#,
                req.body
            );
            HttpResponse::created(response_body).json()
        }
        _ => {
            let html = r#"<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body style="font-family: Arial; padding: 40px; text-align: center;">
    <h1 style="color: #c0392b;">404</h1>
    <p>The page you're looking for doesn't exist.</p>
    <a href="/">Go Home</a>
</body>
</html>"#;
            HttpResponse::new_with_status(HttpStatus::NotFound, html).html()
        }
    };

    res.send(&mut stream)?;
    
    Ok(())
}  
