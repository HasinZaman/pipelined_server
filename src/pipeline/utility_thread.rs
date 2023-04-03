// use function pointers as means

use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};


#[macro_export]
macro_rules! UtilityThreadBuilder {
    () => {todo!()}
}

pub struct UtilityThread<C: Sized + Sync + Send, V> {
    _thread: JoinHandle<()>,

    loop_func: fn(C) -> V,

    pub sender: mpsc::Sender<(C, mpsc::Sender<V>)>,
}

impl<C: Sized + Sync + Send + 'static, V: Sized + Sync + Send + 'static> UtilityThread<C, V> {
    //just added bunch of requirements might not be a good idea
    pub fn new(loop_func: fn(C) -> V) -> Self {
        let (sender, receiver) = mpsc::channel::<(C, mpsc::Sender<V>)>();

        let _thread = generate_thread(receiver, loop_func);

        Self {
            _thread,
            loop_func,
            sender,
        }
    }

    pub fn fix(&mut self) {
        let func = self.loop_func;
        *self = Self::new(func);
    }
}

pub struct PooledUtilityThread<const POOL_SIZE: usize, C: Sized + Sync + Send, V> {
    _thread: JoinHandle<()>,
    _pool_threads: [JoinHandle<()>; POOL_SIZE],

    loop_func: fn(C) -> V,

    pub sender: mpsc::Sender<(C, mpsc::Sender<V>)>,
}

impl<const POOL_SIZE: usize, C, V> PooledUtilityThread<POOL_SIZE, C, V>
where
    C: Sized + Sync + Send + 'static,
    V: Sized + Sync + Send + 'static,
{
    pub fn new(loop_func: fn(C) -> V) -> Self {
        let (sender, receiver) = mpsc::channel::<(C, mpsc::Sender<V>)>();

        // create pool utility threads

        // create main event thread

        let _thread = generate_pool_thread(receiver, loop_func);

        todo!()
    }

    pub fn fix(&mut self) {
        let func = self.loop_func;
        *self = Self::new(func);
    }
}

fn generate_thread<C: Sized + Sync + Send + 'static, V: Sized + Sync + Send + 'static>(
    receiver: mpsc::Receiver<(C, mpsc::Sender<V>)>,
    loop_func: fn(C) -> V,
) -> JoinHandle<()> {
    let _thread = thread::spawn(move || loop {
        let (command, tx) = match receiver.recv() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let _ = tx.send(loop_func(command));
    });
    _thread
}

fn generate_pool_thread<C: Sized + Sync + Send + 'static, V: Sized + Sync + Send + 'static>(
    receiver: mpsc::Receiver<(C, mpsc::Sender<V>)>,
    loop_func: fn(C) -> V,
) -> JoinHandle<()> {
    let _thread = thread::spawn(move || loop {
        let (command, tx) = match receiver.recv() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let _ = tx.send(loop_func(command));
    });
    _thread
}