use std::{io::{Read, ErrorKind}, net::{TcpStream, Shutdown}, str::FromStr, time::{Duration, Instant}, thread, sync::mpsc};

use log::error;

use crate::http::{request::Request, response::response_status_code::ResponseStatusCode};

pub fn parser<const BUFFER_SIZE: usize, const MAX_SIZE: usize, const PACKET_TIMEOUT: u128, const READ_TIMEOUT: u64>(
    stream: &mut TcpStream,
) -> Result<Request, ResponseStatusCode> {
    let mut request_str = String::new();

    let mut request_size = 0;

    if let Err(err) = stream.set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT))) {
        error!("Failed to set read timeout: {err}");
        return Err(ResponseStatusCode::BadRequest)
    };//max read time

    let mut stream_tmp = stream.try_clone().unwrap();

    let (tx, rx) = mpsc::channel();
    let read_thread = thread::spawn(move || {
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        loop {
            match stream_tmp.read(&mut buffer) {
                Ok(0) => {
                    return Ok(());
                },
                Ok(read_size) => {
                    tx.send((read_size, buffer)).unwrap();
                },
                Err(err) => {
                    error!("{err}");
                    return Err(err)
                },
            };
        }
    });

    let mut initial_time = Instant::now();
    loop {
        let elapsed_time = initial_time.elapsed();

        if elapsed_time.as_millis() > PACKET_TIMEOUT {
            break;
            //error!("stream packet timeout: {} > {PACKET_TIMEOUT}",elapsed_time.as_millis());
            //return Err(ResponseStatusCode::RequestTimeout);
        }

        if read_thread.is_finished() {
            match read_thread.join() {
                Ok(Ok(())) => break,
                Ok(Err(ref err)) if err.kind() == ErrorKind::TimedOut => return Err(ResponseStatusCode::RequestTimeout),
                Err(_) | Ok(Err(_)) => return Err(ResponseStatusCode::BadRequest),
            }
        }

        if let Ok((read_size, buffer)) = rx.try_recv() {
            request_size += read_size;

            if MAX_SIZE < request_size {
                return Err(ResponseStatusCode::PayloadTooLarge);
            }

            let slice = match String::from_utf8(buffer[..read_size].to_vec()) {
                Ok(val) => val,
                Err(_err) => return Err(ResponseStatusCode::BadRequest),
            };

            request_str.push_str(&slice);
            initial_time = Instant::now();
        }
    }
    let _ = stream.shutdown(Shutdown::Read);
    Request::from_str(&request_str).map_err(|_| ResponseStatusCode::BadRequest)
}

pub fn single_read_parser<const MAX_SIZE: usize, const READ_TIMEOUT: u64>(stream: &mut TcpStream) -> Result<Request, ResponseStatusCode> {
    if let Err(err) = stream.set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT))) {
        error!("Failed to set read timeout: {err:#?}");
        return Err(ResponseStatusCode::BadRequest)
    };//max read time

    let mut buffer = Vec::with_capacity(MAX_SIZE);

    let request_str = match stream.read(&mut buffer) {
        Ok(read_size) => {
            String::from_utf8(buffer[..read_size].to_vec())
        },
        Err(err) => {
            error!("Failed to read: {err:#?}");
            return Err(ResponseStatusCode::BadRequest)
        },
    };

    match request_str {
        Ok(request_str) => {
            let _ = stream.shutdown(Shutdown::Read);
            match Request::from_str(&request_str) {
                Ok(request) => Ok(request),
                Err(err) => {
                    error!("Failed to convert str to request:{err:#?}\t{request_str:#?}");
                    Err(ResponseStatusCode::BadRequest)
                },
            }
        },
        Err(err) => {
            error!("failed to parse bytes into string: {err:#?}");
            return Err(ResponseStatusCode::BadRequest)
        },
    }
}