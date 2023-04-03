use std::{net::TcpListener, thread::JoinHandle, sync::mpsc::Sender};

use crate::setting::{ServerSetting};

use self::{utility_thread::UtilityThread, builder::Builder};

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

    pub fn run(&self) {
        //build pipeline
        let (head, _pipeline) = self.builder.build();

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
            match stream{
                Ok(stream) => {
                    let _ = head.send(stream);
                }
                Err(_) => {
                    todo!()
                }
            }
            //pipeline check
        }
    }
}

