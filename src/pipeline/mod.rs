use std::{
    net::{TcpListener, TcpStream},
    sync::{mpsc::Sender},
    thread::{JoinHandle, self},
};

use log::{trace, error};

use crate::setting::ServerSetting;

use self::{builder::pipeline::Builder, pipeline::Pipeline};

#[cfg(test)]
//#[cfg(all(feature = "default_impl", test))]
mod tests;

pub mod builder;
mod component;
mod pipeline;
pub mod utility_thread;

//#[cfg(feature = "default_impl")]
pub mod default;

struct Server<U: Clone + Send + 'static> {
    settings: ServerSetting,
    builder: Builder<U>,
    utility_thread: (Sender<U>, JoinHandle<()>),
}

impl<U: Clone + Send + 'static> Server<U> {
    //create new server
    pub fn new(
        settings: ServerSetting,
        utility_thread: (Sender<U>, JoinHandle<()>),
        builder: Builder<U>,
    ) -> Server<U> {
        let builder = builder.set_settings(settings.clone());
        Server {
            settings,
            builder,
            utility_thread,
        }
    }

    pub fn run<const PIPELINES: usize>(&self) {
        //build pipeline
        let (senders, mut pipes) = {

            let mut sender: Vec<Sender<TcpStream>> = Vec::new();
            let mut pipes: Vec<Pipeline<Sender<U>>> = Vec::new();

            (0..PIPELINES).map(|_| self.builder.build())
                .for_each(|(s,p)| {
                    sender.push(s.clone());
                    pipes.push(p);
                });

            let sender : [Sender<TcpStream>; PIPELINES] =  sender.try_into().unwrap();
            let pipes : [Pipeline<Sender<U>>; PIPELINES] =  pipes.try_into().unwrap();

            (sender, pipes)
        };
        let mut i1: usize = 0; //should be replaced with min heap and used to select the least busy pipeline

        // initialize tcp listener
        let listener = {
            let listener = TcpListener::bind({
                let host = &(*self.builder.settings.as_ref().unwrap())
                    .read()
                    .unwrap()
                    .address;
                let port = &(*self.builder.settings.as_ref().unwrap())
                    .read()
                    .unwrap()
                    .port;

                format!("{}:{}", host, port)
            });

            let listener = match listener {
                Ok(listener) => listener,
                Err(_err) => todo!(),
            };

            listener
        };

        let builder = self.builder.clone();
        let _recovery_thread = {
            thread::spawn(move || {
                loop {
                    for pipe in &mut pipes{
                        if !pipe.pipeline_state() {
                            error!("{pipe} failure");
                            let _ = builder.fix(pipe);
                        }
                    }
                }
            })
        };

        for stream in listener.incoming() {
            i1 = (i1 + 1) % PIPELINES;

            match stream {
                Ok(stream) => {
                    let _ = senders[i1].send(stream);
                }
                Err(err) => {
                    trace!("{}", err);
                    todo!()
                }
            }
            //pipeline check
        }
    }
}
