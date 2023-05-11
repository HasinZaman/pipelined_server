mod compressor {
    use std::collections::HashMap;

    use crate::{
        http::{
            body::{Body, ContentType, Text},
            method::Method,
            request::Request,
            response::{response_status_code::ResponseStatusCode, Response},
        },
        pipeline::default::no_compression,
        setting::ServerSetting,
    };

    // compression check
    #[test]
    fn no_compression_basic() {
        let data = Response {
            status: ResponseStatusCode::Continue,
            header: HashMap::new(),
            body: None,
        };

        let request = Request(
            Method::Get {
                file: String::from(""),
            },
            HashMap::new(),
        );
        let server = ServerSetting {
            address: String::from(""),
            port: 8080,
            paths: HashMap::new(),
        };

        println!("{}", data);

        let actual = no_compression(data, Some(request), server);
        let expected = vec![
            72, 84, 84, 80, 47, 49, 46, 49, 32, 49, 48, 48, 32, 67, 111, 110, 116, 105, 110, 117,
            101, 13, 10,
        ];

        assert_eq!(actual, expected)
    }

    #[test]
    fn no_compression_with_header() {
        let data = Response {
            status: ResponseStatusCode::Continue,
            header: {
                let mut tmp = HashMap::new();

                tmp.insert(String::from("key"), String::from("value"));

                tmp
            },
            body: None,
        };

        let request = Request(
            Method::Get {
                file: String::from(""),
            },
            HashMap::new(),
        );
        let server = ServerSetting {
            address: String::from(""),
            port: 8080,
            paths: HashMap::new(),
        };

        println!("{}", data);

        let actual = no_compression(data, Some(request), server);
        let expected = vec![
            72, 84, 84, 80, 47, 49, 46, 49, 32, 49, 48, 48, 32, 67, 111, 110, 116, 105, 110, 117,
            101, 13, 10, 107, 101, 121, 58, 32, 118, 97, 108, 117, 101, 13, 10,
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn no_compression_with_body() {
        let data = Response {
            status: ResponseStatusCode::Continue,
            header: {
                let mut tmp = HashMap::new();

                tmp.insert(String::from("key"), String::from("value"));

                tmp
            },
            body: Some(Body {
                content_type: ContentType::Text(Text::plain),
                content: String::from("hello world").as_bytes().to_vec(),
            }),
        };

        let request = Request(
            Method::Get {
                file: String::from(""),
            },
            HashMap::new(),
        );
        let server = ServerSetting {
            address: String::from(""),
            port: 8080,
            paths: HashMap::new(),
        };

        let actual = no_compression(data, Some(request), server);
        let expected = vec![
            72, 84, 84, 80, 47, 49, 46, 49, 32, 49, 48, 48, 32, 67, 111, 110, 116, 105, 110, 117,
            101, 13, 10, 107, 101, 121, 58, 32, 118, 97, 108, 117, 101, 13, 10, 67, 111, 110, 116,
            101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32, 49, 49, 13, 10, 67, 111, 110,
            116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 116, 101, 120, 116, 47, 112, 108,
            97, 105, 110, 13, 10, 10, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100,
        ];

        assert_eq!(actual, expected);
    }

    // different algo compression check
}
mod parser {
    use std::{
        collections::HashMap,
        io::Write,
        net::{TcpListener, TcpStream},
        sync::{Arc, Condvar, Mutex},
        thread,
    };

    use serial_test::serial;

    use crate::{
        http::{
            method::Method, request::Request, response::response_status_code::ResponseStatusCode,
        },
        pipeline::default::parser,
    };

    #[test]
    #[serial]
    fn get_request() {
        const HOST: &str = "localhost:8080";
        let listener = TcpListener::bind(HOST).unwrap();

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = Arc::clone(&pair);

        let _thread = thread::spawn(move || {
            let (lock, cvar) = &*pair2;
            let mut started = lock.lock().unwrap();

            while !*started {
                started = cvar.wait(started).unwrap();
            }

            //send data
            let mut stream = TcpStream::connect(HOST).unwrap();

            let _ = stream.write(b"GET /index.html  HTTP/1.1");
        });

        {
            let mut start = pair.0.lock().unwrap();

            *start = true;

            pair.1.notify_one()
        }

        {
            let mut stream = listener.incoming().next().unwrap().unwrap();

            let actual = parser::parser::<100, 500, 20, 250>(&mut stream);

            assert_eq!(
                Ok(Request(
                    Method::Get {
                        file: String::from("/index.html")
                    },
                    HashMap::new()
                )),
                actual
            );
        }
    }

    #[test]
    #[serial]
    fn too_large_payload_request() {
        const HOST: &str = "localhost:8080";
        let listener = TcpListener::bind(HOST).unwrap();

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = Arc::clone(&pair);

        let _thread = thread::spawn(move || {
            let (lock, cvar) = &*pair2;
            let mut started = lock.lock().unwrap();

            while !*started {
                started = cvar.wait(started).unwrap();
            }

            //send data
            let mut stream = TcpStream::connect(HOST).unwrap();

            let _ = stream.write(b"GET /index.html HTTP/1.1");
        });

        {
            let mut start = pair.0.lock().unwrap();

            *start = true;

            pair.1.notify_one()
        }

        {
            let mut stream = listener.incoming().next().unwrap().unwrap();

            let actual = parser::parser::<5, 5, 20, 250>(&mut stream);

            assert_eq!(Err(ResponseStatusCode::PayloadTooLarge), actual);
        }
    }

    #[test]
    #[serial]
    fn too_large_buffer_request() {
        const HOST: &str = "localhost:8080";
        let listener = TcpListener::bind(HOST).unwrap();

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = Arc::clone(&pair);

        let _thread = thread::spawn(move || {
            let (lock, cvar) = &*pair2;
            let mut started = lock.lock().unwrap();

            while !*started {
                started = cvar.wait(started).unwrap();
            }

            //send data
            let mut stream = TcpStream::connect(HOST).unwrap();

            let _ = stream.write(b"GET /index.html HTTP/1.1");
        });

        {
            let mut start = pair.0.lock().unwrap();

            *start = true;

            pair.1.notify_one()
        }

        {
            let mut stream = listener.incoming().next().unwrap().unwrap();

            let actual = parser::parser::<10, 100, 20, 250>(&mut stream);

            assert_eq!(Err(ResponseStatusCode::PayloadTooLarge), actual);
        }
    }

    #[test]
    #[serial]
    fn invalid_format_request() {
        const HOST: &str = "localhost:8080";
        let listener = TcpListener::bind(HOST).unwrap();

        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = Arc::clone(&pair);

        let _thread = thread::spawn(move || {
            let (lock, cvar) = &*pair2;
            let mut started = lock.lock().unwrap();

            while !*started {
                started = cvar.wait(started).unwrap();
            }

            //send data
            let mut stream = TcpStream::connect(HOST).unwrap();

            let _ = stream.write(b"GET /index.html");
        });

        {
            let mut start = pair.0.lock().unwrap();

            *start = true;

            pair.1.notify_one()
        }

        {
            let mut stream = listener.incoming().next().unwrap().unwrap();

            let actual = parser::parser::<100, 500, 20, 250>(&mut stream);

            assert_eq!(Err(ResponseStatusCode::BadRequest), actual);
        }
    }
}
