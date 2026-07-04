use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use crate::http::{HttpResponse, HttpStatus};
use crate::utils::path::{PathSanitizer, PathError};
use crate::utils::mime::MimeTypes;

// Handles serving static files from the filesystem
#[derive(Clone)]
pub struct StaticFileHandler {
    root_dir: PathBuf,
    path_sanitizer: PathSanitizer,
    mime_types: MimeTypes,
}

impl StaticFileHandler {
    // Creates a new StaticFileHandler with the given root directory
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        let root_dir = root_dir.into();
        let path_sanitizer = PathSanitizer::new(root_dir.clone());
        let mime_types = MimeTypes::new();
        
        Self {
            root_dir,
            path_sanitizer,
            mime_types,
        }
    }

    // Serves a static file from the given URL path
    pub fn serve_file(&self, url_path: &str) -> Result<HttpResponse, StaticFileError> {
        // Step 1: Sanitize the path
        let fs_path = self.path_sanitizer.sanitizer(url_path)
            .map_err(|e| match e {
                PathError::NotFound(_) => StaticFileError::NotFound(url_path.to_string()),
                PathError::IsDirectory(_) => StaticFileError::DirectoryNotAllowed(url_path.to_string()),
                PathError::OutsideRoot(_) => StaticFileError::AccessDenied(url_path.to_string()),
                PathError::InvalidPath(_) => StaticFileError::InvalidPath(url_path.to_string()),
                PathError::InvalidRoot => StaticFileError::InternalError,
                PathError::PermissionDenied(_) => StaticFileError::PermissionDenied(url_path.to_string()),
            })?;

        // Step 2: Read the file
        let contents = self.read_file(&fs_path)?;
        
        // Step 3: Get file metadata
        let metadata = fs::metadata(&fs_path)
            .map_err(|_| StaticFileError::InternalError)?;
        
        // Step 4: Determine MIME type
        let mime_type = self.mime_types.get_mime_type_from_path(&fs_path);
        
        // Step 5: Build the response
        let mut response = HttpResponse::ok(contents)
            .content_type(mime_type);
        
        // Add cache control for performance
        response.headers.insert(
            "Cache-Control".to_string(),
            "public, max-age=3600".to_string() // Cache for 1 hour
        );
        
        // Add last modified header
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.elapsed() {
                let last_modified = format!("{}", duration.as_secs());
                response.headers.insert("Last-Modified".to_string(), last_modified);
            }
        }
        
        Ok(response)
    }

    // Reads a file from disk with proper error handling
    fn read_file(&self, path: &std::path::Path) -> Result<String, StaticFileError> {
        // Check if file exists and is readable
        if !path.exists() {
            return Err(StaticFileError::NotFound(path.display().to_string()));
        }
        
        if !path.is_file() {
            return Err(StaticFileError::DirectoryNotAllowed(path.display().to_string()));
        }
        
        // Read the file
        match fs::read_to_string(path) {
            Ok(contents) => Ok(contents),
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                Err(StaticFileError::PermissionDenied(path.display().to_string()))
            }
            Err(_) => Err(StaticFileError::InternalError),
        }
    }

    // Serves a file and returns a ready-to-send HttpResponse
    pub fn handle_request(&self, url_path: &str) -> HttpResponse {
        match self.serve_file(url_path) {
            Ok(response) => response,
            Err(error) => {
                eprintln!("Static file error: {}", error);
                error.to_http_response()
            }
        }
    }

    // Checks if a file exists at the given URL path
    pub fn file_exists(&self, url_path: &str) -> bool {
        self.path_sanitizer.sanitizer(url_path)
            .map(|path| path.exists() && path.is_file())
            .unwrap_or(false)
    }

    // Gets the MIME type for a URL path without serving the file
    pub fn get_mime_type(&self, url_path: &str) -> Option<String> {
        let path = self.path_sanitizer.sanitizer(url_path).ok()?;
        Some(self.mime_types.get_mime_type_from_path(&path))
    }
}

