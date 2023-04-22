use std::{io::Read, net::TcpStream, str::FromStr};

use crate::http::{request::Request, response::response_status_code::ResponseStatusCode};

pub fn parser<const BUFFER_SIZE: usize, const MAX_SIZE: usize>(
    stream: &mut TcpStream,
) -> Result<Request, ResponseStatusCode> {
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    let mut request_str = String::new();

    let mut request_size = 0;

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                break;
            }
            Ok(read_size) => {
                request_size += read_size;

                if MAX_SIZE < request_size {
                    return Err(ResponseStatusCode::PayloadTooLarge);
                }

                let slice = match String::from_utf8(buffer[..read_size].to_vec()) {
                    Ok(val) => val,
                    Err(_err) => return Err(ResponseStatusCode::BadRequest),
                };

                request_str.push_str(&slice);
            }
            Err(_err) => return Err(ResponseStatusCode::BadRequest),
        }
    }

    Request::from_str(&request_str).map_err(|_| ResponseStatusCode::BadRequest)
}
