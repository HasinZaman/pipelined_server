//! body module is responsible for how HTTP body are parsed and stored
use super::request::parser_error::ParserError;
use std::{
    fmt::{Debug, Display, Error, Formatter},
    str::FromStr,
};

/// Body struct defines the structure of an HTTP body
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Body {
    pub content_type: ContentType,
    pub content: Vec<u8>,
}

/// ContentType enum define the different types types of [content types](https://www.iana.org/assignments/media-types/media-types.xhtml)
//https://www.geeksforgeeks.org/http-headers-content-type/
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Application(Application),
    Audio(Audio),
    Image(Image),
    Multipart(Multipart),
    Text(Text),
    Video(Video),
}

impl ContentType {
    /// FromStr that creates a new ContentType from string
    ///
    /// # Example
    /// ```
    /// let actual: ContentType = ContentType::Application(Application::EDI_X12);
    /// assert_eq!(actual, ContentType::new("application/EDI-X12"));
    /// ```
    pub fn new(raw_str: &str) -> Result<ContentType, ParserError> {
        let str_vec: Vec<&str> = raw_str.split("/").collect();

        if str_vec.len() < 2 {
            return Err(ParserError::InvalidMethod(Some(String::from(
                "Start line must have more than 2 elements",
            ))));
        }

        let type_raw: &str = str_vec[0];

        let value: &str = str_vec[1];

        match type_raw {
            "application" => {
                let content_type_value = Application::from_str(value)?;
                let content_type = ContentType::Application(content_type_value);
                Ok(content_type)
            }
            "audio" => {
                let content_type_value = Audio::from_str(value)?;
                let content_type = ContentType::Audio(content_type_value);
                Ok(content_type)
            }
            "image" => {
                let content_type_value = Image::from_str(value)?;
                let content_type = ContentType::Image(content_type_value);
                Ok(content_type)
            }
            "multipart" => {
                let content_type_value = Multipart::from_str(value)?;
                let content_type = ContentType::Multipart(content_type_value);
                Ok(content_type)
            }
            "text" => {
                let content_type_value = Text::from_str(value)?;
                let content_type = ContentType::Text(content_type_value);
                Ok(content_type)
            }
            "video" => {
                let content_type_value = Video::from_str(value)?;
                let content_type = ContentType::Video(content_type_value);
                Ok(content_type)
            }
            _ => Err(ParserError::InvalidMethod(Some(String::from(
                "Invalid type",
            )))),
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ContentType::Application(value) => write!(f, "application/{}", value),
            ContentType::Audio(value) => write!(f, "audio/{}", value),
            ContentType::Image(value) => write!(f, "image/{}", value),
            ContentType::Multipart(value) => write!(f, "multipart/{}", value),
            ContentType::Text(value) => write!(f, "text/{}", value),
            ContentType::Video(value) => write!(f, "video/{}", value),
        }
    }
}

impl Debug for ContentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct(&format!("ContentType: {}", self.to_string()))
            .finish()
    }
}

impl TryFrom<&str> for ContentType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        println!("{}", value);
        match value {
            "css" => Ok(Self::Text(Text::css)),
            "csv" => Ok(Self::Text(Text::csv)),
            "html" => Ok(Self::Text(Text::html)),
            "js" | "cjs" | "mjs" => Ok(Self::Text(Text::javascript)),
            "txt" => Ok(Self::Text(Text::plain)),
            "xml" => Ok(Self::Text(Text::xml)),

            "gif" => Ok(Self::Image(Image::gif)),
            "jpg" | "jpeg" | "jpe" | "jif" | "jfif" | "jfi" => Ok(Self::Image(Image::jpeg)),
            "png" => Ok(Self::Image(Image::png)),
            "tiff" | "tif" => Ok(Self::Image(Image::tiff)),
            "cur" => Ok(Self::Image(Image::vnd_microsoft_icon)),
            "ico" => Ok(Self::Image(Image::x_icon)),
            "djvu" | "djv" => Ok(Self::Image(Image::vnd_djvu)),
            "svg" | "svgz" => Ok(Self::Image(Image::svg_xml)),
            
            "ogg" | "ogv" | "oga" | "ogx" | "ogm" | "spx" | "opus" => Ok(Self::Application(Application::ogg)),
            "pdf" => Ok(Self::Application(Application::pdf)),
            "xhtml" => Ok(Self::Application(Application::xhtml_xml)),
            "json" => Ok(Self::Application(Application::json)),
            "jsonld" => Ok(Self::Application(Application::ld_json)),
            "zip" | "zipx" => Ok(Self::Application(Application::zip)),
            "woff" | "woff2" => Ok(Self::Application(Application::woff)),

            //"mpeg" => Ok(Self::Audio(Audio::mpeg)),
            "wma" => Ok(Self::Audio(Audio::x_ms_wma)),
            "ra" | "ram" => Ok(Self::Audio(Audio::vnd_rn_realaudio)),
            "wav" | "wave" => Ok(Self::Audio(Audio::x_wav)),

            "mp4" | "m4a" | "m4p" | "m4b" | "m4r" | "m4v" => Ok(Self::Video(Video::mp4)),
            "wmv" => Ok(Self::Video(Video::x_ms_wmv)),
            "avi" => Ok(Self::Video(Video::x_msvideo)),
            "webm" => Ok(Self::Video(Video::webm)),
            
            _ => todo!(),
        }
    }
}

