use std::{
    io::Write,
    net::TcpStream,
    sync::{
        mpsc::{self, Sender, Receiver},
        Arc, Mutex, RwLock,
    },
    thread::{self, JoinHandle}, collections::HashMap,
};

use cyclic_data_types::{list::List, error::Error};
use log::{error, trace};

use crate::{
    http::{
        request::Request,
        response::{response_status_code::ResponseStatusCode, Response},
    },
    setting::ServerSetting,
};

use super::super::{
    component::Component,
    pipeline::{
        ActionComponent, ActionQueue, Bytes, CompressionComponent, CompressionQueue,
        ParserComponent, Pipeline, SenderComponent, SenderQueue,
    },
};

type ParserFunc = fn(&mut TcpStream) -> Result<Request, ResponseStatusCode>;
type ActionFunc<U> = fn(
    &Result<Request, ResponseStatusCode>,
    &ServerSetting,
    utility_thread: &mut mpsc::Sender<U>,
) -> Result<Response, ResponseStatusCode>;
type CompressionFunc = fn(Response, Option<Request>, ServerSetting) -> Bytes;


#[derive(Clone)]
pub struct Builder<U: Clone> {
    pub parser: Option<ParserFunc>,
    pub action: Option<ActionFunc<U>>,
    pub compression: Option<CompressionFunc>,
    pub utility_sender: Option<mpsc::Sender<U>>,
    pub settings: Option<Arc<RwLock<ServerSetting>>>,
}

impl<U: Clone + Send + 'static> Builder<U> {
    pub fn set_parser(mut self, new_func: ParserFunc) -> Self {
        self.parser = Some(new_func);

        return self;
    }

    pub fn set_action(mut self, new_func: ActionFunc<U>) -> Self {
        self.action = Some(new_func);

        return self;
    }

    pub fn set_compression(mut self, new_func: CompressionFunc) -> Self {
        self.compression = Some(new_func);

        return self;
    }

    pub fn set_utility_thread(mut self, sender: mpsc::Sender<U>) -> Self {
        self.utility_sender = Some(sender);

        self
    }

    pub fn set_settings(mut self, settings: ServerSetting) -> Self {
        self.settings = Some(Arc::new(RwLock::new(settings)));

        self
    }

    pub fn build(&self) -> (Sender<TcpStream>, Pipeline) {
        //building components back to front to deal with input queue dependencies & ownership issues
        let (sender_queue, sender) = build_sender_component();

        //build compressor
        if self.compression.is_none() {
            todo!()
        }
        let (compressor_queue, compression) = build_compressor_component(
            self.compression.unwrap(),
            sender_queue,
            &self.settings.clone().unwrap(),
        );

        //build action
        if self.action.is_none() || self.utility_sender.is_none() || self.settings.is_none() {
            todo!()
        }
        let (action_queue, action) = build_action_component(
            self.action.unwrap(),
            compressor_queue,
            self.utility_sender.clone().unwrap(),
            &self.settings.clone().unwrap(),
        );

        //build parser
        if self.parser.is_none() {
            todo!()
        }
        let (new_conn, parser) = build_parser_component(self.parser.unwrap(), action_queue);

        //construct pipeline
        let pipeline = Pipeline {
            parser: parser,
            action: action,
            compression: compression,
            sender: sender
        };

        (new_conn, pipeline)
    }

    pub fn fix(&self, pipeline: &mut Pipeline) -> Result<(), ()> {
        if !pipeline.sender.thread_state() {
            error!("{pipeline} - Sender component panic");
            let input_queue = pipeline.sender.input_queue.clone();

            let new_thread = build_sender_thread(input_queue);

            pipeline.sender.swap_out_thread(new_thread);
        }

        if !pipeline.compression.thread_state() {
            error!("{pipeline} - Compression component panic");
            let input_queue = pipeline.compression.input_queue.clone();
            let output_queue = pipeline.sender.input_queue.clone();

            let new_thread = build_compressor_thread(self.compression.unwrap(), input_queue, output_queue, self.settings.clone().unwrap());

            pipeline.compression.swap_out_thread(new_thread);
        }

        if !pipeline.action.thread_state() {
            error!("{pipeline} - Action component panic");
            let input_queue = pipeline.action.input_queue.clone();
            let output_queue = pipeline.compression.input_queue.clone();

            let new_thread = build_action_thread(self.action.unwrap(), input_queue, output_queue, self.utility_sender.clone().unwrap(), self.settings.clone().unwrap());

            pipeline.action.swap_out_thread(new_thread);
        }

        if !pipeline.parser.thread_state() {
            error!("{pipeline} - Parser component panic");
            let input_queue = pipeline.parser.input_queue.clone();
            let output_queue = pipeline.action.input_queue.clone();

            let new_thread = build_parser_thread(self.parser.unwrap(), input_queue, output_queue);

            pipeline.parser.swap_out_thread(new_thread);
        }
        
        Ok(())
    }
}

impl<U: Clone + Send + 'static> Default for Builder<U> {
    fn default() -> Self {
        Self {
            parser: None,
            action: None,
            compression: None,
            utility_sender: None,
            settings: None,
        }
    }
}

// look into generic implementations
fn build_parser_component(
    parser: ParserFunc,
    output_queue: Arc<Mutex<ActionQueue>>,
) -> (Sender<TcpStream>, ParserComponent) {
    let (tx, rx) = mpsc::channel::<TcpStream>();

    let rx = Arc::new(Mutex::new(rx));

    let thread = build_parser_thread(parser, rx.clone(), output_queue);

    let component = Component::new(rx, thread);

    (tx, component)
}

