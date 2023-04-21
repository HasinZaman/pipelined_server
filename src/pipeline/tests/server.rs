use std::{
    collections::HashMap,
    net::TcpStream,
    path::PathBuf,
    sync::{mpsc::{Sender}, Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};
use cyclic_data_types::list::List;
use log::{trace};

use crate::{
    file::FileError,
    http::{
        request::Request,
        response::{response_status_code::ResponseStatusCode, Response},
    },
    pipeline::{
        builder::pipeline::Builder,
        Server,
    },
    setting::{DomainPath, ServerSetting},
};

const ADDRESS: &str = "localhost";
const PORT: u16 = 8080;

fn server_initialization(
    trigger_cond: Arc<(Mutex<bool>, Condvar)>,
    parser: fn(&mut TcpStream) -> Result<Request, ResponseStatusCode>,
    action: fn(
        &Result<Request, ResponseStatusCode>,
        &ServerSetting,
        &mut Sender<(PathBuf, Sender<Result<Vec<u8>, FileError>>)>,
    ) -> Result<Response, ResponseStatusCode>,
    compression: fn(Response, Option<Request>, ServerSetting) -> Vec<u8>,
    utility_thread: (
        Sender<(PathBuf, Sender<Result<Vec<u8>, FileError>>)>,
        JoinHandle<()>,
    ),
) -> JoinHandle<()> {
    thread::spawn(move || {
        let utility_thread = utility_thread;
        trace!("Utility thread created 🛠️");

        let setting = ServerSetting {
            address: ADDRESS.to_string(),
            port: PORT,
            paths: {
                let mut tmp = HashMap::new();

                tmp.insert(
                    String::from("localhost"),
                    DomainPath {
                        path: String::from(""),
                        allow: vec![String::from("html")],
                    },
                );
                tmp
            },
        };

        trace!("Setting initialized ⚙️");

        //use builder to make server
        let builder = Builder::default()
            .set_settings(setting.clone())
            .set_parser(parser)
            .set_action(action)
            .set_compression(compression)
            .set_utility_thread(utility_thread.0.clone());

        let server = Server::new(setting, utility_thread, builder);

        trace!("Server built 💽🔨");

        {
            let mut start = trigger_cond.0.lock().unwrap();

            *start = true;

            trigger_cond.1.notify_one()
        }

        trace!("Server starting 💽🏃‍♂️");
        server.run::<1>();
    })
}

fn create_mb_string(n: usize) -> String {
    let mb_size = n * 1024 * 1024; // convert MB to bytes
    let mut s = String::with_capacity(mb_size);
    for _ in 0..mb_size {
        s.push('a');
    }
    s
}

fn split_bytes_at_body<'a>(vec: &'a Vec<u8>) -> Option<(&'a[u8], &'a[u8])> {
    let iter = vec.iter().enumerate();
    let mut buf: List<4, u8, true> = List::default();
    let target: List<4, u8, true> = [13, 10, 13, 10].into();

    for (idx, &byte) in iter {

        let _ = buf.push_back(byte);

        if target == buf {
            return Some(vec.split_at(idx + 1));
        }
    }

    None
}
mod request_test{
    mod one_pipeline{
        use std::{
            io::{Read, Write},
            net::TcpStream,
            sync::{Arc, Condvar, Mutex},
        };
        use log::{trace};
        use serial_test::serial;
        use crate::{
            file::FileError,
            http::{
                request::Request,
                response::{response_status_code::ResponseStatusCode, Response},
            },
            pipeline::{
                default::{
                    self,
                    action::{generate_read_only_file_utility_thread, FileUtilitySender, NO_BOUND},
                }, tests::server::{server_initialization, ADDRESS, PORT},
            },
            setting::{ServerSetting},
            test_tools::file_env::FileEnv,
        };

