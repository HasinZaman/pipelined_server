

// default no compression pipeline test

use std::{thread::{self, JoinHandle}, collections::HashMap, net::TcpStream, io::{Write, Read}, sync::{Arc, Mutex, Condvar, mpsc::Sender}, time::Duration, path::PathBuf};

use log::trace;
use serial_test::serial;

use crate::{
    pipeline::{
        builder::Builder,
        default::{self, action::{generate_file_utility_thread, FileUtilitySender}}, Server
    },
    setting::{
        ServerSetting,
        DomainPath
    },
    test_tools::file_env::FileEnv, file::FileError, http::{request::Request, response::{response_status_code::ResponseStatusCode, Response}}, logging::logger_init
};

const ADDRESS: &str = "localhost";
const PORT: u16 = 8080;

fn server_initialization(
    trigger_cond: Arc<(Mutex<bool>, Condvar)>,
    parser: fn(&mut TcpStream) -> Result<Request, ResponseStatusCode>,
    action: fn(&Result<Request, ResponseStatusCode>, ServerSetting, &mut Sender<(PathBuf, Sender<Result<Vec<u8>, FileError>>)>) -> Result<Response, ResponseStatusCode>,
    compression: fn(Response, Option<Request>, ServerSetting) -> Vec<u8>,
    utility_thread:  (Sender<(PathBuf, Sender<Result<Vec<u8>, FileError>>)>, JoinHandle<()>)
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
                    DomainPath{
                        path: String::from(""),
                        allow: vec![String::from("html")],
                    }
                );
                tmp
            }
        };

        trace!("Setting initialized ⚙️");

        //use builder to make server
        let builder = Builder::default()
            .set_settings(setting.clone())
            .set_parser(parser)
            .set_action(action)
            .set_compression(compression)
            .set_utility_thread(utility_thread.0.clone());

        let server = Server::new(
            setting,
            utility_thread,
            builder
        );

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

#[test]
#[serial]
fn default_one_request_one_pipeline() {
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
            let data = default::parser::<64,1024>(stream);
            trace!("Finished parsing 📄🔍\n{:?}", data);

            data
        },
        |request: &Result<Request, ResponseStatusCode>, setting: ServerSetting, utility_thread: &mut FileUtilitySender<FileError>| {
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
        generate_file_utility_thread()
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

        assert_eq!(response, "HTTP/1.1 200 Ok\r\nContent-Length: 11\r\nContent-Type: text/html\r\n\r\nhello_world");
    }
}

#[test]
#[serial]
fn default_one_request_four_pipeline() {
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
            let data = default::parser::<64,1024>(stream);
            trace!("Finished parsing 📄🔍\n{:?}", data);

            data
        },
        |request: &Result<Request, ResponseStatusCode>, setting: ServerSetting, utility_thread: &mut FileUtilitySender<FileError>| {
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
        generate_file_utility_thread()
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

        assert_eq!(response, "HTTP/1.1 200 Ok\r\nContent-Length: 11\r\nContent-Type: text/html\r\n\r\nhello_world");
    }
}

#[test]
#[serial]
fn default_four_request_one_pipeline() {
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
            let data = default::parser::<64,1024>(stream);
            trace!("Finished parsing 📄🔍\n{:?}", data);

            data
        },
        |request: &Result<Request, ResponseStatusCode>, setting: ServerSetting, utility_thread: &mut FileUtilitySender<FileError>| {
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
        generate_file_utility_thread()
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
        let mut streams:[TcpStream; 4] = core::array::from_fn(|_| TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).unwrap());

        for i in 0..4 {
            trace!("Request {} sent 💽 📃💨 💻", i+1);
            let _ = streams[i].write(format!("GET request_{}.html HTTP/1.1\n\rhost:localhost", i + 1).as_bytes());
        }
        for i in 0..4 {
    
    
            assert!(!server_thread.is_finished());
    
            let mut data = [0; 128];
            streams[i].read(&mut data).unwrap();
            trace!("Response {} received 💻 📃💨 💽", i+1);
    
            let response = String::from_utf8(data.to_vec());
    
            assert!(response.is_ok());
    
            let response = response.unwrap();
            let response = response.trim_end_matches("\0");
    
            trace!("{}", response);
    
            assert_eq!(response, format!("HTTP/1.1 200 Ok\r\nContent-Length: 9\r\nContent-Type: text/html\r\n\r\nrequest {}", i + 1));
        }
    }
}