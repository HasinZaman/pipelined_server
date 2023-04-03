//! request module is responsible for enums, structs and functions responsible for parsing Requests
pub mod method;
pub mod parser_error;

#[cfg(test)]
mod tests;

use std::{
    collections::HashMap,
    str::{FromStr, Split},
};

use self::{method::Method, parser_error::ParserError};

use super::body::{Body, ContentType};

#[derive(Debug, PartialEq, Eq)]
pub struct Request(pub Method, pub HashMap<String, String>);

impl FromStr for Request {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut request = s.split("\n");

        let (method, target, _version) = match get_start_line(request.next()) {
            Ok(ok) => ok,
            Err(err) => return Err(err),
        };

        let method = method.to_uppercase();

        let (body, meta_data) = get_data(request)?;

        let method: Method = Method::new(method.to_string(), target.to_string(), body)?;

        Ok(Request(method, meta_data))
    }
}

/// get_start_line method extracts the method, http version and target from the first line of a request
///
/// # Errors
/// If the first line does not contain all three method, target and http version; in the specified order - then a ParserError is returned for an invalid request.
fn get_start_line<'a>(
    start_line: Option<&'a str>,
) -> Result<(&'a str, &'a str, &'a str), ParserError> {
    let start_line = match start_line {
        Some(val) => val,
        None => {
            return Err(ParserError::InvalidMethod(Some(String::from(
                "No start line",
            ))))
        }
    };

    let mut start_line = start_line.split_whitespace();

    let method: &str = start_line
        .next()
        .ok_or(ParserError::InvalidMethod(Some(String::from("No method"))))?;
    let target: &str = start_line
        .next()
        .ok_or(ParserError::InvalidMethod(Some(String::from("No targert"))))?;
    let version: &str = start_line
        .next()
        .ok_or(ParserError::InvalidMethod(Some(String::from(
            "No HTTP version",
        ))))?;

    Ok((method, target, version))
}

/// get_data method extras metadata and possible Body from request
///
/// # Errors
/// A ParserError is returned if a body doesn't have a valid content-type
fn get_data<'a>(
    mut line_iter: Split<&'a str>,
) -> Result<(Option<Body>, HashMap<String, String>), ParserError> {
    let mut meta_data: HashMap<String, String> = HashMap::new();

    let mut content_type: Option<&str> = None;
    let mut content_lenght: Option<&str> = None;

    loop {
        let line = match line_iter.next() {
            Some(value) => value,
            None => break,
        };

        if line == "\r" || line == "" {
            break;
        }

        let (key, value) = get_key_value_pair(line)?;

        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();

        if key == "content-type" {
            content_type = Some(value);
        } else if key == "content-length" {
            content_lenght = Some(value);
        } else {
            meta_data.insert(key, value.to_string());
        }
    }

    if content_type.is_none() || content_lenght.is_none() {
        return Ok((None, meta_data));
    }

    let content_type = content_type.unwrap();
    let _content_lenght = content_lenght.unwrap();

    let mut body: String = String::from("");

    loop {
        let line = match line_iter.next() {
            Some(value) => value,
            None => break,
        };

        if line.contains("\u{0}") {
            body.push_str(line.trim_end_matches('\u{0}'));
            break;
        }
        body.push_str(line);
    }

    let content_type = match ContentType::new(content_type) {
        Ok(val) => val,
        Err(err) => return Err(err),
    };

    let body = Body {
        content_type: content_type,
        content: body.as_bytes().to_vec(),
    };

    return Ok((Some(body), meta_data));
}

/// get_key_value_pair extracts key and associated value from unparsed metadata string
///
/// # Errors
/// A parse Error is returned if the data isn't stored in the format [key]:value
fn get_key_value_pair<'a>(line: &'a str) -> Result<(&'a str, &'a str), ParserError> {
    let mut line = line.split(':');
    let key = line
        .next()
        .ok_or(ParserError::InvalidMethod(Some(String::from(
            "No key in key value pair",
        ))))?;
    let value = line
        .next()
        .ok_or(ParserError::InvalidMethod(Some(String::from(
            "No value in key value pair",
        ))))?;

    Ok((key, value))
}
