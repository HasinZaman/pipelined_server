use std::{
    io::Write,
    net::TcpStream,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex, RwLock,
    },
    thread::{self}, ops::Receiver,
};

use cyclic_data_types::list::List;

use crate::{
    http::{
        request::Request,
        response::{response_status_code::ResponseStatusCode, Response},
    },
    setting::{ServerSetting},
};

use super::{
    component::Component,
    pipeline::{
        ActionComponent, ActionQueue, Bytes, CompressionComponent, CompressionQueue,
        ParserComponent, Pipeline, SenderComponent, SenderQueue,
    },
};

type ParserFunc = fn(&mut TcpStream) -> Result<Request, ResponseStatusCode>;
type ActionFunc<U> = fn(
    &Result<Request, ResponseStatusCode>,
    ServerSetting,
    utility_thread: &mut mpsc::Sender<U>,
) -> Result<Response, ResponseStatusCode>;
type CompressionFunc = fn(Response, Option<Request>, ServerSetting) -> Bytes;

pub struct Builder<U: Clone> {
    pub parser: Option<ParserFunc>,
    pub action: Option<ActionFunc<U>>,
    pub compression: Option<CompressionFunc>,
    pub utility_sender: Option<mpsc::Sender<U>>,
    pub settings: Option<Arc<RwLock<ServerSetting>>>
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

    pub fn build(&self) -> (Sender<TcpStream>, Pipeline<mpsc::Sender<U>>) {
        
        //building components back to front to deal with input queue dependencies & ownership issues
        let (sender_queue, sender) = build_sender();

        //build compressor
        if self.compression.is_none() {
            todo!()
        }
        let (compressor_queue, compression) = build_compressor(self.compression.unwrap(), sender_queue, &self.settings.clone().unwrap());

        //build action
        if self.action.is_none() || self.utility_sender.is_none() || self.settings.is_none() {
            todo!()
        }
        let (action_queue, action) = build_action(self.action.unwrap(), compressor_queue, self.utility_sender.clone().unwrap(), &self.settings.clone().unwrap());

        //build parser
        if self.parser.is_none() {
            todo!()
        }
        let (new_conn, parser) = build_parser(self.parser.unwrap(), action_queue);

        //construct pipeline
        let pipeline = Pipeline {
            parser: parser,
            action: action,
            compression: compression,
            sender: sender,
            utility_access: self.utility_sender.clone().unwrap(),
        };

        (new_conn, pipeline)
    }

    pub fn fix(&self, pipeline: &mut Pipeline<U>) -> Result<(), ()> {
        todo!()
    }
}

impl<U: Clone + Send + 'static> Default for Builder<U> {
    fn default() -> Self {
        Self {
            parser: None,
            action: None,
            compression: None,
            utility_sender: None,
            settings: None
        }
    }
}

// look into generic implementations
fn build_parser(
    parser: ParserFunc,
    output_queue: Arc<Mutex<ActionQueue>>,
) -> (Sender<TcpStream>, ParserComponent) {
    let (tx, rx) = mpsc::channel::<TcpStream>();

    let rx = Arc::new(Mutex::new(rx));

    let rx_clone = rx.clone();

    let thread = thread::spawn(move || {
        let rx = &*rx_clone.lock().unwrap();

        loop {
            //get tcp stream
            let tcp_stream = rx.recv();

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
                Err(_) => todo!(), //send error message
            }
        }
    });

    let component = Component::new(rx, thread);

    (tx, component)
}

fn build_action<U: Send + 'static>(
    func: ActionFunc<U>,
    output_queue: Arc<Mutex<CompressionQueue>>,
    utility_access: mpsc::Sender<U>,
    settings: &Arc<RwLock<ServerSetting>>
) -> (Arc<Mutex<ActionQueue>>, ActionComponent) {
    let input_queue = Arc::new(Mutex::new(ActionQueue::default()));

    let input_queue_tmp = input_queue.clone();

    let server_settings = settings.clone();

    //let ch = utility_access;

    let thread = thread::spawn(move || {
        let input_queue = input_queue_tmp;
        let mut utility_access = utility_access;
        loop {
            //get control of input queue
            let mut input_queue = match input_queue.try_lock() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let input_queue = &mut *input_queue;

            //get first element in queue
            let (stream, action_cmd) = match input_queue.remove_front() {
                Some(value) => value,
                None => continue,
            };

            //action upon data
            let server_settings = (&*server_settings.read().unwrap()).clone();
            match func(
                &action_cmd,
                server_settings,
                &mut utility_access,
            ) {
                Ok(val) => {
                    let _ = output_queue.lock().unwrap().push_back((stream, val, action_cmd.ok()));
                }
                Err(err) => {
                    let _ = input_queue.push_front((stream, Err(err))); //convert error into raw bytes
                }
            }
        }
    });

    let component = Component::new(input_queue.clone(), thread);

    (input_queue, component)
}

fn build_compressor(
    func: CompressionFunc,
    output_queue: Arc<Mutex<SenderQueue>>,
    settings: &Arc<RwLock<ServerSetting>>
) -> (Arc<Mutex<CompressionQueue>>, CompressionComponent) {
    let input_queue = Arc::new(Mutex::new(CompressionQueue::default()));

    let input_queue_tmp = input_queue.clone();

    let server_settings = settings.clone();

    let thread = thread::spawn(move || {
        let input_queue = input_queue_tmp;
        loop {
            //get first element in queue
            let (stream, response, request) = match dequeue(&input_queue) {
                Some(value) => value,
                None => continue,
            };

            //compress data and push to next pipe
            let server_settings = (&*server_settings.read().unwrap()).clone();
            let _ = output_queue
                .lock()
                .unwrap()
                .push_back((stream, func(response, request, server_settings)));
        }
    });

    let component = Component::new(input_queue.clone(), thread);

    (input_queue, component)
}

fn build_sender() -> (Arc<Mutex<SenderQueue>>, SenderComponent) {
    let input_queue = Arc::new(Mutex::new(SenderQueue::default()));

    let input_queue_tmp = input_queue.clone();
    let thread = thread::spawn(move || {
        let input_queue = input_queue_tmp;
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
    });

    let component = Component::new(input_queue.clone(), thread);

    (input_queue, component)
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