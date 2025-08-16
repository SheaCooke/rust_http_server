use crate::{utilities::{get_directory, get_request, populate_headers_dictionary, return_http_response}};
use std::{collections::HashMap, fs::{self, File}, io::{Write}, net::TcpStream, path::{Path, PathBuf}};
use crate::{types::{RequestState}};


fn handle_echo(request_target: &str, stream: &TcpStream, request_context: &RequestState) -> () {
    let parts: Vec<&str> = request_target.split('/').collect();
    if let Some(content) = parts.last() {
        return_http_response("200", &content, None, request_context, &stream);
    }
    else {
        return_http_response("400", "", None, request_context, &stream);
    }
}

fn handle_user_agent(stream: &TcpStream, headers: &HashMap<String,String>, request_context: &RequestState) -> () {
    if let Some(header_value)  = headers.get("user-agent") {
        return_http_response("200", header_value, None, request_context, &stream);
    }
    else {
        return_http_response("400", "", None, request_context, &stream);
    }
}

fn handle_base_request(stream: &TcpStream, request_context: &RequestState) -> () {
    return_http_response("200", "", None, request_context, &stream);
}

fn handle_file_request(request_target: &str, stream: &TcpStream, method: &str, 
                            headers: &HashMap<String,String>, request_body: &str, request_context: &RequestState) -> () {
    let directory: String = get_directory();
    let full_path: Vec<&str> = request_target.split('/').collect();

    // TODO: better way to verify format
    if full_path.len() != 3 {
        return_http_response("400", "", None, request_context, &stream);
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

fn handle_file_request_get(path: &PathBuf, stream: &TcpStream, headers: &HashMap<String,String>, request_context: &RequestState) -> () {
    if !path.exists() {
        return_http_response("404", "", None, request_context, &stream);
    }
    else {
        let default: String = "application/octet-stream".to_string();
        let content: String = fs::read_to_string(path).unwrap();
        let content_type: &String = headers.get("content-type").unwrap_or(&default);

        return_http_response("200", &content, Some(content_type), request_context, &stream);
    }
}

fn handle_file_request_post(path: &PathBuf, stream: &TcpStream, request_body: &str, request_context: &RequestState) -> () {
    let mut file = File::create(path).unwrap();
    let updated_file: Result<(), std::io::Error> = file.write_all(request_body.as_bytes());
    if updated_file.is_err() {
        return_http_response("500", "", None, request_context, &stream);
    }
    else {
        return_http_response("201", "", None, request_context, &stream);
    }
}

pub fn handle_request(stream: TcpStream) {

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

                let request_context = RequestState {
                    close_connection: headers.get("connection").cloned().unwrap_or_else(||"".to_string()).eq("close"),
                    accept_encoding: headers.get("accept-encoding").cloned().unwrap_or_else(||"".to_string()),
                };
                    
                let parts: Vec<&str> = request_line.split_whitespace().collect();

                //following 3 elements must be included in a valid http reques
                let Ok([method, request_target, _http_version]) : Result<[&str; 3], _> = parts.as_slice().try_into() else {
                    return_http_response("400", "", None, &request_context, &stream);
                    //stream.write_all(response.as_bytes()).unwrap();
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
                    return_http_response("404", "", None, &request_context, &stream);
                }

                if request_context.close_connection {
                    break;
                }
            }
            Err(_e) => {
                let default_state = RequestState::default();
                return_http_response("400", "", None, &default_state, &stream);
                break;
            }
        }
    }

}