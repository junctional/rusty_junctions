use std::{
    sync::mpsc::Sender,
    thread::{JoinHandle, Thread},
};

use crate::{join_pattern::JoinPattern, types::Packet};

/// Handle to a `Junction`'s underlying `Controller`.
///
/// This struct carries a `JoinHandle` to the thread that the `Controller` of
/// a `Junction` is running in. It allows for the `Controller` and its thread
/// to be stopped gracefully at any point.
pub struct ControllerHandle<JP: JoinPattern> {
    sender: Sender<Packet<JP>>,
    control_thread_handle: Option<JoinHandle<()>>,
}

impl<JP: JoinPattern> ControllerHandle<JP> {
    pub(crate) fn new(sender: Sender<Packet<JP>>, handle: JoinHandle<()>) -> ControllerHandle<JP> {
        ControllerHandle {
            sender,
            control_thread_handle: Some(handle),
        }
    }

    /// Extracts a handle to the underlying thread.
    pub fn thread(&self) -> Option<&Thread> {
        match &self.control_thread_handle {
            Some(h) => Some(h.thread()),
            None => None,
        }
    }

    /// Request the `Controller` to stop gracefully, then join its thread.
    ///
    /// # Panics
    ///
    /// Panics if it was unable to send shut-down request to the control thread.
    pub fn stop(&mut self) {
        self.sender.send(Packet::ShutDownRequest).unwrap();

        self.control_thread_handle.take().unwrap().join().unwrap();
    }
}
