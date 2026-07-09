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

/// Global shutdown flag
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

fn main() -> ServerResult<()> {
    setup_signal_handler()?;

    // Load configuration
    let config = match server::config::load_from_env_file() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ Configuration error: {}", e);
            eprintln!("Using default configuration...");
            ServerConfig::new()
        }
    };

    // Create public directory if it doesn't exist
    if !config.public_dir.exists() {
        println!("📁 Creating public directory: {}", config.public_dir.display());
        std::fs::create_dir_all(&config.public_dir)
            .with_context(|| "Failed to create public directory".to_string())?;
    }

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("❌ Invalid configuration: {}", e);
        std::process::exit(1);
    }

    // Print configuration
    config.print();
    println!();

    // Create thread pool
    let pool = ThreadPool::new(config.thread_count);
    println!("🚀 Server starting on {}", config.bind_address());
    println!("📁 Serving static files from {}", config.public_dir.display());
    println!("🧵 Using {} worker threads", pool.worker_count());
    println!("Press Ctrl+C to stop\n");

    // Bind to address
    let listener = TcpListener::bind(config.bind_address())
        .with_context(|| format!("Failed to bind to {}", config.bind_address()))?;
    
    // Set socket options
    listener.set_nonblocking(false)?;

    // Initialize handlers
    let static_handler = StaticFileHandler::new(config.public_dir);
    let static_handler = Arc::new(static_handler);
    let error_handler = Arc::new(ErrorHandler::new());

    // Main accept loop
    for stream_result in listener.incoming() {
        // Check for shutdown signal
        if SHUTDOWN.load(Ordering::SeqCst) {
            println!("\n🛑 Received shutdown signal, stopping accept loop...");
            break;
        }

        match stream_result {
            Ok(stream) => {
                let handler = Arc::clone(&static_handler);
                let error_handler = Arc::clone(&error_handler);
                
                // Execute the client handling in the thread pool
                if let Err(e) = pool.execute(move || {
                    if let Err(e) = handle_client(stream, handler, error_handler) {
                        eprintln!("❌ Error handling connection: {}", e);
                    }
                }) {
                    eprintln!("❌ Failed to execute job: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("❌ Connection failed: {}", e);
            }
        }
    }

    // Graceful shutdown
    println!("\n🛑 Initiating graceful shutdown...");
    
    // Give workers time to finish current requests
    println!("⏳ Waiting for workers to finish...");
    std::thread::sleep(Duration::from_secs(1));
    
    // Shutdown thread pool
    if let Err(e) = pool.shutdown() {
        eprintln!("❌ Error during thread pool shutdown: {}", e);
    }

    println!("✅ Server shutdown complete");
    Ok(())
}

/// Sets up the Ctrl+C signal handler
fn setup_signal_handler() -> ServerResult<()> {
    ctrlc::set_handler(|| {
        println!("\n⚠️  Ctrl+C received!");
        println!("🛑 Shutting down gracefully...");
        SHUTDOWN.store(true, Ordering::SeqCst);
    }).map_err(|e| ServerError::Internal(format!("Failed to set signal handler: {}", e)))?;
    
    Ok(())
}

/// Handles a client connection with proper error handling
fn handle_client(
    mut stream: std::net::TcpStream,
    static_handler: Arc<StaticFileHandler>,
    error_handler: Arc<ErrorHandler>,
) -> ServerResult<()> {
    // Set timeouts to prevent hanging connections
    use std::time::Duration;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
    
    // Parse the request with detailed error handling
    let request = match http::request::parse_request(&mut stream) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("❌ Failed to parse request: {}", e);
            let response = error_handler.bad_request("Invalid HTTP request");
            response.send(&mut stream)?;
            return Ok(());
        }
    };
    
    // Route the request with error handling
    let response = match route_request(&request, &static_handler, &error_handler) {
        Ok(response) => response,
        Err(e) => {
            eprintln!("❌ Error processing request: {}", e);
            error_handler.handle_error(e)
        }
    };

    // Send the response
    response.send(&mut stream)?;
    
    Ok(())
}

/// Routes the request to the appropriate handler
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

/// Handles GET requests
fn handle_get_request(
    request: &http::HttpRequest,
    static_handler: &StaticFileHandler,
    _error_handler: &ErrorHandler,
) -> ServerResult<HttpResponse> {
    // Try to serve static file first
    if request.path.starts_with("/static/") || request.path == "/" {
        let path = if request.path == "/" {
            "/index.html"
        } else {
            request.path.trim_start_matches("/static/")
        };
        
        // Handle static file - convert to ServerError
        match static_handler.serve_file(path) {
            Ok(response) => Ok(response),
            Err(e) => Err(ServerError::from(e)),
        }
    } else {
        // API routes
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
        .error-test { background: #fef3c7; padding: 15px; border-radius: 5px; border-left: 4px solid #f59e0b; }
    </style>
</head>
<body>
    <h1>🚀 My HTTP Server</h1>
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
        <li><a href="/nonexistent">/nonexistent</a> - 404 error test</li>
        <li><a href="/favicon.ico">/favicon.ico</a> - 404 error test</li>
    </ul>
    <div class="error-test">
        <strong>🧪 Test Error Handling:</strong>
        <p>Missing files now return <strong>404 Not Found</strong> instead of 500!</p>
    </div>
</body>
</html>"#;
                Ok(HttpResponse::ok(html).html())
            }
            "/health" => {
                Ok(HttpResponse::ok(r#"{"status": "ok", "threads": 4}"#)
                    .json())
            }
            _ => {
                // Try static file as fallback
                match static_handler.serve_file(&request.path) {
                    Ok(response) => Ok(response),
                    Err(e) => Err(ServerError::from(e)),
                }
            }
        }
    }
}

/// Handles POST requests
fn handle_post_request(
    request: &http::HttpRequest,
    _error_handler: &ErrorHandler,
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

/// Handles PUT requests
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

/// Handles DELETE requests
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

/// Handles HEAD requests (same as GET but without body)
fn handle_head_request(
    request: &http::HttpRequest,
    static_handler: &StaticFileHandler,
    error_handler: &ErrorHandler,
) -> ServerResult<HttpResponse> {
    let mut response = handle_get_request(request, static_handler, error_handler)?;
    
    // Remove body for HEAD requests
    response.body.clear();
    
    // Update content length
    response.set_content_length();
    
    Ok(response)
}