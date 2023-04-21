use std::{
    net::TcpStream,
    sync::{
        mpsc::{Receiver},
        Arc, Mutex,
    }, fmt::{Display, Debug}, ptr,
};

use cyclic_data_types::list::List;

use crate::http::{
    request::Request,
    response::{response_status_code::ResponseStatusCode, Response},
};

use super::component::Component;

const QUEUE_SIZE: usize = 264;

pub(super) type Bytes = Vec<u8>;

pub(super) type ConnectionQueue = Arc<Mutex<Receiver<TcpStream>>>;
pub(super) type ActionQueue =
    List<QUEUE_SIZE, (TcpStream, Result<Request, ResponseStatusCode>), false>;
pub(super) type CompressionQueue = List<QUEUE_SIZE, (TcpStream, Response, Option<Request>), false>;
pub(super) type SenderQueue = List<QUEUE_SIZE, (TcpStream, Bytes), false>;

type MutexWrapper<E> = Arc<Mutex<E>>;

pub(super) type ParserComponent = Component<ConnectionQueue, MutexWrapper<ActionQueue>, ()>;
pub(super) type ActionComponent =
    Component<MutexWrapper<ActionQueue>, MutexWrapper<CompressionQueue>, ()>;
pub(super) type CompressionComponent =
    Component<MutexWrapper<CompressionQueue>, MutexWrapper<SenderQueue>, ()>;
pub(super) type SenderComponent = Component<MutexWrapper<SenderQueue>, (), ()>;

pub struct Pipeline {
    //get connection
    pub(super) parser: ParserComponent,

    //action
    pub(super) action: ActionComponent,

    //compression
    pub(super) compression: CompressionComponent,

    //sender
    pub(super) sender: SenderComponent,
}

impl Pipeline {
    pub fn pipeline_state(&self) -> bool {
        self.parser.thread_state()
            && self.action.thread_state()
            && self.compression.thread_state()
            && self.sender.thread_state()
    }
}

impl Display for Pipeline {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr = ptr::addr_of!(self);
        write!(fmt, "Pipeline({})", ptr as usize)
    }
}

impl Debug for Pipeline {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Pipeline")
            .field("parser component state", &self.parser.thread_state())
            .field("action component state", &self.action.thread_state())
            .field("compression component state", &self.compression.thread_state())
            .field("sender component state", &self.sender.thread_state())
            .finish()
    }
}