use std::{
    collections::HashMap,
    fs::read,
    mem,
    path::PathBuf,
    sync::{
        mpsc::{self, Sender},
    },
    thread::{self, JoinHandle},
};

use log::{error, info, trace, warn};

use crate::{
    file::{self, FileError},
    http::{
        body::{Body, ContentType, Text},
        method::{Method},
        request::Request,
        response::{response_status_code::ResponseStatusCode, Response},
    },
    pipeline::pipeline::Bytes,
    setting::{ServerSetting}, ActionBuilder,
};

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

pub fn not_allowed_logic(
    _: &Request,
    _: &ServerSetting,
    _: &FileUtilitySender<FileError>,
) -> Result<Response, ResponseStatusCode> {
    Err(ResponseStatusCode::MethodNotAllowed)
}

pub fn default_err_page(
    err_code: &ResponseStatusCode,
    _: &ServerSetting,
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

pub const NO_BOUND: usize = 0;
pub fn generate_read_only_file_utility_thread<const MAX_READS: usize>(
) -> (FileUtilitySender<FileError>, JoinHandle<()>) {
    // todo!() fn should take into account reads(aka RWLock)
    // todo!() fn should have server wide caching
    let (tx, rx) = mpsc::channel::<(PathBuf, Sender<Result<Vec<u8>, FileError>>)>();

    let thread = thread::spawn(move || {
        let mut threads = Vec::new();
        let mut cache: Option<(PathBuf, Sender<Result<Vec<u8>, FileError>>)> = Option::None;
        loop {
            match cache {
                Some(_) => {
                    if MAX_READS == 0 || threads.len() <= MAX_READS {
                        let mut tmp = None;

                        mem::swap(&mut cache, &mut tmp);

                        let (path, data_ch) = tmp.unwrap();

                        threads.push(thread::spawn(move || {
                            let _ = data_ch.send(match read(path) {
                                Ok(bytes) => Ok(bytes),
                                Err(_err) => Err(FileError::FileDoesNotExist),
                            });
                        }));
                    }
                }
                None => {
                    if let Ok(val) = rx.try_recv() {
                        cache = Some(val);
                    }
                }
            }

            threads = threads.into_iter().filter(|t| !t.is_finished()).collect();
        }
    });

    return (tx, thread);
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
        } //return Err(ResponseStatusCode::Forbidden),
    };

    let path = match file {
        Ok(path) => path,
        Err(err) => {
            return match err {
                FileError::FileDoesNotExist => {
                    info!("File does not exist");
                    Err(ResponseStatusCode::NotFound)
                }
                FileError::InaccessibleExtension => {
                    info!("Invalid file extension");
                    Err(ResponseStatusCode::Forbidden)
                }
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

    trace!("Waiting for file retrieval response");
    match rx.recv() {
        Ok(bytes) => match bytes {
            Ok(bytes) => {
                trace!("File retrieved");
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