// Errors that can occur during static file handling
#[derive(Debug, Clone, PartialEq)]
pub enum StaticFileError {
    NotFound(String),
    DirectoryNotAllowed(String),
    AccessDenied(String),
    InvalidPath(String),
    PermissionDenied(String),
    InternalError,
}

impl std::fmt::Display for StaticFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StaticFileError::NotFound(path) => write!(f, "File not found: {}", path),
            StaticFileError::DirectoryNotAllowed(path) => write!(f, "Directory access not allowed: {}", path),
            StaticFileError::AccessDenied(path) => write!(f, "Access denied: {}", path),
            StaticFileError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            StaticFileError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            StaticFileError::InternalError => write!(f, "Internal server error"),
        }
    }
}

impl std::error::Error for StaticFileError {}

impl StaticFileError {
    // Converts the error to an HTTP response
    pub fn to_http_response(&self) -> HttpResponse {
        match self {
            StaticFileError::NotFound(path) => {
                let body = format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body style="font-family: Arial; padding: 40px; text-align: center;">
    <h1 style="color: #c0392b;">404 - File Not Found</h1>
    <p>The requested file <code>{}</code> was not found.</p>
    <a href="/">Go Home</a>
</body>
</html>"#,
                    path
                );
                HttpResponse::new_with_status(HttpStatus::NotFound, body).html()
            }
            StaticFileError::DirectoryNotAllowed(path) => {
                let body = format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>403 Forbidden</title></head>
<body style="font-family: Arial; padding: 40px; text-align: center;">
    <h1 style="color: #e67e22;">403 - Directory Access Not Allowed</h1>
    <p>Access to directory <code>{}</code> is not allowed.</p>
    <p>Please request a specific file instead.</p>
    <a href="/">Go Home</a>
</body>
</html>"#,
                    path
                );
                HttpResponse::new_with_status(HttpStatus::Forbidden, body).html()
            }
            StaticFileError::AccessDenied(path) => {
                let body = format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>403 Forbidden</title></head>
<body style="font-family: Arial; padding: 40px; text-align: center;">
    <h1 style="color: #e67e22;">403 - Access Denied</h1>
    <p>Access to <code>{}</code> was denied.</p>
    <a href="/">Go Home</a>
</body>
</html>"#,
                    path
                );
                HttpResponse::new_with_status(HttpStatus::Forbidden, body).html()
            }
            StaticFileError::InvalidPath(path) => {
                let body = format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>400 Bad Request</title></head>
<body style="font-family: Arial; padding: 40px; text-align: center;">
    <h1 style="color: #e67e22;">400 - Invalid Path</h1>
    <p>The path <code>{}</code> is invalid.</p>
    <a href="/">Go Home</a>
</body>
</html>"#,
                    path
                );
                HttpResponse::new_with_status(HttpStatus::BadRequest, body).html()
            }
            StaticFileError::PermissionDenied(path) => {
                let body = format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>403 Forbidden</title></head>
<body style="font-family: Arial; padding: 40px; text-align: center;">
    <h1 style="color: #e67e22;">403 - Permission Denied</h1>
    <p>Permission denied reading <code>{}</code>.</p>
    <a href="/">Go Home</a>
</body>
</html>"#,
                    path
                );
                HttpResponse::new_with_status(HttpStatus::Forbidden, body).html()
            }
            StaticFileError::InternalError => {
                let body = r#"<!DOCTYPE html>
<html>
<head><title>500 Internal Server Error</title></head>
<body style="font-family: Arial; padding: 40px; text-align: center;">
    <h1 style="color: #c0392b;">500 - Internal Server Error</h1>
    <p>An internal error occurred while serving the file.</p>
    <a href="/">Go Home</a>
</body>
</html>"#;
                HttpResponse::new_with_status(HttpStatus::InternalServerError, body).html()
            }
        }
    }
}

