//! response module is responsible for all structs, enums and methods related to formation of a HTTP response

use std::{collections::HashMap, fmt::Display, str};

use self::response_status_code::ResponseStatusCode;

use super::body::Body;
pub mod response_status_code;

#[cfg(test)]
mod tests;

/// Response struct define the Response structure
#[derive(Debug)]
pub struct Response {
    pub status: ResponseStatusCode,
    pub header: HashMap<String, String>,
    pub body: Option<Body>,
}

macro_rules! append_to {
    ($list:ident, $value:expr) => {
        for byte in $value.as_bytes() {
            $list.push(byte.clone());
        }
    };
    ($list:ident, $value:ident) => {
        for byte in $value {
            $list.append(byte.clone());
        }
    };
}

impl Response {
    /// as_bytes provides a bytes required in order send response
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();

        append_to!(output, format!("HTTP/1.1 {}\r\n", self.status.to_string()));

        self.header.keys().into_iter().for_each(|key| {
            append_to!(
                output,
                format!("{}: {}\r\n", key, &self.header.get(key).unwrap())
            );
        });

        match &self.body {
            Some(body) => {
                append_to!(
                    output,
                    format!("Content-Length: {}\r\n", body.content.len())
                );

                append_to!(
                    output,
                    format!("Content-Type: {}\r\n", body.content_type.to_string())
                );

                append_to!(output, "\r\n");

                output.append(&mut body.content.clone());
            }
            None => {}
        }

        output
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output: String = String::new();

        output.push_str(&format!("HTTP/1.1 {}\r\n", self.status.to_string()));

        self.header.keys().into_iter().for_each(|key| {
            output.push_str(&format!("{}: {}\r\n", key, &self.header.get(key).unwrap()));
        });

        match &self.body {
            Some(body) => {
                output.push_str(&format!("Content-Length: {}\r\n", body.content.len()));
                output.push_str(&format!(
                    "Content-Type: {}\r\n",
                    body.content_type.to_string()
                ));

                output.push_str(&format!("\r\n"));

                output.push_str(&str::from_utf8(&body.content).unwrap());
            }
            None => {}
        }

        write!(f, "{}", output)
    }
}
