use crate::{utilities::{get_directory, get_request, populate_headers_dictionary, create_http_response}};
use std::{collections::HashMap, fs::{self, File}, io::{Write}, net::TcpStream, path::{Path, PathBuf}};


struct request_state {
    close_connection: bool,
    accept_encoding: String
}

fn handle_echo(request_target: &str, mut stream: &TcpStream, request_context: &request_state) -> () {
    let parts: Vec<&str> = request_target.split('/').collect();
    if let Some(content) = parts.last() {
        let response = create_http_response("200", &content, None, request_context.close_connection);
        stream.write_all(response.as_bytes()).unwrap();
    }
    else {
        let response: String = create_http_response("400", "", None, request_context.close_connection);
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn handle_user_agent(mut stream: &TcpStream, headers: &HashMap<String,String>, request_context: &request_state) -> () {
    if let Some(header_value)  = headers.get("user-agent") {
        let response: String = create_http_response("200", header_value, None, request_context.close_connection);
        stream.write_all(response.as_bytes()).unwrap();
    }
    else {
        let response: String = create_http_response("400", "", None, request_context.close_connection);
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn handle_base_request(mut stream: &TcpStream, request_context: &request_state) -> () {
    let response: String = create_http_response("200", "", None, request_context.close_connection);
    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_file_request(request_target: &str, mut stream: &TcpStream, method: &str, 
                            headers: &HashMap<String,String>, request_body: &str, request_context: &request_state) -> () {
    let directory: String = get_directory();
    let full_path: Vec<&str> = request_target.split('/').collect();

    // TODO: better way to verify format
    if full_path.len() != 3 {
        let response: String = create_http_response("400", "", None, request_context.close_connection);
        stream.write_all(response.as_bytes()).unwrap();
        return;
    }

    let file_name = full_path[2];
    let path = Path::new(&directory).join(file_name);
                        
    if method == "GET" {
        handle_file_request_get(&path, &stream, &headers, request_context);
    }
    else if method == "POST" {
        handle_file_request_post(&path, &stream, request_body, request_context);
    }
}

fn handle_file_request_get(path: &PathBuf, mut stream: &TcpStream, headers: &HashMap<String,String>, request_context: &request_state) -> () {
    if !path.exists() {
        let response: String = create_http_response("404", "", None, request_context.close_connection);
        stream.write_all(response.as_bytes()).unwrap();
    }
    else {
        let default: String = "application/octet-stream".to_string();
        let content: String = fs::read_to_string(path).unwrap();
        let content_type: &String = headers.get("content-type").unwrap_or(&default);

        let response: String = create_http_response("200", &content, Some(content_type), request_context.close_connection);
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn handle_file_request_post(path: &PathBuf, mut stream: &TcpStream, request_body: &str, request_context: &request_state) -> () {
    let mut file = File::create(path).unwrap();
    let updated_file: Result<(), std::io::Error> = file.write_all(request_body.as_bytes());
    if updated_file.is_err() {
        let response: String = create_http_response("500", "", None, request_context.close_connection);
        stream.write_all(response.as_bytes()).unwrap();
    }
    else {
        let response: String = create_http_response("201", "", None, request_context.close_connection);
        stream.write_all(response.as_bytes()).unwrap();
    }
}

pub fn handle_request(mut stream: TcpStream) {

    loop {
        let request = get_request(&stream);
        
        match request {
            Ok(None) => {
                //client disconnect
                break;
            }
            Ok(Some(request)) => {
                let request_string: String = request.to_string();

                let request_portions : [&str; 2] = request_string.split("\r\n\r\n").collect::<Vec<&str>>().try_into().unwrap();
                let request_body: &str = request_portions.get(1).unwrap_or(&"");
                let lines: Vec<&str> = request_portions.get(0).unwrap_or(&"").lines().collect();
                let request_line = lines[0];

                //must at least have the host header to be valid
                let headers: HashMap<String, String> = populate_headers_dictionary(lines);

                let request_context = request_state {
                    close_connection: headers.get("connection").cloned().unwrap_or_else(||"".to_string()).eq("close"),
                    accept_encoding : headers.get("accept-encoding").cloned().unwrap_or_else(||"".to_string()),
                };
                    
                let parts: Vec<&str> = request_line.split_whitespace().collect();

                //following 3 elements must be included in a valid http reques
                let Ok([method, request_target, _http_version]) : Result<[&str; 3], _> = parts.as_slice().try_into() else {
                    let response: String = create_http_response("400", "", None, request_context.close_connection);
                    stream.write_all(response.as_bytes()).unwrap();
                    break;
                };

                if request_target == "/" {
                    handle_base_request(&stream, &request_context);
                } 
                else if request_target.to_lowercase().starts_with("/echo/") {
                    handle_echo(request_target, &stream, &request_context);
                }
                else if request_target == "/user-agent" {
                    handle_user_agent(&stream, &headers, &request_context);
                }
                else if request_target.starts_with("/files/") {

                    handle_file_request(request_target, &stream, method, &headers, request_body, &request_context);
                }
                else {
                    let response: String = create_http_response("404", "", None, request_context.close_connection);
                    stream.write_all(response.as_bytes()).unwrap();
                }

                if request_context.close_connection {
                    break;
                }
            }
            Err(_e) => {
                let response: String = create_http_response("400", "", None, true);
                stream.write_all(response.as_bytes()).unwrap();
                break;
            }
        }
    }

}