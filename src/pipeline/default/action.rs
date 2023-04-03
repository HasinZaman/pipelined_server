use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, Sender},
        RwLock,
    }, thread::{self, JoinHandle}, fs::read,
};

use log::{trace, info, warn, error};

use crate::{
    file::{self, FileError},
    http::{
        body::{Application, Body, ContentType, Image, Text, Video},
        method::{self, Method},
        request::Request,
        response::{response_status_code::ResponseStatusCode, Response},
    },
    pipeline::pipeline::Bytes,
    setting::{self, ServerSetting},
};

type MethodFn = fn(Request, &ServerSetting) -> Result<Response, ResponseStatusCode>;
type ErrFn = fn(ResponseStatusCode, ServerSetting) -> Response;

#[macro_export]
macro_rules! ActionBuilder {
    (
        name = $name: ident,

        utility = $utility: ty,

        get = $get: ident,
        head = $head: ident,
        post = $post: ident,
        put = $put: ident,
        delete = $delete: ident,
        connect = $connect: ident,
        options =$options: ident,
        trace = $trace: ident,
        patch = $patch: ident,

        error = $err: ident
     ) => {
        use paste::paste;

        paste!{
            pub fn [<$name>] (request: &Result<Request, ResponseStatusCode>, setting: ServerSetting, utility_thread: &mut $utility) -> Result<Response, ResponseStatusCode> {
                match request{
                    Ok(request) => {
                        match &request.0 {
                            Method::Get { .. } => {
                                trace!("Get");
                                $get(request, &setting, utility_thread)
                            },
                            Method::Head { .. } => {
                                trace!("Head");
                                $head(request, &setting, utility_thread)
                            },
                            Method::Post { .. } => {
                                trace!("Post");
                                $post(request, &setting, utility_thread)
                            },
                            Method::Put { .. } => {
                                trace!("Put");
                                $put(request, &setting, utility_thread)
                            },
                            Method::Delete { .. } => {
                                trace!("Delete");
                                $delete(request, &setting, utility_thread)
                            },
                            Method::Connect { .. } => {
                                trace!("Connect");
                                $connect(request, &setting, utility_thread)
                            },
                            Method::Options { .. } => {
                                trace!("Options");
                                $options(request, &setting, utility_thread)
                            },
                            Method::Trace { .. } => {
                                trace!("Trace");
                                $trace(request, &setting, utility_thread)
                            },
                            Method::Patch { .. } => {
                                trace!("Patch");
                                $patch(request, &setting, utility_thread)
                            },
                        }
                    },
                    Err(err_code) => {
                        trace!("Error");
                        return Ok(
                            $err(err_code, setting, utility_thread)
                        )
                    },
                }
            }
        }
    }
}

ActionBuilder!(
    name = default_action,
    utility = FileUtilitySender<FileError>,
    get = default_get_logic,
    head = not_allowed_logic,
    post = not_allowed_logic,
    put = not_allowed_logic,
    delete = not_allowed_logic,
    connect = not_allowed_logic,
    options = not_allowed_logic,
    trace = not_allowed_logic,
    patch = not_allowed_logic,
    error = default_err_page
);

// pub fn build(self) -> Box<dyn Fn(Result<Request, ResponseStatusCode>,ServerSetting) -> Result<Response, ResponseStatusCode>> {
//     Box::new(
//         |request: Result<Request, ResponseStatusCode>, setting: ServerSetting| -> Result<Response, ResponseStatusCode> {
//             match request{
//                 Ok(request) => {
//                     match request.0 {
//                         Method::Get { file } => (self.get)(request, &setting),
//                         Method::Head { file } => (self.head)(request, &setting),
//                         Method::Post { file, body } => (self.post)(request, &setting),
//                         Method::Put { file, body } => (self.put)(request, &setting),
//                         Method::Delete { file, body } => (self.delete)(request, &setting),
//                         Method::Connect { url } => (self.connect)(request, &setting),
//                         Method::Options { url } => (self.options)(request, &setting),
//                         Method::Trace { file } => (self.trace)(request, &setting),
//                         Method::Patch { file, body } => (self.patch)(request, &setting),
//                     }
//                 },
//                 Err(err_code) => {
//                     return Ok(
//                         (self.error_page)(err_code, setting)
//                     )
//                 },
//             }
//         }
//     )
// }

