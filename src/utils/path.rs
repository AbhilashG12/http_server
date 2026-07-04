use std::path::{Path,PathBuf};
use std::fs;

#[derive(Clone)]
pub struct PathSanitizer {
    root : PathBuf,
}

impl PathSanitizer {
    // This creates a new PathSanitizer with the given root directory 
    pub fn new(root:impl Into<PathBuf>) -> Self {
        Self {
            root : root.into()
        }
    }

    pub fn sanitizer(&self,url_path:&str) -> Result<PathBuf,PathError> {
        // We decode the hex codes to find their original special characters
        let decoded = urlencoding::decode(url_path)
            .map_err(|_| PathError::InvalidPath(url_path.to_string()))?;
        // Split the decoded patj
        let clean_path = decoded.split('?').next().unwrap_or(&decoded).split('#').next().unwrap_or(&decoded);
        // Get the full path and join it to the original root dir
        let full_path = self.root.join(clean_path);
        // Removes . and .. and appends the url to the original root dir
        let canonicalized = full_path.canonicalize().map_err(|_| PathError::NotFound(clean_path.to_string()))?;
        // checks if the target files prefix is same as root dirs prefix
        let root_canonical = self.root.canonicalize().map_err(|_| PathError::InvalidRoot)?;

        if !canonicalized.starts_with(&root_canonical){
            return Err(PathError::OutsideRoot(clean_path.to_string()));
        }
        // If a valid path enters , then it makes more checks
        if canonicalized.is_dir(){
            let index_path = canonicalized.join("index.html");
            if index_path.exists() && index_path.is_file(){
                return Ok(index_path);
            }
            return Err(PathError::IsDirectory(clean_path.to_string()));
        }

        if !canonicalized.is_file(){
            return Err(PathError::NotFound(clean_path.to_string()));
        }

        Ok(canonicalized)
    }
    // checks if a path exists and is readable
    pub fn path_exists(&self,path:&Path) -> bool {
        path.exists() && path.is_file() && fs::metadata(path).map(|m| m.is_file()).unwrap_or(false)
    }
    // gets the file extensions from a path
    pub fn get_extension(path:&Path)->Option<String>{
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    }
    // gets the size of the file
    pub fn get_file_size(path:&Path) -> Option<u64>{
        fs::metadata(path).ok().map(|m| m.len())
    }
}


#[derive(Debug,Clone,PartialEq)]
pub enum PathError{
    InvalidPath(String),
    NotFound(String),
    IsDirectory(String),
    OutsideRoot(String),
    InvalidRoot,
    PermissionDenied(String),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            PathError::NotFound(path) => write!(f, "File not found: {}", path),
            PathError::IsDirectory(path) => write!(f, "Path is a directory: {}", path),
            PathError::OutsideRoot(path) => write!(f, "Path outside root: {}", path),
            PathError::InvalidRoot => write!(f, "Invalid root directory"),
            PathError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
        }
    }
}

impl std::error::Error for PathError {}


// Helper function to sanitize a path with a default root of "./public"
pub fn sanitize_path(url_path: &str) -> Result<PathBuf, PathError> {
    let sanitizer = PathSanitizer::new("./public");
    sanitizer.sanitizer(url_path)
}
