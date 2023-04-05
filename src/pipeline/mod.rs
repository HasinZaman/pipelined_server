use std::{net::{TcpListener, TcpStream}, thread::JoinHandle, sync::mpsc::Sender};

use log::trace;

use crate::setting::{ServerSetting};

use self::{utility_thread::UtilityThread, builder::Builder, pipeline::Pipeline};

#[cfg(test)]
mod tests;

pub mod builder;
mod component;
mod pipeline;
pub mod utility_thread;

mod default;

struct Server<U:  Clone + Send + 'static> {
    settings: ServerSetting,
    builder: Builder<U>,
    utility_thread: (Sender<U>, JoinHandle<()>),
}

impl<U: Clone + Send + 'static> Server<U> {
    //create new server
    pub fn new(settings: ServerSetting, utility_thread: (Sender<U>, JoinHandle<()>), builder: Builder<U>) -> Server<U> {
        let builder = builder.set_settings(settings.clone());
        Server{
            settings,
            builder,
            utility_thread
        }
    }

    pub fn run<const PIPELINES: usize>(&self) {
        //build pipeline
        let pipe_lines:[(Sender<TcpStream>, Pipeline<Sender<U>>); PIPELINES] = core::array::from_fn(|_| self.builder.build());

        let mut i1:usize = 0; //should be replaced with min heap and used to select the least busy pipeline

        // initialize tcp listener
        let listener =  {
            let listener = TcpListener::bind(
                {
                    let host = &(*self.builder.settings.as_ref().unwrap()).read().unwrap().address;
                    let port = &(*self.builder.settings.as_ref().unwrap()).read().unwrap().port;
    
                    format!(
                        "{}:{}",
                        host,
                        port
                    )
                }
            );
            
            let listener = match listener {
                Ok(listener) => listener,
                Err(_err) => todo!(),
            };

            listener
        };


        for stream in listener.incoming() {
            i1 = (i1 + 1) % PIPELINES;

            match stream{
                Ok(stream) => {
                    let _ = pipe_lines[i1].0.send(stream);
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