pub fn not_allowed_logic(
    _: &Request,
    _: &ServerSetting,
    _: &FileUtilitySender<FileError>,
) -> Result<Response, ResponseStatusCode> {
    Err(ResponseStatusCode::MethodNotAllowed)
}

pub fn default_err_page(
    err_code: &ResponseStatusCode,
    _: ServerSetting,
    _: &FileUtilitySender<FileError>,
) -> Response {
    Response {
        header: HashMap::new(),
        body: Some(Body {
            content_type: ContentType::Text(Text::html),
            content: format!("<H1>{}</H1>", err_code.to_string())
                .as_bytes()
                .to_vec(),
        }),
        status: *err_code,
    }
}
pub(crate) type FileUtilitySender<E> = mpsc::Sender<(PathBuf, Sender<Result<Bytes, E>>)>;

pub fn generate_file_utility_thread() -> (FileUtilitySender<FileError>, JoinHandle<()>) {
    let (tx,rx) = mpsc::channel::<(PathBuf, Sender<Result<Vec<u8>, FileError>>)>();


    let thread = thread::spawn(move || {

        loop {

            let (path, data_ch) = match rx.recv() {
                Ok(val) => val,
                Err(_) => {
                    //send error
                    todo!();
                    continue;
                },
            };

            //get file data
            let _ = data_ch.send(match read(path) {
                Ok(bytes) => Ok(bytes),
                Err(_err) => Err(FileError::FileDoesNotExist),
            });
        }
    });

    return (tx, thread)
}

pub fn default_get_logic(
    request: &Request,
    setting: &ServerSetting,
    utility_thread: &FileUtilitySender<FileError>,
) -> Result<Response, ResponseStatusCode> {
    let Request(method, meta_data) = request;

    let host: &str = match meta_data.get("host") {
        Some(host) => host,
        None => return Err(ResponseStatusCode::ImATeapot),
    };

    info!("Request Host:{}", host);

    let domain_path = match setting.paths.get(host) {
        Some(setting) => setting,
        None => return Err(ResponseStatusCode::Forbidden),
    };

    let file = match method {
        Method::Get { file } => file::parse(&file, &domain_path.path, |ext| {
            domain_path
                .allow
                .iter()
                .any(|allowed_ext| ext == allowed_ext)
        }),
        _ => {
            error!("Invalid request method in Get function");
            panic!("Invalid Request")
        }, //return Err(ResponseStatusCode::Forbidden),
    };

    let path = match file {
        Ok(path) => path,
        Err(err) => {
            return match err {
                FileError::FileDoesNotExist => {
                    info!("File does not exist");
                    Err(ResponseStatusCode::NotFound)
                },
                FileError::InaccessibleExtension => {
                    info!("Invalid file extension");
                    Err(ResponseStatusCode::Forbidden)
                },
            }
        }
    };

    let ext = path.extension().unwrap().to_str().unwrap().to_string();

    let (tx, rx) = mpsc::channel();

    {
        let mut attempts = 0;

        let mut val = (path, tx);

        trace!("Begin file retrieval");

        while let Err(err) = utility_thread.send(val) {
            if attempts > 5 {
                return Err(ResponseStatusCode::InternalServerError);
            }

            val = err.0;

            attempts += 1;
            warn!("Failed to retrieve file\nattempt:{}", attempts);
        }
    }

    match rx.recv() {
        Ok(bytes) => match bytes {
            Ok(bytes) => {
                let content_type = match ContentType::try_from(ext.as_str()) {
                    Ok(content_type) => content_type,
                    Err(_err) => todo!(),
                };

                let content = bytes;

                return Ok(Response {
                    status: ResponseStatusCode::Ok,
                    header: HashMap::new(),
                    body: Some(Body {
                        content_type,
                        content,
                    }),
                });
            }
            Err(err) => match err {
                FileError::FileDoesNotExist => return Err(ResponseStatusCode::NotFound),
                FileError::InaccessibleExtension => return Err(ResponseStatusCode::Forbidden),
            },
        },
        Err(err) => {
            warn!("Failed to recv error: {}", err);
            todo!()
        }
    }
}