        #[test]
        #[serial]
        fn default_one() {
            //logger_init();

            // create file environment
            let _file_1 = FileEnv::new("source\\file_1.html", "hello_world");

            trace!("File environment created 📁");

            let pair = Arc::new((Mutex::new(false), Condvar::new()));

            // start server in separated thread
            let server_thread = server_initialization(
                Arc::clone(&pair),
                |stream: &mut TcpStream| {
                    trace!("Starting parsing 📄🔍");
                    let data = default::parser::<64, 1024>(stream);
                    trace!("Finished parsing 📄🔍\n{:?}", data);

                    data
                },
                |request: &Result<Request, ResponseStatusCode>,
                setting: &ServerSetting,
                utility_thread: &mut FileUtilitySender<FileError>| {
                    trace!("Staring action 💪");
                    let data = default::action(request, setting, utility_thread);
                    trace!("Finished action 💪\n{:?}", data);

                    data
                },
                |response: Response, request: Option<Request>, settings: ServerSetting| {
                    trace!("Staring compression 💥");
                    let data = default::no_compression(response, request, settings);
                    trace!("Finished compression 💥");

                    data
                },
                generate_read_only_file_utility_thread::<NO_BOUND>(),
            );

            {
                let (lock, cvar) = &*pair;
                let mut started = lock.lock().unwrap();

                while !*started {
                    started = cvar.wait(started).unwrap();
                }
            }
            trace!("Server test initiated 🤞");

            // send request
            {
                let mut stream = TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap();

                let _ = stream.write(b"GET file_1.html HTTP/1.1\n\rhost:localhost");

                trace!("Request sent 💽 📃💨 💻");

                assert!(!server_thread.is_finished());

                let mut data = [0; 128];
                stream.read(&mut data).unwrap();
                trace!("Response received 💻 📃💨 💽");

                let response = String::from_utf8(data.to_vec());

                assert!(response.is_ok());

                let response = response.unwrap();
                let response = response.trim_end_matches("\0");

                trace!("{}", response);

                assert_eq!(
                    response,
                    "HTTP/1.1 200 Ok\r\nContent-Length: 11\r\nContent-Type: text/html\r\n\r\nhello_world"
                );
            }
        }

        #[test]
        #[serial]
        fn default_four() {
            //logger_init();

            // create file environment
            let _file_1 = FileEnv::new("source\\request_1.html", "request 1");
            let _file_2 = FileEnv::new("source\\request_2.html", "request 2");
            let _file_3 = FileEnv::new("source\\request_3.html", "request 3");
            let _file_4 = FileEnv::new("source\\request_4.html", "request 4");

            trace!("File environment created 📁");

            let pair = Arc::new((Mutex::new(false), Condvar::new()));

            // start server in separated thread
            let server_thread = server_initialization(
                Arc::clone(&pair),
                |stream: &mut TcpStream| {
                    trace!("Starting parsing 📄🔍");
                    let data = default::parser::<64, 1024>(stream);
                    trace!("Finished parsing 📄🔍\n{:?}", data);

                    data
                },
                |request: &Result<Request, ResponseStatusCode>,
                setting: &ServerSetting,
                utility_thread: &mut FileUtilitySender<FileError>| {
                    trace!("Staring action 💪");
                    let data = default::action(request, setting, utility_thread);
                    trace!("Finished action 💪\n{:?}", data);

                    data
                },
                |response: Response, request: Option<Request>, settings: ServerSetting| {
                    trace!("Staring compression 💥");
                    let data = default::no_compression(response, request, settings);
                    trace!("Finished compression 💥");

                    data
                },
                generate_read_only_file_utility_thread::<NO_BOUND>(),
            );

            {
                let (lock, cvar) = &*pair;
                let mut started = lock.lock().unwrap();

                while !*started {
                    started = cvar.wait(started).unwrap();
                }
            }
            trace!("Server test initiated 🤞");

            // send request
            {
                let mut streams: [TcpStream; 4] =
                    core::array::from_fn(|_| TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap());

                for i in 0..4 {
                    trace!("Request {} sent 💽 📃💨 💻", i + 1);
                    let _ = streams[i]
                        .write(format!("GET request_{}.html HTTP/1.1\n\rhost:localhost", i + 1).as_bytes());
                }
                for i in 0..4 {
                    assert!(!server_thread.is_finished());

                    let mut data = [0; 128];
                    streams[i].read(&mut data).unwrap();
                    trace!("Response {} received 💻 📃💨 💽", i + 1);

                    let response = String::from_utf8(data.to_vec());

                    assert!(response.is_ok());

                    let response = response.unwrap();
                    let response = response.trim_end_matches("\0");

                    trace!("{}", response);

                    assert_eq!(response, format!("HTTP/1.1 200 Ok\r\nContent-Length: 9\r\nContent-Type: text/html\r\n\r\nrequest {}", i + 1));
                }
            }
        }

    }

    mod four_pipeline{
        use std::{
            io::{Read, Write},
            net::TcpStream,
            sync::{Arc, Condvar, Mutex, RwLock},
            thread::{self, JoinHandle},
        };
        use log::{trace};
        use serial_test::serial;
        use crate::{
            file::FileError,
            http::{
                request::Request,
                response::{response_status_code::ResponseStatusCode, Response},
            },
            logging::logger_init,
            pipeline::{
                default::{
                    self,
                    action::{generate_read_only_file_utility_thread, FileUtilitySender, NO_BOUND},
                }, tests::server::{server_initialization, ADDRESS, PORT, create_mb_string},
            },
            setting::{ServerSetting},
            test_tools::file_env::FileEnv,
        };

