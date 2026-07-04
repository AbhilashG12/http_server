use std::collections::HashMap;
use std::path::Path;

#[derive(Debug,Clone)]
pub struct MimeTypes {
    mappings : HashMap<String,String>
}

impl MimeTypes{

    pub fn new() ->Self{
        let mut mappings = HashMap::new();
        // Text types
        mappings.insert("html".to_string(),"text/html".to_string());
        mappings.insert("htm".to_string(), "text/html".to_string());
        mappings.insert("css".to_string(), "text/css".to_string());
        mappings.insert("js".to_string(), "application/javascript".to_string());
        mappings.insert("json".to_string(), "application/json".to_string());
        mappings.insert("txt".to_string(), "text/plain".to_string());
        mappings.insert("xml".to_string(), "application/xml".to_string());
        mappings.insert("csv".to_string(), "text/csv".to_string());
        
        // Image types
        mappings.insert("png".to_string(), "image/png".to_string());
        mappings.insert("jpg".to_string(), "image/jpeg".to_string());
        mappings.insert("jpeg".to_string(), "image/jpeg".to_string());
        mappings.insert("gif".to_string(), "image/gif".to_string());
        mappings.insert("svg".to_string(), "image/svg+xml".to_string());
        mappings.insert("webp".to_string(), "image/webp".to_string());
        mappings.insert("ico".to_string(), "image/x-icon".to_string());
        
        // Font types
        mappings.insert("woff".to_string(), "font/woff".to_string());
        mappings.insert("woff2".to_string(), "font/woff2".to_string());
        mappings.insert("ttf".to_string(), "font/ttf".to_string());
        mappings.insert("otf".to_string(), "font/otf".to_string());
        
        // Video types
        mappings.insert("mp4".to_string(), "video/mp4".to_string());
        mappings.insert("webm".to_string(), "video/webm".to_string());
        mappings.insert("ogv".to_string(), "video/ogg".to_string());
        
        // Audio types
        mappings.insert("mp3".to_string(), "audio/mpeg".to_string());
        mappings.insert("wav".to_string(), "audio/wav".to_string());
        mappings.insert("ogg".to_string(), "audio/ogg".to_string());
        
        // Document types
        mappings.insert("pdf".to_string(), "application/pdf".to_string());
        mappings.insert("doc".to_string(), "application/msword".to_string());
        mappings.insert("docx".to_string(), "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string());
        mappings.insert("xls".to_string(), "application/vnd.ms-excel".to_string());
        mappings.insert("xlsx".to_string(), "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string());
        
        // Compressed types
        mappings.insert("zip".to_string(), "application/zip".to_string());
        mappings.insert("gz".to_string(), "application/gzip".to_string());
        mappings.insert("tar".to_string(), "application/x-tar".to_string());
        
        Self { mappings }
    }
    // get a mime type for file extension
    pub fn get_mime_type(&self,extension:&str)->String{
        let ext = extension.to_lowercase();
        self.mappings
            .get(&ext)
            .cloned()
            .unwrap_or_else(|| "application/octet-stream".to_string())
    }
    // get a mime type from file path
    pub fn get_mime_type_from_path(&self,path:&Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.get_mime_type(ext))
            .unwrap_or_else(|| "application/octet-stream".to_string())
    }
    // Adds or updates a MIME type mapping
    pub fn add_mapping(&mut self, extension: impl Into<String>, mime_type: impl Into<String>) {
        self.mappings.insert(extension.into(), mime_type.into());
    }

    // Removes a MIME type mapping
    pub fn remove_mapping(&mut self, extension: &str) -> Option<String> {
        self.mappings.remove(extension)
    }

    // Checks if an extension is supported
    pub fn supports_extension(&self, extension: &str) -> bool {
        self.mappings.contains_key(extension)
    }

    // Gets all supported extensions
    pub fn supported_extensions(&self) -> Vec<&String> {
        self.mappings.keys().collect()
    }

}

impl Default for MimeTypes {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience function to get MIME type from a path
pub fn get_mime_type(path: &Path) -> String {
    MimeTypes::new().get_mime_type_from_path(path)
}

// Convenience function to get MIME type from an extension
pub fn get_mime_type_for_ext(extension: &str) -> String {
    MimeTypes::new().get_mime_type(extension)
}

