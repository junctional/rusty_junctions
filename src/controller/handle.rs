use std::{
    sync::mpsc::Sender,
    thread::{JoinHandle, Thread},
};

use crate::types::Packet;

/// Handle to a `Junction`'s underlying `Controller`.
///
/// This struct carries a `JoinHandle` to the thread that the `Controller` of
/// a `Junction` is running in. It allows for the `Controller` and its thread
/// to be stopped gracefully at any point.
pub struct ControllerHandle {
    sender: Sender<Packet>,
    control_thread_handle: Option<JoinHandle<()>>,
}

impl ControllerHandle {
    pub(crate) fn new(sender: Sender<Packet>, handle: JoinHandle<()>) -> ControllerHandle {
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
        log::debug!("Controller asked to shutdown");
        self.sender
            .send(Packet::ShutDownRequest)
            .map_err(|e| log::error!("Failed to send ShutDownRequest: {e:?}"))
            .unwrap();

        let controller_handle = self.control_thread_handle.take();

        if controller_handle.is_none() {
            log::error!("Failed to take the ControllerHandle");
        }

        controller_handle
            .unwrap()
            .join()
            .map_err(|_| log::error!("Failed to join the Controller thread to the main thread"))
            .unwrap();

        log::debug!("Controller shutdown");
    }
}