        #[test]
        #[serial]
        fn default_one() {
            //logger_init();

            // create file environment
            let _file_1 = FileEnv::new("source\\file_1.html", "hello_world");

            trace!("File environment created 📁");

            let pair = Arc::new((Mutex::new(false), Condvar::new()));

            // start server in separated thread
            let server_thread = server_initialization(
                Arc::clone(&pair),
                |stream: &mut TcpStream| {
                    trace!("Starting parsing 📄🔍");
                    let data = default::parser::<64, 1024>(stream);
                    trace!("Finished parsing 📄🔍\n{:?}", data);

                    data
                },
                |request: &Result<Request, ResponseStatusCode>,
                setting: &ServerSetting,
                utility_thread: &mut FileUtilitySender<FileError>| {
                    trace!("Staring action 💪");
                    let data = default::action(request, setting, utility_thread);
                    trace!("Finished action 💪\n{:?}", data);

                    data
                },
                |response: Response, request: Option<Request>, settings: ServerSetting| {
                    trace!("Staring compression 💥");
                    let data = default::no_compression(response, request, settings);
                    trace!("Finished compression 💥");

                    data
                },
                generate_read_only_file_utility_thread::<NO_BOUND>(),
            );

            {
                let (lock, cvar) = &*pair;
                let mut started = lock.lock().unwrap();

                while !*started {
                    started = cvar.wait(started).unwrap();
                }
            }
            trace!("Server test initiated 🤞");

            // send request
            {
                let mut stream = TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap();

                let _ = stream.write(b"GET file_1.html HTTP/1.1\n\rhost:localhost");

                trace!("Request sent 💽 📃💨 💻");

                assert!(!server_thread.is_finished());

                let mut data = [0; 128];
                stream.read(&mut data).unwrap();
                trace!("Response received 💻 📃💨 💽");

                let response = String::from_utf8(data.to_vec());

                assert!(response.is_ok());

                let response = response.unwrap();
                let response = response.trim_end_matches("\0");

                trace!("{}", response);

                assert_eq!(
                    response,
                    "HTTP/1.1 200 Ok\r\nContent-Length: 11\r\nContent-Type: text/html\r\n\r\nhello_world"
                );
            }
        }

        #[test]
        #[serial]
        fn default_eight() {
            //logger_init();

            // create file environment
            let _file_1 = FileEnv::new("source\\request_1.html", "request 1");
            let _file_2 = FileEnv::new("source\\request_2.html", "request 2");
            let _file_3 = FileEnv::new("source\\request_3.html", "request 3");
            let _file_4 = FileEnv::new("source\\request_4.html", "request 4");

            trace!("File environment created 📁");

            let pair = Arc::new((Mutex::new(false), Condvar::new()));

            // start server in separated thread
            let _server_thread = server_initialization(
                Arc::clone(&pair),
                |stream: &mut TcpStream| {
                    trace!("Starting parsing 📄🔍");
                    let data = default::parser::<64, 1024>(stream);
                    trace!("Finished parsing 📄🔍\n{:?}", data);

                    data
                },
                |request: &Result<Request, ResponseStatusCode>,
                setting: &ServerSetting,
                utility_thread: &mut FileUtilitySender<FileError>| {
                    trace!("Staring action 💪");
                    let data = default::action(request, setting, utility_thread);
                    trace!("Finished action 💪\n{:?}", data);

                    data
                },
                |response: Response, request: Option<Request>, settings: ServerSetting| {
                    trace!("Staring compression 💥");
                    let data = default::no_compression(response, request, settings);
                    trace!("Finished compression 💥");

                    data
                },
                generate_read_only_file_utility_thread::<NO_BOUND>(),
            );

            {
                let (lock, cvar) = &*pair;
                let mut started = lock.lock().unwrap();

                while !*started {
                    started = cvar.wait(started).unwrap();
                }
            }
            trace!("Server test initiated 🤞");

            // send request
            {
                //
                let mut streams:Vec<JoinHandle<()>> = (0..8)
                    .map(
                        |i| {
                            let mut stream = TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap();
                            
                            trace!("Request {} sent 💽 📃💨 💻", i+1);
                            let _ = stream.write(format!("GET request_{}.html HTTP/1.1\n\rhost:localhost", (i%4) + 1).as_bytes());

                            thread::spawn(
                                move|| {
                                    let mut data = [0; 128];
                                    stream.read(&mut data).unwrap();
                                    trace!("Response {} received 💻 📃💨 💽", i+1);
                            
                                    let response = String::from_utf8(data.to_vec());
                            
                                    assert!(response.is_ok());
                            
                                    let response = response.unwrap();
                                    let response = response.trim_end_matches("\0");
                            
                                    trace!("{}", response);
                            
                                    assert_eq!(response, format!("HTTP/1.1 200 Ok\r\nContent-Length: 9\r\nContent-Type: text/html\r\n\r\nrequest {}", (i%4) + 1));
                                }
                            )
                        }
                    ).collect();
                while streams.len() > 0 {
                    for i in (0..streams.len()).rev() {
                        if streams[i].is_finished() {
                            let steam = streams.pop().unwrap();

                            match steam.join() {
                                Ok(_) => assert!(true),
                                Err(err) => {
                                    println!("{:?}", err);
                                    assert!(false);
                                }
                            }
                        }
                    }
                }
            }
        }

