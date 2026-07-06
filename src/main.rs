mod http;
mod utils;
mod handler;
mod server;
use std::io;
use std::net::{TcpListener, TcpStream};

use http::*;
use handler::StaticFileHandler;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server running on http://127.0.0.1:8080");
    println!("Serving static files from ./public");
    println!("Press Ctrl+C to stop\n");

    // Initialize the static file handler
    let static_handler = StaticFileHandler::new("./public");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Clone the handler for each thread
                let handler = static_handler.clone();
                
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(stream, handler) {
                        eprintln!(" Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!(" Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, static_handler: StaticFileHandler) -> io::Result<()> {
    // Parse the request
    let request = http::request::parse_request(&mut stream)?;
    
    // Route based on path
    let response = match request.method.as_str() {
        "GET" => {
            // Try to serve static file first
            if request.path.starts_with("/static/") || request.path == "/" {
                // Serve static files
                let path = if request.path == "/" {
                    "/index.html"
                } else {
                    // Remove /static/ prefix if present
                    request.path.trim_start_matches("/static/")
                };
                
                static_handler.handle_request(path)
            } else {
                // API routes
                match request.path.as_str() {
                    "/hello" => HttpResponse::ok("Hello, World!").content_type("text/plain"),
                    "/json" => HttpResponse::ok(r#"{"message": "Hello, JSON!"}"#).json(),
                    "/html" => {
                        let html = r#"<!DOCTYPE html>
<html>
<head><title>Welcome</title></head>
<body style="font-family: Arial; max-width: 800px; margin: 40px auto; padding: 20px;">
    <h1>🌟 Welcome to the Server!</h1>
    <p>This server supports static file serving.</p>
    <h2>Static Files</h2>
    <p>Place files in the <code>public/</code> directory:</p>
    <ul>
        <li><a href="/static/test.html">/static/test.html</a></li>
        <li><a href="/static/style.css">/static/style.css</a></li>
        <li><a href="/static/image.png">/static/image.png</a></li>
    </ul>
    <h2>API Routes</h2>
    <ul>
        <li><a href="/hello">/hello</a> - Plain text</li>
        <li><a href="/json">/json</a> - JSON response</li>
        <li><a href="/html">/html</a> - This page</li>
    </ul>
</body>
</html>"#;
                        HttpResponse::ok(html).html()
                    }
                    _ => {
                        // Try static file
                        static_handler.handle_request(&request.path)
                    }
                }
            }
        }
        "POST" => {
            // Handle POST requests
            match request.path.as_str() {
                "/api/data" => {
                    let response_body = format!(
                        r#"{{"received": true, "body": "{}"}}"#,
                        request.body
                    );
                    HttpResponse::created(response_body).json()
                }
                _ => HttpResponse::not_found()
            }
        }
        _ => {
            // Method not allowed
            HttpResponse::new_with_status(
                HttpStatus::MethodNotAllowed,
                "Method Not Allowed"
            )
        }
    };

    // Send the response
    response.send(&mut stream)?;
    
    Ok(())
}
