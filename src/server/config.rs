use std::env;
use std::path::PathBuf;

#[derive(Debug,Clone)]
pub struct ServerConfig{
    pub port : u16,
    pub thread_count : usize,
    pub public_dir : PathBuf,
    pub max_request_size : usize,
    pub read_timeout : u64,
    pub write_timeout : u64,
}

impl ServerConfig{
    pub fn new()->Self{
        Self{
            port : 8080,
            thread_count : num_cpus::get(),
            public_dir : PathBuf::from("./public"),
            max_request_size : 1_048_576,
            read_timeout : 30,
            write_timeout : 30
        }
    }

    pub fn from_env()->Self{
        let mut config = Self::new();

        if let Ok(port) = env::var("SENDER_PORT") {
            if let Ok(port) = port.parse() {
                config.port = port;
            }
        }

        if let Ok(threads) = env::var("SERVER_THREADS") {
            if let Ok(threads) = threads.parse() {
                if threads > 0 {
                    config.thread_count = threads;
                }
            }
        }

        if let Ok(dir) = env::var("SERVER_PUBLIC_DIR") {
            config.public_dir = PathBuf::from(dir);
        }

        if let Ok(size) = env::var("SERVER_MAX_REQUEST_SIZE") {
            if let Ok(size) = size.parse() {
                config.max_request_size = size;
            }
        }

        if let Ok(timeout) = env::var("SERVER_READ_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                config.read_timeout = timeout;
            }
        }

        if let Ok(timeout) = env::var("SERVER_WRITE_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                config.write_timeout = timeout;
            }
        }

        config
    }

    pub fn with_port(mut self,port:u16)->Self{
        self.port = port;
        self
    }

    pub fn with_threads(mut self,count:usize)->Self{
        if count > 0 {
            self.thread_count = count;
        }
        self
    }

    pub fn with_pub_dir(mut self,dir:impl Into<PathBuf>)->Self{
        self.public_dir = dir.into();
        self
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.port == 0 || self.port > 65535 {
            return Err(ConfigError::InvalidPort(self.port));
        }

        if self.thread_count == 0 {
            return Err(ConfigError::InvalidThreadCount(self.thread_count));
        }

        if !self.public_dir.exists() {
            return Err(ConfigError::DirectoryNotFound(self.public_dir.clone()));
        }

        if !self.public_dir.is_dir() {
            return Err(ConfigError::NotADirectory(self.public_dir.clone()));
        }

        if self.max_request_size == 0 || self.max_request_size > 100_000_000 {
            return Err(ConfigError::InvalidMaxRequestSize(self.max_request_size));
        }

        Ok(())
    }
    
     pub fn bind_address(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

     pub fn print(&self) {
        println!("📋 Server Configuration:");
        println!("  Port: {}", self.port);
        println!("  Threads: {}", self.thread_count);
        println!("  Public Directory: {}", self.public_dir.display());
        println!("  Max Request Size: {} bytes", self.max_request_size);
        println!("  Read Timeout: {}s", self.read_timeout);
        println!("  Write Timeout: {}s", self.write_timeout);
    }

}

impl Default for ServerConfig {
    fn default()->Self{
        Self::new()
    }
}


#[derive(Debug,Clone,PartialEq)]
pub enum ConfigError{
    InvalidPort(u16),
    InvalidThreadCount(usize),
    DirectoryNotFound(PathBuf),
    NotADirectory(PathBuf),
    InvalidMaxRequestSize(usize),
}

impl std::fmt::Display for ConfigError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidPort(port) => write!(f, "Invalid port: {}", port),
            ConfigError::InvalidThreadCount(count) => write!(f, "Invalid thread count: {}", count),
            ConfigError::DirectoryNotFound(path) => write!(f, "Directory not found: {}", path.display()),
            ConfigError::NotADirectory(path) => write!(f, "Not a directory: {}", path.display()),
            ConfigError::InvalidMaxRequestSize(size) => write!(f, "Invalid max request size: {}", size),
        }
    }
}

impl std::error::Error for ConfigError {}


pub fn load_from_env_file() -> Result<ServerConfig, ConfigError> {
    if let Ok(env_content) = std::fs::read_to_string(".env") {
        for line in env_content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                unsafe {

                std::env::set_var(key, value);
                }
            }
        }
    }

    let config = ServerConfig::from_env();
    config.validate()?;
    Ok(config)
}