        #[test]
        #[serial]
        fn default_eight_two_file() {
        logger_init();

        let file_1_content = Arc::new(RwLock::new(create_mb_string(100)));
        let file_2_content = Arc::new(RwLock::new(create_mb_string(10)));

        // create file environment
        let _file_1 = FileEnv::new("source\\request_1.html", &*file_1_content.read().unwrap());
        let _file_2 = FileEnv::new("source\\request_2.html", &*file_2_content.read().unwrap());

        trace!("File environment created 📁");

        let pair = Arc::new((Mutex::new(false), Condvar::new()));

        // start server in separated thread
        let _server_thread = server_initialization(
            Arc::clone(&pair),
            |stream: &mut TcpStream| {
                trace!("Starting parsing 📄🔍");
                let data = default::parser::<64, 1024>(stream);
                trace!("Finished parsing 📄🔍\n{:?}", data);

                data
            },
            |request: &Result<Request, ResponseStatusCode>,
            setting: &ServerSetting,
            utility_thread: &mut FileUtilitySender<FileError>| {
                trace!("Staring action 💪");
                let data = default::action(request, setting, utility_thread);
                trace!("Finished action 💪");

                data
            },
            |response: Response, request: Option<Request>, settings: ServerSetting| {
                trace!("Staring compression 💥");
                let data = default::no_compression(response, request, settings);
                trace!("Finished compression 💥");

                data
            },
            generate_read_only_file_utility_thread::<NO_BOUND>(),
        );

        {
            let (lock, cvar) = &*pair;
            let mut started = lock.lock().unwrap();

            while !*started {
                started = cvar.wait(started).unwrap();
            }
        }
        trace!("Server test initiated 🤞");

        // send request
        {
            let mut streams:Vec<JoinHandle<()>> = (0..8)
                .map(
                    |i| {
                        let mut stream = TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap();
                        
                        trace!("Request {} sent 💽 📃💨 💻", i+1);
                        let _ = stream.write(format!("GET request_{}.html HTTP/1.1\n\rhost:localhost", (i%2) + 1).as_bytes());

                        let file_1_content = file_1_content.clone();
                        let file_2_content = file_2_content.clone();

                        thread::spawn(
                            move|| {
                                let file_1_content = &*file_1_content.read().unwrap();
                                let file_2_content = &*file_2_content.read().unwrap();
                                let mut data_buffer = [0; 128];
                                let mut response = String::new();
                                while let Ok(size) = stream.read(&mut data_buffer) {
                                    if size == 0 {
                                        // Stop the TCP listener when the stream stops receiving data
                                        break;
                                    }
                                    let data = &data_buffer[..size];

                                    let data_str = String::from_utf8(data.to_vec()).unwrap();
                                    response.push_str(&data_str.trim_end_matches('\0'));
                                }
                                trace!("Response {} received 💻 📃💨 💽", i+1);
                        
                                match i%2 {
                                    0 => {
                                        assert_eq!(response, format!("HTTP/1.1 200 Ok\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}", file_1_content.len(), file_1_content));
                                    },
                                    1 => {
                                        assert_eq!(response, format!("HTTP/1.1 200 Ok\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}", file_2_content.len(), file_2_content));
                                    },
                                    _ => panic!()
                                }
                            }
                        )
                    }
                ).collect();
            while streams.len() > 0 {
                for i in (0..streams.len()).rev() {
                    if streams[i].is_finished() {
                        let steam = streams.pop().unwrap();

                        match steam.join() {
                            Ok(_) => {
                                assert!(true);
                            },
                            Err(err) => {
                                println!("{:?}", err);
                                assert!(false);
                            }
                        }
                    }
                }
            }
        }
    }

    }
}

mod compression_test{
    use std::{
        io::{Read, Write},
        net::TcpStream,
        sync::{Arc, Condvar, Mutex},
    };
    use flate2::{read::{GzDecoder, DeflateDecoder, ZlibDecoder}};
    use log::{trace};
    use serial_test::serial;

    use crate::{test_tools::file_env::FileEnv, pipeline::{tests::server::{server_initialization, ADDRESS, PORT, split_bytes_at_body}, default::{self, action::{FileUtilitySender, generate_read_only_file_utility_thread, NO_BOUND}}}, http::{request::Request, response::{response_status_code::ResponseStatusCode, Response}}, setting::ServerSetting, file::FileError};
    
    const FILE_1_CONTENT: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz";


