use std::collections::HashMap;
use std::io::{self, Write};
use std::net::TcpStream;

use super::status::HttpStatus;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    /// Creates a new empty response with default status 200 OK
    pub fn new() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        headers.insert("Connection".to_string(), "close".to_string());
        
        Self {
            status: HttpStatus::ok,
            headers,
            body: String::new(),
        }
    }

    /// Creates a 200 OK response with the given body
    pub fn ok(body: impl Into<String>) -> Self {
        let mut response = Self::new();
        response.body = body.into();
        response.status = HttpStatus::ok;
        response.set_content_length();
        response
    }

    /// Creates a 404 Not Found response
    pub fn not_found() -> Self {
        let body = "404 Not Found".to_string();
        let mut response = Self::new();
        response.body = body;
        response.status = HttpStatus::NotFound;
        response.set_content_length();
        response
    }

    /// Creates a 400 Bad Request response
    pub fn bad_request() -> Self {
        let body = "400 Bad Request".to_string();
        let mut response = Self::new();
        response.body = body;
        response.status = HttpStatus::BadRequest;
        response.set_content_length();
        response
    }

    /// Creates a 500 Internal Server Error response
    pub fn internal_error() -> Self {
        let body = "500 Internal Server Error".to_string();
        let mut response = Self::new();
        response.body = body;
        response.status = HttpStatus::InternalServerError;
        response.set_content_length();
        response
    }

    /// Creates a 201 Created response
    pub fn created(body: impl Into<String>) -> Self {
        let mut response = Self::new();
        response.body = body.into();
        response.status = HttpStatus::Created;
        response.set_content_length();
        response
    }

    /// Creates a response with custom status and body
    pub fn new_with_status(status: HttpStatus, body: impl Into<String>) -> Self {
        let mut response = Self::new();
        response.body = body.into();
        response.status = status;
        response.set_content_length();
        response
    }

    /// Sets a header value
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets multiple headers at once
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Sets the content length based on the body
    pub fn set_content_length(&mut self) {
        let len = self.body.len();
        self.headers.insert("Content-Length".to_string(), len.to_string());
    }

    /// Sets the content type
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.headers.insert("Content-Type".to_string(), content_type.into());
        self
    }

    /// Sets JSON content type
    pub fn json(mut self) -> Self {
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self
    }

    /// Sets HTML content type
    pub fn html(mut self) -> Self {
        self.headers.insert("Content-Type".to_string(), "text/html".to_string());
        self
    }

    /// Serializes the response to a string
    pub fn to_string(&self) -> String {
        let mut response = String::new();
        
        // Status line
        response.push_str(&format!("HTTP/1.1 {}\r\n", self.status));
        
        // Headers
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        // Empty line separating headers from body
        response.push_str("\r\n");
        
        // Body
        response.push_str(&self.body);
        
        response
    }

    /// Sends the response to the TCP stream
    pub fn send(&self, stream: &mut TcpStream) -> io::Result<()> {
        let response_string = self.to_string();
        stream.write_all(response_string.as_bytes())?;
        stream.flush()?;
        
        println!("\n--- [SENT HTTP RESPONSE] ---");
        println!("Status: {}", self.status);
        println!("Body: {}", self.body);
        println!("-----------------------------\n");
        
        Ok(())
    }

    /// Sends a quick hello response (backward compatibility)
    pub fn send_hello(stream: &mut TcpStream) -> io::Result<()> {
        let response = HttpResponse::ok("Hello world")
            .content_type("text/plain");
        response.send(stream)
    }

    /// Sends a JSON response
    pub fn send_json(stream: &mut TcpStream, data: &str) -> io::Result<()> {
        let response = HttpResponse::ok(data)
            .json();
        response.send(stream)
    }

    /// Sends an HTML response
    pub fn send_html(stream: &mut TcpStream, html: &str) -> io::Result<()> {
        let response = HttpResponse::ok(html)
            .html();
        response.send(stream)
    }
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for common response types
pub mod response_helpers {
    use super::*;

    /// Creates an HTML response with a simple page
    pub fn html_page(title: &str, content: &str) -> HttpResponse {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>{}</p>
</body>
</html>"#,
            title, title, content
        );
        HttpResponse::ok(html).html()
    }

    /// Creates a JSON error response
    pub fn json_error(message: &str, status: HttpStatus) -> HttpResponse {
        let json = format!(r#"{{"error": "{}", "status": {}}}"#, message, status.code());
        HttpResponse::new_with_status(status, json).json()
    }
}

// Re-export commonly used items
pub use response_helpers::*;
