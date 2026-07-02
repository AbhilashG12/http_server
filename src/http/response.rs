use std::io::{self,Write};
use std::net::TcpStream;

pub struct HttpResponse;

impl HttpResponse {
    pub fn send_hello(stream:&mut TcpStream) -> io::Result<()> {
        let body = "Hello world";
        let content_len = body.len();

        let res = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\
            \r\n\
            {}",
            content_len,body
        );
        // writing the above respone to stream
        stream.write_all(res.as_bytes())?;
        //all bytes are immediately pushed to client
        stream.flush();
        println!("Sent Hello world response successfully");
        Ok(())
    }
}