// //! value module defines the specific Enums for each of the ContentType variants
// use super::super::request::parser_error::ParserError;

/// Application enum defines the variants of ContentType::Application
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Application {
    EDI_X12,
    EDIFACT,
    javascript,
    octet_stream,
    ogg,
    pdf,
    xhtml_xml,
    x_shockwave_flash,
    json,
    ld_json,
    xml,
    zip,
    x_www_form_urlencoded,
    woff,
}

impl Display for Application {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Application::EDI_X12 => "EDI-X12",
                Application::EDIFACT => "EDIFACT",
                Application::javascript => "javascript",
                Application::octet_stream => "octet-stream",
                Application::ogg => "ogg",
                Application::pdf => "pdf",
                Application::xhtml_xml => "xhtml+xml",
                Application::x_shockwave_flash => "x-shockwave-flash",
                Application::json => "json",
                Application::ld_json => "ld+json",
                Application::xml => "xml",
                Application::zip => "zip",
                Application::x_www_form_urlencoded => "x-www-form-urlencoded",
                Application::woff => "woff",
            }
        )
    }
}

impl FromStr for Application {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value_parameter_vec: Vec<&str> = s.split(";").collect();

        let value: &str = value_parameter_vec[0];
        match value {
            "EDI-X12" => Ok(Application::EDI_X12),
            "EDIFACT" => Ok(Application::EDIFACT),
            "javascript" => Ok(Application::javascript),
            "octet-stream" => Ok(Application::octet_stream),
            "ogg" => Ok(Application::ogg),
            "pdf" => Ok(Application::pdf),
            "xhtml+xml" => Ok(Application::xhtml_xml),
            "x-shockwave-flash" => Ok(Application::x_shockwave_flash),
            "json" => Ok(Application::json),
            "ld+json" => Ok(Application::ld_json),
            "xml" => Ok(Application::xml),
            "zip" => Ok(Application::zip),
            "x-www-form-urlencoded" => Ok(Application::x_www_form_urlencoded),
            "woff" => Ok(Application::woff),
            _ => Err(ParserError::InvalidMethod(Some(format!(
                "{} is not a valid Application variant",
                s
            )))),
        }
    }
}

/// Application enum defines the variants of ContentType::Audio
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Audio {
    mpeg,
    x_ms_wma,
    vnd_rn_realaudio,
    x_wav,
}

impl Display for Audio {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Audio::mpeg => "mpeg",
                Audio::x_ms_wma => "x-ms-wma",
                Audio::vnd_rn_realaudio => "vnd.rn-realaudio",
                Audio::x_wav => "x-wav",
            }
        )
    }
}

impl FromStr for Audio {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value_parameter_vec: Vec<&str> = s.split(";").collect();

        let value: &str = value_parameter_vec[0];
        match value {
            "mpeg" => Ok(Audio::mpeg),
            "x-ms-wma" => Ok(Audio::x_ms_wma),
            "vnd.rn-realaudio" => Ok(Audio::vnd_rn_realaudio),
            "x-wav" => Ok(Audio::x_wav),
            _ => Err(ParserError::InvalidMethod(Some(format!(
                "{} is not a valid Audio variant",
                s
            )))),
        }
    }
}

