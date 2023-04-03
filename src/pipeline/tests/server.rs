

// default no compression pipeline test

use std::{thread, collections::HashMap, net::TcpStream, io::{Write, Read}, sync::{Arc, Mutex, Condvar}, time::Duration};

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

#[test]
#[serial]
fn default_pipeline() {
    logger_init();

    // create file environment
    let _file_1 = FileEnv::new("source\\file_1.html", "hello_world");
    
    trace!("File environment created 📁");

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = Arc::clone(&pair);

    // start server in separated thread
    let server_thread = thread::spawn(move || {

        let utility_thread = generate_file_utility_thread();
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

        let test_parser = |stream: &mut TcpStream| {
            trace!("Starting parsing 📄🔍");
            let data = default::parser::<64,1024>(stream);
            trace!("Finished parsing 📄🔍\n{:?}", data);

            data
        };

        let test_action = |request: &Result<Request, ResponseStatusCode>, setting: ServerSetting, utility_thread: &mut FileUtilitySender<FileError>| {
            trace!("Staring action 💪");
            let data = default::action(request, setting, utility_thread);
            trace!("Finished action 💪\n{:?}", data);

            data
        };

        let test_no_compression = |response: Response, request: Option<Request>, settings: ServerSetting| {
            trace!("Staring compression 💥");
            let data = default::no_compression(response, request, settings);
            trace!("Finished compression 💥");
            
            data
        };

        //use builder to make server
        let builder = Builder::default()
            .set_settings(setting.clone())
            .set_parser(test_parser)
            .set_action(test_action)
            .set_compression(test_no_compression)
            .set_utility_thread(utility_thread.0.clone());

        let server = Server::new(
            setting,
            utility_thread,
            builder
        );

        trace!("Server built 💽🔨");

        {
            let mut start = pair.0.lock().unwrap();

            *start = true;

            pair.1.notify_one()
        }

        trace!("Server starting 💽🏃‍♂️");
        server.run();
    });

    {
        let (lock, cvar) = &*pair2;
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

        thread::sleep(Duration::from_secs(5));

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
fn default_server_server_side_test() {

}

#[test]
#[serial]
fn default_server_response_test() {
    
}