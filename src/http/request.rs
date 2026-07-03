use std::collections::HashMap;
use std::io::{self,Read};
use std::net::TcpStream;

#[derive(Debug,Clone)]
pub struct HttpRequest{
    pub method : String,
    pub path : String,
    pub version : String,
    pub headers : HashMap<String,String>,
    pub body : String,
    pub raw_data : Vec<u8>,
}

impl HttpRequest {
    // Creates a new empty HTTP Request
    pub fn new()->Self{
        Self{
            method : String::new(),
            path : String::new(),
            version : String::new(),
            headers : HashMap::new(),
            body : String::new(),
            raw_data : Vec::new(),
        }
    }
    // This Parses raw HTTP Req to structured HTTP Req
    pub fn from_bytes(data:&[u8]) -> Result<Self,String> {
        // Take raw bytes and convert them into string and split them according to the delimiter
        let raw_string = String::from_utf8_lossy(data);
        let lines : Vec<&str> = raw_string.split("\r\n").collect();
        
        if lines.is_empty(){
            return Err("Empty Request".to_string());
        }
        // Create a new HTTP Request 
        let mut request = HttpRequest::new();
        request.raw_data = data.to_vec();
        // first line is request line
        let request_line = lines[0];
        let parts : Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() < 3 {
            return Err(format!("Invalid request line : {}",request_line));
        }
        // method path and version are extracted from request line in their order
        request.method = parts[0].to_string();
        request.path = parts[1].to_string();
        request.version = parts[2].to_string();
    
        let mut header_lines = Vec::new();
        let mut body_start_index = 0;
        let mut found_empty_line = false;
        // Iterate over lines and push them to header_lines vectors
        for (i,line) in lines.iter().enumerate().skip(1) {
            if line.is_empty(){
                found_empty_line = true;
                body_start_index = i + 1;
                break;
            }
            header_lines.push(*line);
        }

        // for each value in head_lines vector we can map a hashmap when found ":" and insert those
        // key value in the new requests headers
        for line in header_lines {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                request.headers.insert(key,value);
            }
        }

        if found_empty_line && body_start_index < lines.len() {
            let body_lines = &lines[body_start_index..];
            request.body = body_lines.join("\r\n");
        }
        Ok(request)
    }

    // Some helping methods 

    pub fn is_post(&self) -> bool {
        self.method == "POST"
    }

    pub fn is_get(&self) -> bool {
        self.method == "GET"
    }
    pub fn get_header(&self,key:&str) -> Option<&String> {
        self.headers.get(key)
    }
    pub fn content_length(&self) -> Option<usize> {
        self.get_header("Content-Length").and_then(|v| v.parse::<usize>().ok())
    }

    // logs the parsed requests for debugging
    
    pub fn logs(&self){
        println!("\n--- [PARSED HTTP REQUEST] ---");
        println!("Method : {}",self.method);
        println!("Path : {}",self.path);
        println!("Version : {}",self.version);
        println!("Headers : ");
        for(k,v) in &self.headers {
            println!(" {} : {} ",k,v);
        }

        if !self.body.is_empty(){
            println!("Body : {}",self.body);
        }
        println!("--------------------------------");

    }
}

// Reads and parses http request from TCP Stream

    pub fn parse_request(mut stream: &mut TcpStream) -> Result<HttpRequest,io::Error> {
        let mut buffer = [0;4096];
        let mut raw_data = Vec::new();

       loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        raw_data.extend_from_slice(&buffer[..bytes_read]);

        if has_reached_end_of_headers(&raw_data) {
            break;
        }
    }
        match HttpRequest::from_bytes(&raw_data) {
            Ok(request) => {
                request.logs();
                Ok(request)
            }
            Err(e) =>{
                eprintln!("Failed to parse request : {}",e);
                Err(io::Error::new(io::ErrorKind::InvalidData,e))
            }
        }
    }

fn has_reached_end_of_headers(bytes: &[u8]) -> bool {
    bytes.windows(4).any(|window| window == b"\r\n\r\n")
}
