use std::io;
use std::net::{TcpListener};
pub mod http;

fn main() -> io::Result<()>{
    let address = String::from("127.0.0.1:7878");
    let listen = TcpListener::bind(&address);
    println!("SUCCESS TcpListener is Running on {}",&address);
    println!("Waiting for connections.......\n");

    for res in listen?.incoming(){
        match res {
            Ok(stream) =>{
                match stream.peer_addr(){
                    Ok(peer_addr) =>{
                        println!("[CONNECTED!] New Client established from IP : {} , port : {} ",peer_addr.ip(),peer_addr.port());
                    }
                    Err(e)=>{
                        println!("[WARNING!!] Connected client , but couldnt resolve peer address {}",e);
                    }
                }
                if let Err(e) = http::request::handle_connection(stream){
                    eprintln!("[ERROR] Failed to handle request");
                }
            }
            Err(e)=>{
                eprintln!("[ERROR] Failed to accept incoming connections : {}",e);
            }
        }
    }

        Ok(())

}