    #[test]
    #[serial]
    fn no_compression() {
        // create file environment
        let _file_1 = FileEnv::new("source\\file_1.html", &FILE_1_CONTENT);
        trace!("File environment created 📁");

        let pair = Arc::new((Mutex::new(false), Condvar::new()));

        // start server in separated thread
        let _server_thread = server_initialization(
            Arc::clone(&pair),
            |stream: &mut TcpStream| {
                trace!("Starting parsing 📄🔍");
                let data = default::parser::<64, 1024>(stream);
                trace!("Finished parsing 📄🔍\n{:?}", data);

                data
            },
            |request: &Result<Request, ResponseStatusCode>,
            setting: &ServerSetting,
            utility_thread: &mut FileUtilitySender<FileError>| {
                trace!("Staring action 💪");
                let data = default::action(request, setting, utility_thread);
                trace!("Finished action 💪");

                data
            },
            |response: Response, request: Option<Request>, settings: ServerSetting| {
                trace!("Staring compression 💥");
                let data = default::compression(response, request, settings);
                trace!("Finished compression 💥");

                data
            },
            generate_read_only_file_utility_thread::<NO_BOUND>(),
        );

        {
            let (lock, cvar) = &*pair;
            let mut started = lock.lock().unwrap();

            while !*started {
                started = cvar.wait(started).unwrap();
            }
        }
        trace!("Server test initiated 🤞");

        // send request
        {
            let mut stream = TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap();
            assert!(stream.write(b"GET file_1.html HTTP/1.1\n\rhost:localhost").is_ok());

            let mut data_buffer = [0; 128];
            let mut response = Vec::new();
            while let Ok(size) = stream.read(&mut data_buffer) {
                if size == 0 {
                    // Stop the TCP listener when the stream stops receiving data
                    break;
                }
                let data = &data_buffer[..size];

                response.append(&mut data.into());
            }
            trace!("Response gzip received 💻 📃💨 💽");
    
            assert_eq!(
                response,
                format!("HTTP/1.1 200 Ok\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}", FILE_1_CONTENT.len(),FILE_1_CONTENT).as_bytes()
            );
        }
    }

    #[test]
    #[serial]
    fn gzip() {
        // create file environment
        let _file_1 = FileEnv::new("source\\file_1.html", &FILE_1_CONTENT);
        trace!("File environment created 📁");

        let pair = Arc::new((Mutex::new(false), Condvar::new()));

        // start server in separated thread
        let _server_thread = server_initialization(
            Arc::clone(&pair),
            |stream: &mut TcpStream| {
                trace!("Starting parsing 📄🔍");
                let data = default::parser::<64, 1024>(stream);
                trace!("Finished parsing 📄🔍\n{:?}", data);

                data
            },
            |request: &Result<Request, ResponseStatusCode>,
            setting: &ServerSetting,
            utility_thread: &mut FileUtilitySender<FileError>| {
                trace!("Staring action 💪");
                let data = default::action(request, setting, utility_thread);
                trace!("Finished action 💪");

                data
            },
            |response: Response, request: Option<Request>, settings: ServerSetting| {
                trace!("Staring compression 💥");
                let data = default::compression(response, request, settings);
                trace!("Finished compression 💥");

                data
            },
            generate_read_only_file_utility_thread::<NO_BOUND>(),
        );

        {
            let (lock, cvar) = &*pair;
            let mut started = lock.lock().unwrap();

            while !*started {
                started = cvar.wait(started).unwrap();
            }
        }
        trace!("Server test initiated 🤞");

        // send request
        {
            let mut stream = TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap();
            assert!(stream.write(b"GET file_1.html HTTP/1.1\n\rhost:localhost\n\rAccept-Encoding: gzip").is_ok());

            let mut data_buffer = [0; 128];
            let mut response = Vec::new();
            while let Ok(size) = stream.read(&mut data_buffer) {
                if size == 0 {
                    // Stop the TCP listener when the stream stops receiving data
                    break;
                }
                let data = &data_buffer[..size];

                response.append(&mut data.into());
            }
            trace!("Response gzip received 💻 📃💨 💽");
    
            let (header, body) = split_bytes_at_body(&response).unwrap();

            let response_body = {
                let mut decoder = GzDecoder::new(body);
                    
                let mut decoded_data: Vec<u8> = Vec::new();
                
                assert!(decoder.read(&mut decoded_data).is_ok());

                decoded_data
                // let response_body = String::from_utf8(decoded_data);

                // assert!(response_body.is_ok());

                // response_body.unwrap()
            };

            let mut response = header.to_vec();
            response.append(&mut response_body.clone());

            assert_eq!(
                response,
                format!("HTTP/1.1 200 Ok\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}", FILE_1_CONTENT.len(),FILE_1_CONTENT).as_bytes()
            );
        }
    }

