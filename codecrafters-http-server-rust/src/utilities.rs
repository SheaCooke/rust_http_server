use std::{collections::HashMap, env, io::{Read}, net::TcpStream};
use crate::{types::{RequestState}};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;

const SUCCESS_RESPONSE_200: &str = "HTTP/1.1 200 OK\r\n";
const SUCCESS_RESPONSE_201: &str = "HTTP/1.1 201 Created\r\n";
const ERROR_RESPONSE_400: &str = "HTTP/1.1 400 Bad Request\r\n";
const ERROR_RESPONSE_404: &str = "HTTP/1.1 404 Not Found\r\n";
const ERROR_RESPONSE_500: &str = "HTTP/1.1 500 Internal Server Error\r\n";
const SUPPORTED_ENCODINGS: [&str ; 1] = ["gzip"];

pub fn get_directory() -> String {
    let mut directory: String = "".to_string();
    let arguments: Vec<String> = env::args().collect();

    for i in 0..arguments.len() {
         if arguments[i] == "--directory" && i < arguments.len() {
             directory = arguments[i+1].clone();
        }
    }
    return directory;
}

pub fn populate_headers_dictionary(lines: Vec<&str>) -> HashMap<String,String> {
    let mut headers: HashMap<String, String> = HashMap::new();

    for line in &lines[1..] {
        let parts: Vec<&str> = line.split(':').collect();

        if parts.len() < 2 {
            continue;
        }

        //http headers are case insensitive
        headers.insert(parts[0].to_lowercase().trim().to_string(), parts[1].trim().to_string());
    }
                    
    return headers;
}

pub fn get_request(mut stream : &TcpStream) -> Result<Option<String>, std::io::Error> {
    let mut buffer = [0; 1024];
    let bytes = stream.read(&mut buffer)?; //blocking by default
    if bytes == 0 {
        return Ok(None);
    }
    return Ok(Some(String::from_utf8_lossy(&buffer[..bytes]).to_string()));
}

fn supported_encoding(accepted_encoding: &String) -> Option<String> {
    let encodings: Vec<&str> = accepted_encoding.split(',').collect();
    //if single encoding
    if encodings.len() == 1 && SUPPORTED_ENCODINGS.contains(encodings.get(0).unwrap()) {
        return Some(encodings.get(0).unwrap().to_string());
    }
    else {
        for encoding in encodings {
            
            if SUPPORTED_ENCODINGS.contains(&encoding.trim()) {
                return Some(encoding.trim().to_string());
            }
        }
    }
    return None;
}

fn compress_string(input: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input.as_bytes()).unwrap();
    return encoder.finish().unwrap();
}

pub fn return_http_response(http_code: &str, response_body: &str, content_type: Option<&str>, 
                                                            request_context: &RequestState, mut stream: &TcpStream) -> () {

    let mut encoded_body: bool = false;

    let content_type = content_type.unwrap_or("text/plain");

    let response_base: &str = match http_code {
        "200" => SUCCESS_RESPONSE_200,
        "201" => SUCCESS_RESPONSE_201,
        "400" => ERROR_RESPONSE_400,
        "404" => ERROR_RESPONSE_404,
        "500" => ERROR_RESPONSE_500,
        _ => ERROR_RESPONSE_500
    };

    let mut additiona_headders: String = String::new();
    
    if request_context.close_connection {
        additiona_headders.push_str("\r\nConnection: close");
    }

    if let Some(usable_encoding) = supported_encoding(&request_context.accept_encoding) {
        let content_encoding = format!("\r\nContent-Encoding: {}", usable_encoding);
        additiona_headders.push_str(content_encoding.as_str());
        encoded_body = true;
    }

    //mark end of headers, add response body
    additiona_headders.push_str("\r\n\r\n");

    if encoded_body {
        let compressed_body: Vec<u8> = compress_string(response_body);
        let base_headders: String = format!("{base}Content-Type: {cont_type}\r\nContent-Length: {length}", 
                                base=response_base, cont_type=content_type, length=compressed_body.len());
        let http_response: String = format!("{base}{additional}", base=base_headders, additional=additiona_headders);

        stream.write(http_response.as_bytes()).unwrap();
        stream.write(&compressed_body).unwrap();
    }
    else {
        let base_headders: String = format!("{base}Content-Type: {cont_type}\r\nContent-Length: {length}", 
                                 base=response_base, cont_type=content_type, length=response_body.len());
                                 
        let http_response: String = format!("{base}{additional}{body}", base=base_headders, 
                                                        additional=additiona_headders,body=response_body);
        stream.write(http_response.as_bytes()).unwrap();
    }
}