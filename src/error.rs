use std::io;
use std::sync::mpsc;

use crate::http::HttpStatus;
use crate::handler::static_handler::StaticFileError;
use crate::server::{ConfigError, ThreadPoolError};
use crate::utils::path::PathError;

/// Central error type for the entire server
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// I/O errors from std::io
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    
    /// HTTP parsing errors
    #[error("HTTP parse error: {0}")]
    HttpParse(String),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    /// Thread pool errors
    #[error("Thread pool error: {0}")]
    ThreadPool(#[from] ThreadPoolError),
    
    /// Path sanitization errors
    #[error("Path error: {0}")]
    Path(#[from] PathError),
    
    /// Static file errors - properly mapped
    #[error("Static file error: {0}")]
    StaticFile(#[from] StaticFileError),
    
    /// Channel errors during shutdown
    #[error("Channel error: {0}")]
    Channel(String),
    
    /// Shutdown signal received
    #[error("Server shutting down")]
    Shutdown,
    
    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Bad request
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl ServerError {
    /// Converts the error to an HTTP status code
    pub fn to_http_status(&self) -> HttpStatus {
        match self {
            ServerError::NotFound(_) => HttpStatus::NotFound,
            ServerError::PermissionDenied(_) => HttpStatus::Forbidden,
            ServerError::BadRequest(_) => HttpStatus::BadRequest,
            ServerError::Shutdown => HttpStatus::ServiceUnavailable,
            ServerError::HttpParse(_) => HttpStatus::BadRequest,
            ServerError::StaticFile(e) => {
                match e {
                    StaticFileError::NotFound(_) => HttpStatus::NotFound,
                    StaticFileError::DirectoryNotAllowed(_) => HttpStatus::Forbidden,
                    StaticFileError::AccessDenied(_) => HttpStatus::Forbidden,
                    StaticFileError::PermissionDenied(_) => HttpStatus::Forbidden,
                    StaticFileError::InvalidPath(_) => HttpStatus::BadRequest,
                    StaticFileError::InternalError => HttpStatus::InternalServerError,
                }
            }
            _ => HttpStatus::InternalServerError,
        }
    }

    /// Creates an error page HTML for the error
    pub fn to_error_html(&self) -> String {
        let status = self.to_http_status();
        let code = status.code();
        let message = status.reason_phrase();
        let detail = self.to_string();

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Error</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 20px;
        }}
        .error-container {{
            background: white;
            border-radius: 20px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            max-width: 600px;
            width: 100%;
            padding: 50px;
            text-align: center;
            animation: fadeIn 0.6s ease-in;
        }}
        @keyframes fadeIn {{
            from {{ opacity: 0; transform: translateY(20px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}
        .error-code {{
            font-size: 6em;
            font-weight: bold;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            line-height: 1.2;
        }}
        .error-message {{
            font-size: 1.5em;
            color: #2d3748;
            margin: 20px 0;
        }}
        .error-detail {{
            color: #718096;
            margin: 20px 0 30px;
            padding: 15px;
            background: #f7fafc;
            border-radius: 8px;
            border-left: 4px solid #667eea;
            text-align: left;
            font-size: 0.9em;
            word-break: break-all;
        }}
        .home-link {{
            display: inline-block;
            padding: 12px 30px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            text-decoration: none;
            border-radius: 8px;
            font-weight: 600;
            transition: transform 0.2s;
        }}
        .home-link:hover {{
            transform: translateY(-2px);
            box-shadow: 0 5px 20px rgba(102, 126, 234, 0.4);
        }}
    </style>
</head>
<body>
    <div class="error-container">
        <div class="error-code">{}</div>
        <div class="error-message">{}</div>
        <div class="error-detail">💡 {}</div>
        <a href="/" class="home-link">🏠 Go Home</a>
    </div>
</body>
</html>"#,
            code, code, message, detail
        )
    }

    /// Creates a JSON error response for APIs
    pub fn to_error_json(&self) -> String {
        let status = self.to_http_status();
        format!(
            r#"{{"error": {{"code": {}, "message": "{}", "detail": "{}"}}}}"#,
            status.code(),
            status.reason_phrase(),
            self.to_string().escape_default()
        )
    }
}

/// Result type using ServerError
pub type ServerResult<T> = Result<T, ServerError>;

/// Extension trait for converting Result types to ServerResult
pub trait IntoServerResult<T> {
    fn into_server_result(self) -> ServerResult<T>;
}

impl<T> IntoServerResult<T> for Result<T, io::Error> {
    fn into_server_result(self) -> ServerResult<T> {
        self.map_err(ServerError::from)
    }
}

impl<T> IntoServerResult<T> for Result<T, String> {
    fn into_server_result(self) -> ServerResult<T> {
        self.map_err(|e| ServerError::Internal(e))
    }
}

/// Extension trait for adding context to errors
pub trait WithContext<T> {
    fn with_context<F>(self, context: F) -> ServerResult<T>
    where
        F: FnOnce() -> String;
}

impl<T> WithContext<T> for Result<T, ServerError> {
    fn with_context<F>(self, context: F) -> ServerResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            ServerError::Internal(format!("{}: {}", context(), e))
        })
    }
}

impl<T> WithContext<T> for Result<T, io::Error> {
    fn with_context<F>(self, context: F) -> ServerResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            ServerError::Io(io::Error::new(e.kind(), format!("{}: {}", context(), e)))
        })
    }
}

impl From<mpsc::RecvError> for ServerError {
    fn from(e: mpsc::RecvError) -> Self {
        ServerError::Channel(e.to_string())
    }
}

impl From<mpsc::SendError<Box<dyn FnOnce() + Send + 'static>>> for ServerError {
    fn from(e: mpsc::SendError<Box<dyn FnOnce() + Send + 'static>>) -> Self {
        ServerError::Channel(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for ServerError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        ServerError::HttpParse(format!("Invalid UTF-8: {}", e))
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(e: serde_json::Error) -> Self {
        ServerError::Internal(format!("JSON error: {}", e))
    }
}