    #[test]
    #[serial]
    fn deflate() {
        // create file environment
        let _file_1 = FileEnv::new("source\\file_1.html", &FILE_1_CONTENT);
        trace!("File environment created 📁");

        let pair = Arc::new((Mutex::new(false), Condvar::new()));

        // start server in separated thread
        let _server_thread = server_initialization(
            Arc::clone(&pair),
            |stream: &mut TcpStream| {
                trace!("Starting parsing 📄🔍");
                let data = default::parser::<64, 1024>(stream);
                trace!("Finished parsing 📄🔍\n{:?}", data);

                data
            },
            |request: &Result<Request, ResponseStatusCode>,
            setting: &ServerSetting,
            utility_thread: &mut FileUtilitySender<FileError>| {
                trace!("Staring action 💪");
                let data = default::action(request, setting, utility_thread);
                trace!("Finished action 💪");

                data
            },
            |response: Response, request: Option<Request>, settings: ServerSetting| {
                trace!("Staring compression 💥");
                let data = default::compression(response, request, settings);
                trace!("Finished compression 💥");

                data
            },
            generate_read_only_file_utility_thread::<NO_BOUND>(),
        );

        {
            let (lock, cvar) = &*pair;
            let mut started = lock.lock().unwrap();

            while !*started {
                started = cvar.wait(started).unwrap();
            }
        }
        trace!("Server test initiated 🤞");

        // send request
        {
            let mut stream = TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap();
            assert!(stream.write(b"GET file_1.html HTTP/1.1\n\rhost:localhost\n\rAccept-Encoding: deflate").is_ok());

            let mut data_buffer = [0; 128];
            let mut response = Vec::new();
            while let Ok(size) = stream.read(&mut data_buffer) {
                if size == 0 {
                    // Stop the TCP listener when the stream stops receiving data
                    break;
                }
                let data = &data_buffer[..size];

                response.append(&mut data.into());
            }
            trace!("Response gzip received 💻 📃💨 💽");
    
            let (header, body) = split_bytes_at_body(&response).unwrap();

            let response_body = {
                let mut decoder = DeflateDecoder::new(body);
                    
                let mut decoded_data: Vec<u8> = Vec::new();
                
                assert!(decoder.read(&mut decoded_data).is_ok());

                decoded_data
                // let response_body = String::from_utf8(decoded_data);

                // assert!(response_body.is_ok());

                // response_body.unwrap()
            };

            let mut response = header.to_vec();
            response.append(&mut response_body.clone());

            assert_eq!(
                response,
                format!("HTTP/1.1 200 Ok\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}", FILE_1_CONTENT.len(),FILE_1_CONTENT).as_bytes()
            );
        }
    }

    #[test]
    #[serial]
    fn zlib() {
        // create file environment
        let _file_1 = FileEnv::new("source\\file_1.html", &FILE_1_CONTENT);
        trace!("File environment created 📁");

        let pair = Arc::new((Mutex::new(false), Condvar::new()));

        // start server in separated thread
        let _server_thread = server_initialization(
            Arc::clone(&pair),
            |stream: &mut TcpStream| {
                trace!("Starting parsing 📄🔍");
                let data = default::parser::<64, 1024>(stream);
                trace!("Finished parsing 📄🔍\n{:?}", data);

                data
            },
            |request: &Result<Request, ResponseStatusCode>,
            setting: &ServerSetting,
            utility_thread: &mut FileUtilitySender<FileError>| {
                trace!("Staring action 💪");
                let data = default::action(request, setting, utility_thread);
                trace!("Finished action 💪");

                data
            },
            |response: Response, request: Option<Request>, settings: ServerSetting| {
                trace!("Staring compression 💥");
                let data = default::compression(response, request, settings);
                trace!("Finished compression 💥");

                data
            },
            generate_read_only_file_utility_thread::<NO_BOUND>(),
        );

        {
            let (lock, cvar) = &*pair;
            let mut started = lock.lock().unwrap();

            while !*started {
                started = cvar.wait(started).unwrap();
            }
        }
        trace!("Server test initiated 🤞");

        // send request
        {
            let mut stream = TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap();
            assert!(stream.write(b"GET file_1.html HTTP/1.1\n\rhost:localhost\n\rAccept-Encoding: zlib").is_ok());

            let mut data_buffer = [0; 128];
            let mut response = Vec::new();
            while let Ok(size) = stream.read(&mut data_buffer) {
                if size == 0 {
                    // Stop the TCP listener when the stream stops receiving data
                    break;
                }
                let data = &data_buffer[..size];

                response.append(&mut data.into());
            }
            trace!("Response gzip received 💻 📃💨 💽");
    
            let (header, body) = split_bytes_at_body(&response).unwrap();

            let response_body = {
                let mut decoder = ZlibDecoder::new(body);
                    
                let mut decoded_data: Vec<u8> = Vec::new();
                
                assert!(decoder.read(&mut decoded_data).is_ok());

                decoded_data
                // let response_body = String::from_utf8(decoded_data);

                // assert!(response_body.is_ok());

                // response_body.unwrap()
            };

            let mut response = header.to_vec();
            response.append(&mut response_body.clone());

            assert_eq!(
                response,
                format!("HTTP/1.1 200 Ok\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}", FILE_1_CONTENT.len(),FILE_1_CONTENT).as_bytes()
            );
        }
    }
    
}
mod error_recovery_test{
    mod one_pipeline{ 
        use std::{
            io::{Read, Write},
            net::TcpStream,
            sync::{Arc, Condvar, Mutex},
        };
        