// pub fn default_get_logic(request: Method, server_settings: &ServerSetting, meta_data: &HashMap<String, String>) -> Result<Response, ResponseStatusCode> {
//     if let Method::Get{file} = request {
//         let host: &str = match meta_data.get("host") {
//             Some(host) => host,
//             None => {
//                 return Err(ResponseStatusCode::ImATeapot)
//             }
//         };

//         let (host_path, allowed_extension) = match server_settings.paths.get(host) {
//             Some(host_path) => (&host_path.path, &host_path.allow),
//             None => {
//                 return Err(ResponseStatusCode::NotFound)
//             }
//         };

//         let path_buf = match file_reader::parse(&file, &host_path, allowed_extension) {
//             Some(path_buf) => path_buf,
//             None => {
//                 match file_reader::parse(
//                     "error\\404.html",
//                     &host_path,
//                     &allowed_extension,
//                 ) {
//                     Some(file) => file,
//                     None => {
//                         return Response {
//                             status: ResponseStatusCode::NotFound,
//                             meta_data : HashMap::new(),
//                             body: Some(Body {
//                                 content_type: ContentType::Text(Text::html),
//                                 content: format!(
//                                     "<html><h1>Error code: {}</h1><p>{}</p></html>",
//                                     ResponseStatusCode::NotFound.get_code(),
//                                     ResponseStatusCode::NotFound.to_string()
//                                 )
//                                 .as_bytes()
//                                 .to_vec(),
//                             }),
//                         }
//                     }
//                 }
//             }
//         };

//         let file_name = match path_buf.file_name() {
//             Some(name) => name.to_str().unwrap(),
//             None => panic!(
//                 "FileReader::parse does not return file name. {:?}",
//                 path_buf
//             ),
//         };

//         let body = file_reader::get_file_content_bytes(&path_buf).unwrap();

//         match file_name {
//             "404.html" => {
//                 return Response {
//                     status: ResponseStatusCode::NotFound,
//                     meta_data : HashMap::new(),
//                     body: Some(Body {
//                         content_type: ContentType::Text(Text::html),
//                         content: body,
//                     }),
//                 }
//             }
//             _ => {
//                 let content_type = path_buf.extension().unwrap();
//                 let content_type = content_type.to_str().unwrap();

//                 let content_type: ContentType = match content_type {
//                     "gif" => ContentType::Image(Image::gif),
//                     "jpg" | "jpeg" | "jpe" | "jfif" => ContentType::Image(Image::jpeg),
//                     "png" => ContentType::Image(Image::png),
//                     "tif" | "tiff" => ContentType::Image(Image::tiff),
//                     "css" => ContentType::Text(Text::css),
//                     "csv" => ContentType::Text(Text::csv),
//                     "html" => ContentType::Text(Text::html),
//                     "js" => ContentType::Text(Text::javascript),
//                     "xml" => ContentType::Text(Text::xml),
//                     "mpeg" => ContentType::Video(Video::mpeg),
//                     "mp4" => ContentType::Video(Video::mp4),
//                     "webm" => ContentType::Video(Video::webm),
//                     "svg" => ContentType::Image(Image::svg_xml),
//                     "woff" => ContentType::Application(Application::woff),
//                     _ => {
//                         return Response {
//                             status: ResponseStatusCode::UnsupportedMediaType,
//                             meta_data : HashMap::new(),
//                             body: None,
//                         }
//                     }
//                 };

//                 Response {
//                     status: ResponseStatusCode::Ok,
//                     meta_data : HashMap::from([("Cache-Control".to_string(), "private".to_string())]),
//                     body: Some(Body {
//                         content_type: content_type,
//                         content: body,
//                     }),
//                 }
//             }
//         }
//     } else {
//         panic!("default_get_logic should only used to handle Method::Get requests")
//     }
// }
