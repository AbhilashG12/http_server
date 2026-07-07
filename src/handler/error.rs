use std::io;

use crate::error::{ServerError,ServerResult};
use crate::http::{HttpResponse,HttpStatus};
use crate::handler::StaticFileHandler;

use crate::handler::static_handler::StaticFileError;
pub struct ErrorHandler;

impl ErrorHandler {
    pub fn new()->Self{
        Self
    }

    pub fn to_response(&self,error:ServerError) -> HttpResponse {
        let status = error.to_http_status();

        let is_json = false;

        if is_json {
            let body = error.to_error_json();
            HttpResponse::new_with_status(status,body).json()
        } else{
            let body = error.to_error_html();
            HttpResponse::new_with_status(status,body).html()
        }
    }

    pub fn not_found(&self, message: impl Into<String>) -> HttpResponse {
        let error = ServerError::NotFound(message.into());
        self.to_response(error)
    }
    pub fn bad_request(&self, message: impl Into<String>) -> HttpResponse {
        let error = ServerError::BadRequest(message.into());
        self.to_response(error)
    }

    pub fn internal_error(&self, message: impl Into<String>) -> HttpResponse {
        let error = ServerError::Internal(message.into());
        self.to_response(error)
    }

    pub fn permission_denied(&self, message: impl Into<String>) -> HttpResponse {
        let error = ServerError::PermissionDenied(message.into());
        self.to_response(error)
    }

    pub fn handle_error(&self, error: impl Into<ServerError>) -> HttpResponse {
        self.to_response(error.into())
    }

    pub fn handle_io_error(&self, error: io::Error) -> HttpResponse {
        match error.kind() {
            io::ErrorKind::NotFound => {
                self.not_found(error.to_string())
            }
            io::ErrorKind::PermissionDenied => {
                self.permission_denied(error.to_string())
            }
            io::ErrorKind::InvalidInput => {
                self.bad_request(error.to_string())
            }
            _ => {
                self.internal_error(error.to_string())
            }
        }
    }

    pub fn error_page(title: &str, message: &str, code: u16) -> HttpResponse {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{} - Error</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: #f7fafc;
        }}
        .error-box {{
            background: white;
            padding: 50px;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0,0,0,0.1);
            text-align: center;
            max-width: 500px;
        }}
        h1 {{
            color: #e53e3e;
            font-size: 3em;
            margin: 0;
        }}
        .code {{
            font-size: 4em;
            font-weight: bold;
            color: #a0aec0;
        }}
        p {{
            color: #4a5568;
            line-height: 1.6;
        }}
        a {{
            color: #667eea;
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
    </style>
</head>
<body>
    <div class="error-box">
        <div class="code">{}</div>
        <h1>{}</h1>
        <p>{}</p>
        <p><a href="/">🏠 Return to Home</a></p>
    </div>
</body>
</html>"#,
            title, code, title, message
        );
        
        let status = HttpStatus::from_code(code)
            .unwrap_or(HttpStatus::InternalServerError);
        
        HttpResponse::new_with_status(status, html).html()
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn handle_static_error(error:StaticFileError) -> HttpResponse {
    let handler = ErrorHandler::new();
    handler.handle_error(error)
}