        use log::{trace, error};
        use serial_test::serial;
        
        use crate::{
            file::FileError,
            http::{
                request::Request,
                response::{response_status_code::ResponseStatusCode, Response}, method::Method,
            },
            logging::logger_init,
            pipeline::{
                default::{
                    self,
                    action::{generate_read_only_file_utility_thread, FileUtilitySender, NO_BOUND},
                }, tests::server::{server_initialization, ADDRESS, PORT},
            },
            setting::{ServerSetting},
            test_tools::file_env::FileEnv,
        };
        
        #[test]
        #[serial]
        fn default_parser() {
            logger_init();

            // create file environment
            let _file_1 = FileEnv::new("source\\request_1.html", "request 1");

            trace!("File environment created 📁");

            let pair = Arc::new((Mutex::new(false), Condvar::new()));

            // start server in separated thread
            let server_thread = server_initialization(
                Arc::clone(&pair),
                |stream: &mut TcpStream| {
                    trace!("Starting parsing 📄🔍");
                    let data = default::parser::<64, 1024>(stream);

                    if let Ok(Request(Method::Get { file }, _)) = &data {
                        if file == "request_2.html" {
                            error!("Parser Panic");
                            panic!("Simulated Panic") 
                        }
                    }

                    trace!("Finished parsing 📄🔍\n{:?}", data);

                    data
                },
                |request: &Result<Request, ResponseStatusCode>,
                setting: &ServerSetting,
                utility_thread: &mut FileUtilitySender<FileError>| {
                    trace!("Staring action 💪");
                    let data = default::action(request, setting, utility_thread);
                    trace!("Finished action 💪\n{:?}", data);

                    data
                },
                |response: Response, request: Option<Request>, settings: ServerSetting| {
                    trace!("Staring compression 💥");
                    let data = default::no_compression(response, request, settings);
                    trace!("Finished compression 💥");

                    data
                },
                generate_read_only_file_utility_thread::<NO_BOUND>(),
            );

            {
                let (lock, cvar) = &*pair;
                let mut started = lock.lock().unwrap();

                while !*started {
                    started = cvar.wait(started).unwrap();
                }
            }
            trace!("Server test initiated 🤞");

            // send request
            {
                let mut streams: [TcpStream; 3] =
                    core::array::from_fn(|_| TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap());

                for i in 0..3 {
                    trace!("Request {} sent 💽 📃💨 💻", i + 1);
                    match i {
                        1 => {
                            let _ = streams[i]
                                .write(b"GET request_2.html HTTP/1.1\n\rhost:localhost");
                        },
                        _ => {
                            let _ = streams[i]
                                .write(b"GET request_1.html HTTP/1.1\n\rhost:localhost");
                        }
                    }
                    
                }
                for i in 0..2 {
                    assert!(!server_thread.is_finished());

                    let mut data = [0; 128];
                    streams[i * 2].read(&mut data).unwrap();//skip streams[1] due to panic
                    trace!("Response {} received 💻 📃💨 💽", (i * 2) + 1);

                    let response = String::from_utf8(data.to_vec());

                    assert!(response.is_ok());

                    let response = response.unwrap();
                    let response = response.trim_end_matches("\0");

                    trace!("{}", response);

                    assert_eq!(response, format!("HTTP/1.1 200 Ok\r\nContent-Length: 9\r\nContent-Type: text/html\r\n\r\nrequest 1"));
                }
            }
        }

