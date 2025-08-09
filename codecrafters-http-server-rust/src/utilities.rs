use std::{collections::HashMap, env, io::{Read}, net::TcpStream};

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

pub fn create_http_response(http_code: &str, response_body: &str, content_type: Option<&str>, close_connection: bool) -> String {

    let content_type = content_type.unwrap_or("text/plain");

    let response_base: &str = match http_code {
        "200" => SUCCESS_RESPONSE_200,
        "201" => SUCCESS_RESPONSE_201,
        "400" => ERROR_RESPONSE_400,
        "404" => ERROR_RESPONSE_404,
        "500" => ERROR_RESPONSE_500,
        _ => ERROR_RESPONSE_500
    };

    //TODO: construct response headers with a loop

    let response: String = match close_connection {
        false => format!("{base}Content-Type: {cont_type}\r\nContent-Length: {length}\r\n\r\n{body}", 
                        base=response_base, cont_type=content_type, length= response_body.len(), body=response_body ),
        true => format!("{base}Content-Type: {cont_type}\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n{body}", 
                        base=response_base, cont_type=content_type, length= response_body.len(), body=response_body )
    };
    
    
    return response;
}