fn build_parser_thread(
    parser: ParserFunc,
    input_queue: Arc<Mutex<Receiver<TcpStream>>>,
    output_queue: Arc<Mutex<ActionQueue>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
       // let rx = &*input_queue.lock().unwrap();

        loop {
            //get tcp stream
            let tcp_stream = {
                let rx = &*input_queue.lock().unwrap();
                
                rx.recv()
            };

            if let Err(_err) = tcp_stream {
                todo!();

                //continue;
            }

            let mut tcp_stream = tcp_stream.unwrap();

            //parse
            match parser(&mut tcp_stream) {
                Ok(val) => {
                    //send data
                    let _ = output_queue
                        .lock()
                        .unwrap()
                        .push_back((tcp_stream, Ok(val)));
                }
                Err(err) => {
                    error!("failed to parse: {}",err);

                    let _ = output_queue
                        .lock()
                        .unwrap()
                        .push_back((tcp_stream, Err(err)));
                },
            }
        }
    })
}

fn build_action_component<U: Send + 'static>(
    func: ActionFunc<U>,
    output_queue: Arc<Mutex<CompressionQueue>>,
    utility_access: mpsc::Sender<U>,
    settings: &Arc<RwLock<ServerSetting>>,
) -> (Arc<Mutex<ActionQueue>>, ActionComponent) {
    let input_queue = Arc::new(Mutex::new(ActionQueue::default()));

    let thread = build_action_thread(func, input_queue.clone(), output_queue, utility_access, settings.clone());

    let component = Component::new(input_queue.clone(), thread);

    (input_queue, component)
}

fn build_action_thread<U: Send + 'static>(
    func: ActionFunc<U>,
    input_queue: Arc<Mutex<ActionQueue>>,
    output_queue: Arc<Mutex<CompressionQueue>>,
    utility_access: mpsc::Sender<U>,
    server_settings: Arc<RwLock<ServerSetting>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut utility_access = utility_access;
        loop {

            let (stream, action_cmd) = match dequeue(&input_queue){
                Some(val) => val,
                None => continue
            };

            //action upon data
            let server_settings = (&*server_settings.read().unwrap()).clone();

            let response = match func(&action_cmd, &server_settings, &mut utility_access) {
                Ok(val) => val,
                Err(err) => {
                    match func(&Err(err), &server_settings, &mut utility_access) {
                        Ok(val) => val,
                        Err(_) => {
                            error!("Failed get error response");
                            Response{
                                status: err,
                                header: HashMap::new(),
                                body: None,
                            }
                        },
                    }
                },
            };

            match enqueue(output_queue.clone(), (stream, response, action_cmd.ok())) {
                Ok(_) => trace!("successful response generation"),
                Err(err) => error!("{err:?}"),
            }
        }
    })
}

fn build_compressor_component(
    func: CompressionFunc,
    output_queue: Arc<Mutex<SenderQueue>>,
    settings: &Arc<RwLock<ServerSetting>>,
) -> (Arc<Mutex<CompressionQueue>>, CompressionComponent) {
    let input_queue = Arc::new(Mutex::new(CompressionQueue::default()));

    let thread = build_compressor_thread(func, input_queue.clone(), output_queue, settings.clone());

    let component = Component::new(input_queue.clone(), thread);

    (input_queue, component)
}

fn build_compressor_thread(
    func: CompressionFunc,
    input_queue: Arc<Mutex<CompressionQueue>>,
    output_queue: Arc<Mutex<SenderQueue>>,
    server_settings: Arc<RwLock<ServerSetting>>
) -> JoinHandle<()> {
    thread::spawn(move || {
        let server_settings = (&*server_settings.read().unwrap()).clone();

        loop {
            //get first element in queue
            let (stream, response, request) = match dequeue(&input_queue) {
                Some(value) => value,
                None => continue,
            };

            trace!("Begin compression");

            //compress data and push to next pipe
            match enqueue(output_queue.clone(), (stream, func(response, request, server_settings.clone()))) {
                Ok(_) => {trace!("Begin compression");},
                Err(err) => error!("{err:?}"),
            }
        }
    })
}

fn build_sender_component() -> (Arc<Mutex<SenderQueue>>, SenderComponent) {
    let input_queue = Arc::new(Mutex::new(SenderQueue::default()));

    let thread = build_sender_thread(input_queue.clone());

    let component = Component::new(input_queue.clone(), thread);

    (input_queue, component)
}

fn build_sender_thread(
    input_queue: Arc<Mutex<SenderQueue>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            //get first element in queue
            let (mut stream, bytes) = match dequeue(&input_queue) {
                Some(value) => value,
                None => continue,
            };

            //send data
            if let Err(_err) = stream.write(&bytes) {
                todo!()
            }

            if let Err(_err) = stream.flush() {
                todo!()
            }
        }
    })
}

fn dequeue<const T: usize, U>(queue: &Arc<Mutex<List<T, U, false>>>) -> Option<U> {
    // functions ensures locking is localized to function rather than lifetime of loop
    // with the goal of minimizing chance of poisoning lock
    let mut input_queue = match queue.try_lock() {
        Ok(val) => val,
        Err(_) => return None,
    };

    let input_queue = &mut *input_queue;

    //get first element in queue
    input_queue.remove_front()
}

fn enqueue<const T: usize, U>(queue: Arc<Mutex<List<T, U, false>>>, value: U) -> Result<(), Error> {
    // functions ensures locking is localized to function rather than lifetime of loop
    // with the goal of minimizing chance of poisoning lock
    let mut queue = queue.lock();

    let result = match &mut queue {
        Ok(queue) => {
            queue.push_back(value)
        },
        Err(_) => todo!(),
    };

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}