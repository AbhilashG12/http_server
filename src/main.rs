mod error;
mod handler;
mod http;
mod server;
mod utils;

use std::io;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ctrlc;

use error::{ServerError, ServerResult, WithContext};
use handler::{ErrorHandler, StaticFileHandler};
use http::*;
use server::{ServerConfig, ThreadPool};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

fn main() -> ServerResult<()> {
    setup_signal_handler()?;

    let config = match server::config::load_from_env_file() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            eprintln!("Using default configuration...");
            ServerConfig::new()
        }
    };

    if !config.public_dir.exists() {
        println!(" Creating public directory: {}", config.public_dir.display());
        std::fs::create_dir_all(&config.public_dir)
            .with_context(|| "Failed to create public directory".to_string())?;
    }

    if let Err(e) = config.validate() {
        eprintln!(" Invalid configuration: {}", e);
        std::process::exit(1);
    }
    config.print();
    println!();

    let pool = ThreadPool::new(config.thread_count);
    println!(" Server starting on {}", config.bind_address());
    println!(" Serving static files from {}", config.public_dir.display());
    println!(" Using {} worker threads", pool.worker_count());
    println!("Press Ctrl+C to stop\n");

    let listener = TcpListener::bind(config.bind_address())
        .with_context(|| format!("Failed to bind to {}", config.bind_address()))?;
    
    listener.set_nonblocking(false)?;

    let static_handler = StaticFileHandler::new(config.public_dir);
    let static_handler = Arc::new(static_handler);
    let error_handler = Arc::new(ErrorHandler::new());

    for stream_result in listener.incoming() {
        if SHUTDOWN.load(Ordering::SeqCst) {
            println!("\n Received shutdown signal, stopping accept loop...");
            break;
        }

        match stream_result {
            Ok(stream) => {
                let handler = Arc::clone(&static_handler);
                let error_handler = Arc::clone(&error_handler);
                
                if let Err(e) = pool.execute(move || {
                    if let Err(e) = handle_client(stream, handler, error_handler) {
                        eprintln!("Error handling connection: {}", e);
                    }
                }) {
                    eprintln!("Failed to execute job: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!(" Connection failed: {}", e);
            }
        }
    }

    println!("\n Initiating graceful shutdown...");
    
    println!("⏳ Waiting for workers to finish...");
    std::thread::sleep(Duration::from_secs(1));
    
    if let Err(e) = pool.shutdown() {
        eprintln!(" Error during thread pool shutdown: {}", e);
    }

    println!(" Server shutdown complete");
    Ok(())
}

fn setup_signal_handler() -> ServerResult<()> {
    ctrlc::set_handler(|| {
        println!("\n  Ctrl+C received!");
        println!(" Shutting down gracefully...");
        SHUTDOWN.store(true, Ordering::SeqCst);
    }).map_err(|e| ServerError::Internal(format!("Failed to set signal handler: {}", e)))?;
    
    Ok(())
}

fn handle_client(
    mut stream: std::net::TcpStream,
    static_handler: Arc<StaticFileHandler>,
    error_handler: Arc<ErrorHandler>,
) -> ServerResult<()> {
    use std::time::Duration;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
    
    let request = match http::request::parse_request(&mut stream) {
        Ok(req) => req,
        Err(e) => {
            eprintln!(" Failed to parse request: {}", e);
            let response = error_handler.bad_request("Invalid HTTP request");
            response.send(&mut stream)?;
            return Ok(());
        }
    };
    
    let response = match route_request(&request, &static_handler, &error_handler) {
        Ok(response) => response,
        Err(e) => {
            eprintln!(" Error processing request: {}", e);
            error_handler.handle_error(e)
        }
    };

    response.send(&mut stream)?;
    
    Ok(())
}

fn route_request(
    request: &http::HttpRequest,
    static_handler: &StaticFileHandler,
    error_handler: &ErrorHandler,
) -> ServerResult<HttpResponse> {
    match request.method.as_str() {
        "GET" => handle_get_request(request, static_handler, error_handler),
        "POST" => handle_post_request(request, error_handler),
        "PUT" => handle_put_request(request, error_handler),
        "DELETE" => handle_delete_request(request, error_handler),
        "HEAD" => handle_head_request(request, static_handler, error_handler),
        _ => {
            Err(ServerError::BadRequest(format!(
                "Method not allowed: {}",
                request.method
            )))
        }
    }
}

fn handle_get_request(
    request: &http::HttpRequest,
    static_handler: &StaticFileHandler,
    error_handler: &ErrorHandler,
) -> ServerResult<HttpResponse> {
    if request.path.starts_with("/static/") || request.path == "/" {
        let path = if request.path == "/" {
            "/index.html"
        } else {
            request.path.trim_start_matches("/static/")
        };
        
        match static_handler.serve_file(path) {
            Ok(response) => Ok(response),
            Err(e) => Err(ServerError::from(e)),
        }
    } else {
        match request.path.as_str() {
            "/" | "/hello" => {
                Ok(HttpResponse::ok("Hello, World!")
                    .content_type("text/plain"))
            }
            "/json" => {
                Ok(HttpResponse::ok(r#"{"message": "Hello, JSON!", "version": "1.0"}"#)
                    .json())
            }
            "/html" => {
                let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Welcome</title>
    <style>
        body { font-family: Arial; max-width: 800px; margin: 40px auto; padding: 20px; }
        h1 { color: #2c3e50; }
        .info { background: #ecf0f1; padding: 15px; border-radius: 5px; }
    </style>
</head>
<body>
    <h1> My HTTP Server</h1>
    <div class="info">
        <p><strong>Status:</strong> Running</p>
        <p><strong>Thread Pool:</strong> Active</p>
        <p><strong>Error Handling:</strong> Enhanced</p>
    </div>
    <p>Try these endpoints:</p>
    <ul>
        <li><a href="/hello">/hello</a> - Plain text</li>
        <li><a href="/json">/json</a> - JSON response</li>
        <li><a href="/static/index.html">/static/index.html</a> - Static file</li>
    </ul>
</body>
</html>"#;
                Ok(HttpResponse::ok(html).html())
            }
            "/health" => {
                Ok(HttpResponse::ok(r#"{"status": "ok", "threads": 4}"#)
                    .json())
            }
            _ => {
                match static_handler.serve_file(&request.path) {
                    Ok(response) => Ok(response),
                    Err(e) => Err(ServerError::from(e)),
                }
            }
        }
    }
}

fn handle_post_request(
    request: &http::HttpRequest,
    error_handler: &ErrorHandler,
) -> ServerResult<HttpResponse> {
    match request.path.as_str() {
        "/api/data" => {
            let response_body = format!(
                r#"{{"received": true, "body": "{}"}}"#,
                request.body
            );
            Ok(HttpResponse::created(response_body).json())
        }
        _ => {
            Err(ServerError::NotFound(format!(
                "Endpoint not found: {}",
                request.path
            )))
        }
    }
}

fn handle_put_request(
    request: &http::HttpRequest,
    _error_handler: &ErrorHandler,
) -> ServerResult<HttpResponse> {
    match request.path.as_str() {
        "/api/data" => {
            let response_body = format!(
                r#"{{"updated": true, "body": "{}"}}"#,
                request.body
            );
            Ok(HttpResponse::ok(response_body).json())
        }
        _ => {
            Err(ServerError::NotFound(format!(
                "Endpoint not found: {}",
                request.path
            )))
        }
    }
}

fn handle_delete_request(
    request: &http::HttpRequest,
    _error_handler: &ErrorHandler,
) -> ServerResult<HttpResponse> {
    match request.path.as_str() {
        "/api/data" => {
            Ok(HttpResponse::ok(r#"{"deleted": true}"#).json())
        }
        _ => {
            Err(ServerError::NotFound(format!(
                "Endpoint not found: {}",
                request.path
            )))
        }
    }
}

fn handle_head_request(
    request: &http::HttpRequest,
    static_handler: &StaticFileHandler,
    error_handler: &ErrorHandler,
) -> ServerResult<HttpResponse> {
    let mut response = handle_get_request(request, static_handler, error_handler)?;
    
    response.body.clear();
    
    response.set_content_length();
    
    Ok(response)
}
