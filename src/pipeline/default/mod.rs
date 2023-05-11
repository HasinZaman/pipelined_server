use crate::{
    http::{body::Body, request::Request, response::Response},
    setting::ServerSetting,
};

use super::pipeline::Bytes;

pub mod action;
pub mod parser;

#[cfg(test)]
mod tests;

pub use action::default_action as action;
use flate2::{
    write::{DeflateEncoder, GzEncoder, ZlibEncoder},
    Compression,
};
use log::{trace, error};
use std::io::Write;

pub fn no_compression(response: Response, _: Option<Request>, _: ServerSetting) -> Bytes {
    return response.as_bytes();
}

pub fn compression(mut response: Response, request: Option<Request>, _setting: ServerSetting) -> Bytes {
    
    let request = match request{
        Some(val) => val,
        None => return response.as_bytes(),
    };

    if let None = response.body {
        return response.as_bytes();
    }

    let request_header = &request.1;

    if !request_header.contains_key("accept-encoding") {
        return response.as_bytes();
    }

    let mut body_content = response.body.clone().unwrap().content;

    let accepted = request_header
        .get("accept-encoding")
        .unwrap()
        .to_string()
        .replace(' ', "");

    trace!("accepted encoder:{:?}", accepted);

    let mut iter = accepted.split(',').peekable();

    if let None = &iter.peek() {
        return response.as_bytes();
    }

    for decoder in iter {
        match decoder {
            "gzip" => {
                println!("d={:?}", body_content);
                let mut encoder = GzEncoder::new(Vec::new(), Compression::none());
                if let Err(err) = encoder.write_all(&body_content) {
                    error!("{err}");

                    continue;
                }

                body_content = encoder.finish().unwrap();
                println!("d'={:?}", body_content);
                //println!("data:{body_content:?}");
                response
                    .header
                    .insert(String::from("Content-Encoding"), String::from("gzip"));
                break;
            }
            "deflate" => {
                let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
                if let Err(err) = encoder.write_all(&body_content) {
                    error!("{err}");

                    continue;
                }

                body_content = encoder.finish().unwrap();
                response
                    .header
                    .insert(String::from("Content-Encoding"), String::from("deflate"));
                break;
            }
            "zlib" => {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                if let Err(err) = encoder.write_all(&body_content) {
                    error!("{err}");

                    continue;
                }

                body_content = encoder.finish().unwrap();
                response
                    .header
                    .insert(String::from("Content-Encoding"), String::from("zlib"));
                break;
            }
            _ => {
                //no compression
            }
        }
    }

    response.body = Some(Body {
        content_type: response.body.unwrap().content_type,
        content: body_content,
    });

    return response.as_bytes();
}

