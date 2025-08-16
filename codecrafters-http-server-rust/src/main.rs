mod utilities;
mod requests;
mod types;

use std::{thread};
use std::net::TcpListener;

use crate::{requests::handle_request};

fn main() {

   let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
   
    for stream in listener.incoming() {

        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_request(stream);
                });
            }
            Err(e) => {
                println!("Failed to connect to stream: {}", e);
            }
        }
    }
}
