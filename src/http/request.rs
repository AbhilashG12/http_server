use std::io::{self,Read};
use std::net::TcpStream;
use crate::http::response::HttpResponse;
pub struct HttpRequest{
    pub raw_data : Vec<u8>
}

pub fn handle_connection(mut stream : TcpStream) -> io::Result<()> {
    
    let mut buffer = [0;512];
    let mut raw_request = Vec::new();
    
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        raw_request.extend_from_slice(&buffer[..bytes_read]);

        if has_reached_end_of_headers(&raw_request){
            break;
        }
    }

    let request = HttpRequest {raw_data : raw_request};

    let parsed_string = String::from_utf8_lossy(&request.raw_data);
    println!("\n--- [RECEIVED RAW HTTP REQUEST] ---");
    println!("{}", parsed_string);
    println!("------------------------------------\n");
    HttpResponse::send_hello(&mut stream)?;
    Ok(())

}

fn has_reached_end_of_headers(bytes:&[u8]) -> bool{
    bytes.windows(4).any(|window| window == b"\r\n\r\n")
}
