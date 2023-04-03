use crate::{
    http::{
        request::Request,
        response::{Response}, body::Body,
    }, setting::ServerSetting,
};



use super::{pipeline::Bytes};

pub mod action;
mod parser;

#[cfg(test)]
mod tests;

use flate2::{Compression, write::{GzEncoder, DeflateEncoder, ZlibEncoder}};
use std::io::Write;
use log::trace;
pub use parser::parser;
pub use action::default_action as action;

pub fn no_compression(response: Response, _: Option<Request>, _: ServerSetting) -> Bytes {
    return response.as_bytes()
}

pub fn compression(mut response: Response, request: Request, setting: ServerSetting) -> Bytes {
    if let None = response.body {
        return response.as_bytes()
    }
    
    
    let request_header = &request.1;

    if !request_header.contains_key("accept-encoding") {
        return response.as_bytes()
    }
    
    let mut body_content = response.body.clone().unwrap().content;

    let accepted = request_header.get("accept-encoding").unwrap()
            .to_string()
            .replace(' ', "");

    trace!("accepted encoder:{:?}", accepted);

    for decoder in accepted.split(',') {
        match decoder {
            "gzip" => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write(&body_content);

                body_content = encoder.finish().unwrap();
                response.header.insert(String::from("Content-Encoding"), String::from("gzip"));
                break;
            },
            "deflate" => {
                let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
                encoder.write(&body_content);

                body_content = encoder.finish().unwrap();
                response.header.insert(String::from("Content-Encoding"), String::from("deflate"));
                break;
            },
            "zlib" => {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write(&body_content);

                body_content = encoder.finish().unwrap();
                response.header.insert(String::from("Content-Encoding"), String::from("zlib"));
                break;
            },
            _ => {
                //no compression
            }
        }
    }

    response.body = Some(Body{
        content_type: response.body.unwrap().content_type,
        content: body_content,
    });
    
    return response.as_bytes()
}

// impl Default for Builder<FileUtility> {
//     fn default() -> Self {
//         Self {
//             parser: parser::<1024, 262144>,
//             action: default_action,
//             compression: compression,
//         }
//     }
// }