        #[test]
        #[serial]
        fn default_action() {
            logger_init();

            // create file environment
            let _file_1 = FileEnv::new("source\\request_1.html", "request 1");

            trace!("File environment created 📁");

            let pair = Arc::new((Mutex::new(false), Condvar::new()));

            // start server in separated thread
            let server_thread = server_initialization(
                Arc::clone(&pair),
                |stream: &mut TcpStream| {
                    trace!("Starting parsing 📄🔍");
                    let data = default::parser::<64, 1024>(stream);
                    trace!("Finished parsing 📄🔍\n{:?}", data);

                    data
                },
                |request: &Result<Request, ResponseStatusCode>,
                setting: &ServerSetting,
                utility_thread: &mut FileUtilitySender<FileError>| {
                    trace!("Staring action 💪");
                    if let Ok(Request(Method::Get { file }, _)) = request {
                        if file == "request_2.html" {
                            error!("Action Panic");
                            panic!("Simulated Panic") 
                        }
                    }
                    let data = default::action(request, setting, utility_thread);
                    trace!("Finished action 💪\n{:?}", data);

                    data
                },
                |response: Response, request: Option<Request>, settings: ServerSetting| {
                    trace!("Staring compression 💥");
                    let data = default::no_compression(response, request, settings);
                    trace!("Finished compression 💥");

                    data
                },
                generate_read_only_file_utility_thread::<NO_BOUND>(),
            );

            {
                let (lock, cvar) = &*pair;
                let mut started = lock.lock().unwrap();

                while !*started {
                    started = cvar.wait(started).unwrap();
                }
            }
            trace!("Server test initiated 🤞");

            // send request
            {
                let mut streams: [TcpStream; 3] =
                    core::array::from_fn(|_| TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap());

                for i in 0..3 {
                    trace!("Request {} sent 💽 📃💨 💻", i + 1);
                    match i {
                        1 => {
                            let _ = streams[i]
                                .write(b"GET request_2.html HTTP/1.1\n\rhost:localhost");
                        },
                        _ => {
                            let _ = streams[i]
                                .write(b"GET request_1.html HTTP/1.1\n\rhost:localhost");
                        }
                    }
                    
                }
                for i in 0..2 {
                    assert!(!server_thread.is_finished());

                    let mut data = [0; 128];
                    streams[i * 2].read(&mut data).unwrap();//skip streams[1] due to panic
                    trace!("Response {} received 💻 📃💨 💽", (i * 2) + 1);

                    let response = String::from_utf8(data.to_vec());

                    assert!(response.is_ok());

                    let response = response.unwrap();
                    let response = response.trim_end_matches("\0");

                    trace!("{}", response);

                    assert_eq!(response, format!("HTTP/1.1 200 Ok\r\nContent-Length: 9\r\nContent-Type: text/html\r\n\r\nrequest 1"));
                }
            }
        }

        #[test]
        #[serial]
        fn default_compressor() {
            logger_init();

            // create file environment
            let _file_1 = FileEnv::new("source\\request_1.html", "request 1");

            trace!("File environment created 📁");

            let pair = Arc::new((Mutex::new(false), Condvar::new()));

            // start server in separated thread
            let server_thread = server_initialization(
                Arc::clone(&pair),
                |stream: &mut TcpStream| {
                    trace!("Starting parsing 📄🔍");
                    let data = default::parser::<64, 1024>(stream);
                    trace!("Finished parsing 📄🔍\n{:?}", data);

                    data
                },
                |request: &Result<Request, ResponseStatusCode>,
                setting: &ServerSetting,
                utility_thread: &mut FileUtilitySender<FileError>| {
                    trace!("Staring action 💪");
                    let data = default::action(request, setting, utility_thread);
                    trace!("Finished action 💪\n{:?}", data);

                    data
                },
                |response: Response, request: Option<Request>, settings: ServerSetting| {
                    trace!("Staring compression 💥");
                    trace!("compression: {request:?}");
                    if None == request {
                        error!("Compressor Panic");
                        panic!("Simulated Panic")
                    }
                    let data = default::no_compression(response, request, settings);
                    trace!("Finished compression 💥");

                    data
                },
                generate_read_only_file_utility_thread::<NO_BOUND>(),
            );

            {
                let (lock, cvar) = &*pair;
                let mut started = lock.lock().unwrap();

                while !*started {
                    started = cvar.wait(started).unwrap();
                }
            }
            trace!("Server test initiated 🤞");

            // send request
            {
                let mut streams: [TcpStream; 3] =
                    core::array::from_fn(|_| TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap());

                for i in 0..3 {
                    trace!("Request {} sent 💽 📃💨 💻", i + 1);
                    match i {
                        1 => {
                            let _ = streams[i]
                                .write(b"GET request_2.html HTTP/1.1\n\rhost:localhost");
                        },
                        _ => {
                            let _ = streams[i]
                                .write(b"GET request_1.html HTTP/1.1\n\rhost:localhost");
                        }
                    }
                    
                }
                for i in 0..2 {
                    assert!(!server_thread.is_finished());

                    let mut data = [0; 128];
                    streams[i * 2].read(&mut data).unwrap();//skip streams[1] due to panic
                    trace!("Response {} received 💻 📃💨 💽", (i * 2) + 1);

                    let response = String::from_utf8(data.to_vec());

                    assert!(response.is_ok());

                    let response = response.unwrap();
                    let response = response.trim_end_matches("\0");

                    trace!("{}", response);

                    assert_eq!(response, format!("HTTP/1.1 200 Ok\r\nContent-Length: 9\r\nContent-Type: text/html\r\n\r\nrequest 1"));
                }
            }
        }
    }
}