/// Application enum defines the variants of ContentType::Image
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Image {
    gif,
    jpeg,
    png,
    tiff,
    vnd_microsoft_icon,
    x_icon,
    vnd_djvu,
    svg_xml,
}

impl Display for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Image::gif => "gif",
                Image::jpeg => "jpeg",
                Image::png => "png",
                Image::tiff => "tiff",
                Image::vnd_microsoft_icon => "vnd.microsoft.icon",
                Image::x_icon => "x-icon",
                Image::vnd_djvu => "vnd.djvu",
                Image::svg_xml => "svg+xml",
            }
        )
    }
}

impl FromStr for Image {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value_parameter_vec: Vec<&str> = s.split(";").collect();

        let value: &str = value_parameter_vec[0];
        match value {
            "gif" => Ok(Image::gif),
            "jpeg" => Ok(Image::jpeg),
            "png" => Ok(Image::png),
            "tiff" => Ok(Image::tiff),
            "vnd.microsoft.icon" => Ok(Image::vnd_microsoft_icon),
            "x-icon" => Ok(Image::x_icon),
            "vnd.djvu" => Ok(Image::vnd_djvu),
            "svg+xml" => Ok(Image::svg_xml),
            _ => Err(ParserError::InvalidMethod(Some(format!(
                "{} is not a valid Image variant",
                s
            )))),
        }
    }
}

/// Application enum defines the variants of ContentType::Multipart
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Multipart {
    mixed,
    alternative,
    related,
    // form_data { boundary: String },
}

impl Display for Multipart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Multipart::mixed => "mixed",
                Multipart::alternative => "alternative",
                Multipart::related => "related",
                // #[allow(unused_variables)]
                // Multipart::form_data { boundary } => "form-data",
            }
        )
    }
}

impl FromStr for Multipart {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value_parameter_vec: Vec<&str> = s.split(";").collect();

        let value: &str = value_parameter_vec[0];
        match value {
            "mixed" => Ok(Multipart::mixed),
            "alternative" => Ok(Multipart::alternative),
            "related" => Ok(Multipart::related),
            // "form-data" => Ok(Multipart::form_data {
            //     boundary: String::from(""),
            // }),
            _ => Err(ParserError::InvalidMethod(Some(format!(
                "{} is not a valid Multipart variant",
                s
            )))),
        }
    }
}

/// Application enum defines the variants of ContentType::Text
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Text {
    css,
    csv,
    html,
    javascript,
    plain,
    xml,
}

impl Display for Text {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Text::css => "css",
                Text::csv => "csv",
                Text::html => "html",
                Text::javascript => "javascript",
                Text::plain => "plain",
                Text::xml => "xml",
            }
        )
    }
}

impl FromStr for Text {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value_parameter_vec: Vec<&str> = s.split(";").collect();

        let value: &str = value_parameter_vec[0];
        match value {
            "css" => Ok(Text::css),
            "csv" => Ok(Text::csv),
            "html" => Ok(Text::html),
            "javascript" => Ok(Text::javascript),
            "plain" => Ok(Text::plain),
            "xml" => Ok(Text::xml),
            _ => Err(ParserError::InvalidMethod(Some(format!(
                "{} is not a valid Text variant",
                s
            )))),
        }
    }
}

/// Application enum defines the variants of ContentType::Video
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Video {
    mpeg,
    mp4,
    quicktime,
    x_ms_wmv,
    x_msvideo,
    x_flv,
    webm,
}

impl Display for Video {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Video::mpeg => "mpeg",
                Video::mp4 => "mp4",
                Video::quicktime => "quicktime",
                Video::x_ms_wmv => "x-ms-wmv",
                Video::x_msvideo => "x-msvideo",
                Video::x_flv => "x-flv",
                Video::webm => "webm",
            }
        )
    }
}

impl FromStr for Video {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value_parameter_vec: Vec<&str> = s.split(";").collect();

        let value: &str = value_parameter_vec[0];
        match value {
            "mpeg" => return Ok(Video::mpeg),
            "mp4" => return Ok(Video::mp4),
            "quicktime" => return Ok(Video::quicktime),
            "x-ms-wmv" => return Ok(Video::x_ms_wmv),
            "x-msvideo" => return Ok(Video::x_msvideo),
            "x-flv" => return Ok(Video::x_flv),
            "webm" => return Ok(Video::webm),
            _ => panic!("Invalid variant type"),
        }
    }
}
