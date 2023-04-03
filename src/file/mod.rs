//! file_reader module is responsible for storing functions that parses file paths and retrieving file data

use log::trace;
use std::{
    error::Error,
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};
use strum_macros::Display;

#[cfg(test)]
mod tests;

#[derive(PartialEq, Debug, Display)]
pub enum FileError {
    FileDoesNotExist,
    InaccessibleExtension,
}

impl Error for FileError {}

/// parse method converts a file path into a PathBuf
///
/// # Errors
/// None is returned instead of a PathBuf if file does not exist or the file has unaccessible extensions. Unaccessible extensions are defined in settings.ron for each path.
pub fn parse<F: Fn(&str) -> bool>(
    url: &str,
    search_folder: &str,
    allowed_extension: F,
) -> Result<PathBuf, FileError> {
    let url = url.trim_matches('\\').trim_matches('/');

    let mut path_buffer = PathBuf::new();

    path_buffer.push(r"source");

    path_buffer.push(&search_folder);

    match url.rfind('?') {
        Some(index) => path_buffer.push(url[0..index].trim_matches('\\')),
        None => path_buffer.push(url.trim_matches('\\')),
    }

    if path_buffer.extension().is_none() {
        path_buffer.push("index.html");
    }

    let extension = match path_buffer.extension() {
        Some(extension) => extension.to_str().unwrap(),
        None => panic!("path_buffer has no file extension"),
    };

    trace!(
        "path:{:?}\textension:{:?}",
        &path_buffer,
        path_buffer.extension()
    );

    //file checks
    if !path_buffer.exists() {
        return Err(FileError::FileDoesNotExist);
    }

    if !allowed_extension(extension) {
        return Err(FileError::InaccessibleExtension);
    }

    return Ok(path_buffer);
}

/// get_file_content_string returns string content of a file
///
/// # Errors
/// None is returned if the file does not exist
/// None is also returned if file cannot be read as a string. Ex. an file like jpg would cause this error
pub fn get_file_content_string(file_path: &Path) -> Option<String> {
    let mut file = match File::open(file_path) {
        Err(_err) => return None,
        Ok(file) => file,
    };

    let mut contents: String = String::new();

    match file.read_to_string(&mut contents) {
        Err(_err) => return None,
        Ok(_) => return Some(contents),
    }
}

/// get_file_content_bytes returns vector of bytes of file contents
///
/// # Errors
/// None is returned if the file does not exist
pub fn get_file_content_bytes(file_path: &Path) -> Option<Vec<u8>> {
    let mut file = match File::open(file_path) {
        Err(_err) => return None,
        Ok(file) => file,
    };

    let mut contents: Vec<u8> = Vec::new();

    match file.read_to_end(&mut contents) {
        Err(_err) => return None,
        Ok(_) => return Some(contents),
    }
}
