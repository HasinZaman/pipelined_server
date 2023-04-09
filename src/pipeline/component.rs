use std::{marker::PhantomData, thread::JoinHandle};

pub struct Component<IQ, OQ, E> {
    pub(super) input_queue: IQ,

    thread: JoinHandle<E>,

    output_queue: PhantomData<OQ>,
}

impl<IQ, OQ, E> Component<IQ, OQ, E> {
    pub fn new(input_queue: IQ, thread: JoinHandle<E>) -> Self {
        Self {
            input_queue,
            thread: thread,
            output_queue: PhantomData,
        }
    }

    pub fn thread_state(&self) -> bool {
        !self.thread.is_finished()
    }

    //swap out thread on recovery
    pub(super) fn swap_out_thread(&mut self, new_thread: JoinHandle<E>) {
        if self.thread_state() {
            return;
        }

        self.thread = new_thread;
    }
}
