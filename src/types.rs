//! Collection of types to increase readability and maintainability of the
//! crate.

use std::any::Any;
use std::sync::mpsc::Sender;
use std::thread::{JoinHandle, Thread};

use crate::patterns;

/// Shallow wrapper for a trait object using `Box` that can pass through thread
/// boundaries.
pub struct Message(Box<dyn Any + Send>);

impl Message {
    pub(crate) fn new<T>(raw_value: T) -> Message
    where
        T: Any + Send,
    {
        Message(Box::new(raw_value))
    }

    /// Cast internal trait object to `Box<T>`.
    pub(crate) fn downcast<T>(self) -> Result<Box<T>, Box<dyn Any + Send>>
    where
        T: Any + Send,
    {
        self.0.downcast::<T>()
    }
}

/// Standardized packet to be used to send messages of various types on the
/// channels of a Junction.
pub enum Packet {
    /// General message send from channel identified by `channel_id`.
    Message {
        channel_id: ids::ChannelId,
        msg: Message,
    },
    /// Request a new channel ID from the Junction so a new channel can be
    /// constructed. New ID will be sent back through `return_sender`.
    NewChannelIdRequest {
        return_sender: Sender<ids::ChannelId>,
    },
    /// Request adding a new Join Pattern to the Junction.
    AddJoinPatternRequest { join_pattern: JoinPattern },
    /// Request the Junction to shut down the internal control thread.
    ShutDownRequest,
}

/// Enum defining all Join Patterns that can be added to a Junction using the
/// `AddJoinPatternRequest` in a `Packet`.
pub enum JoinPattern {
    /// Single channel Join Pattern.
    UnarySend(patterns::unary::SendJoinPattern),
    /// Single `RecvChannel` Join Pattern.
    UnaryRecv(patterns::unary::RecvJoinPattern),
    /// Single `BidirChannel` Join Pattern.
    UnaryBidir(patterns::unary::BidirJoinPattern),
    /// Two `SendChannel` Join Pattern.
    BinarySend(patterns::binary::SendJoinPattern),
    /// `SendChannel` and `RecvChannel` Join Pattern.
    BinaryRecv(patterns::binary::RecvJoinPattern),
    /// `SendChannel` and `BidirChannel` Join Pattern.
    BinaryBidir(patterns::binary::BidirJoinPattern),
    /// Three `SendChannel` Join Pattern.
    TernarySend(patterns::ternary::SendJoinPattern),
    /// Two `SendChannel` and `RecvChannel` Join Pattern.
    TernaryRecv(patterns::ternary::RecvJoinPattern),
    /// Two `SendChannel` and `BidirChannel` Join Pattern.
    TernaryBidir(patterns::ternary::BidirJoinPattern),
}

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
        self.sender.send(Packet::ShutDownRequest).unwrap();

        self.control_thread_handle.take().unwrap().join().unwrap();
    }
}

/// Function types related to various kind of functions that can be stored and
/// executed with Join Patterns.
pub mod functions {
    use super::*;

    /// Types and Traits for functions which take one argument.
    pub mod unary {
        use super::*;

        /// Trait to allow boxed up functions that take one `Message` and return
        /// nothing to be cloned.
        pub trait FnBoxClone: Fn(Message) -> () + Send {
            fn clone_box(&self) -> Box<dyn FnBoxClone>;
        }

        impl<F> FnBoxClone for F
        where
            F: Fn(Message) -> () + Send + Clone + 'static,
        {
            /// Proxy function to be able to implement the `Clone` trait on
            /// boxed up functions that take one `Message` and return nothing.
            fn clone_box(&self) -> Box<dyn FnBoxClone> {
                Box::new(self.clone())
            }
        }

        impl Clone for Box<dyn FnBoxClone> {
            fn clone(&self) -> Box<dyn FnBoxClone> {
                (**self).clone_box()
            }
        }

        /// Type alias for boxed up cloneable functions that take one `Message` and
        /// return nothing. Mainly meant to increase readability of code.
        pub type FnBox = Box<dyn FnBoxClone>;
    }

    /// Types and Traits for functions which take two arguments.
    pub mod binary {
        use super::*;

        /// Trait to allow boxed up functions that take two `Message`s and return
        /// nothing to be cloned.
        pub trait FnBoxClone: Fn(Message, Message) -> () + Send {
            fn clone_box(&self) -> Box<dyn FnBoxClone>;
        }

        impl<F> FnBoxClone for F
        where
            F: Fn(Message, Message) -> () + Send + Clone + 'static,
        {
            /// Proxy function to be able to implement the `Clone` trait on
            /// boxed up functions that take two `Message`s and return nothing.
            fn clone_box(&self) -> Box<dyn FnBoxClone> {
                Box::new(self.clone())
            }
        }

        impl Clone for Box<dyn FnBoxClone> {
            fn clone(&self) -> Box<dyn FnBoxClone> {
                (**self).clone_box()
            }
        }

        /// Type alias for boxed up cloneable functions that take two `Message`s and
        /// return nothing. Mainly meant to increase readability of code.
        pub type FnBox = Box<dyn FnBoxClone>;
    }

    /// Types and Traits for functions which take three arguments.
    pub mod ternary {
        use super::*;

        /// Trait to allow boxed up functions that take three `Message`s and return
        /// nothing to be cloned.
        pub trait FnBoxClone: Fn(Message, Message, Message) -> () + Send {
            fn clone_box(&self) -> Box<dyn FnBoxClone>;
        }

        impl<F> FnBoxClone for F
        where
            F: Fn(Message, Message, Message) -> () + Send + Clone + 'static,
        {
            /// Proxy function to be able to implement the `Clone` trait on
            /// boxed up functions that take three `Message`s and return nothing.
            fn clone_box(&self) -> Box<dyn FnBoxClone> {
                Box::new(self.clone())
            }
        }

        impl Clone for Box<dyn FnBoxClone> {
            fn clone(&self) -> Box<dyn FnBoxClone> {
                (**self).clone_box()
            }
        }

        /// Type alias for boxed up cloneable functions that take three `Message`s and
        /// return nothing. Mainly meant to increase readability of code.
        pub type FnBox = Box<dyn FnBoxClone>;
    }
}

/// Adds specific ID types for the various IDs that are used in the crate.
pub mod ids {
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// ID to identify a channel within a Join Pattern.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct ChannelId(usize);

    impl ChannelId {
        pub(crate) fn new(value: usize) -> ChannelId {
            ChannelId(value)
        }

        /// Increment the internal value of the channel ID.
        pub(crate) fn increment(&mut self) {
            self.0 += 1;
        }
    }

    /// ID to identify a Join Pattern within a Junction.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct JoinPatternId(usize);

    impl JoinPatternId {
        /// Increment the internal value of the Join Pattern ID.
        pub(crate) fn increment(&mut self) {
            self.0 += 1;
        }
    }

    /// Globally synchronized counter to ensure that no two Junctions will have
    /// the same ID.
    pub static LATEST_JUNCTION_ID: AtomicUsize = AtomicUsize::new(0);

    /// ID for a Junction to identify itself.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct JunctionId(usize);

    impl JunctionId {
        pub(crate) fn new() -> JunctionId {
            JunctionId(LATEST_JUNCTION_ID.fetch_add(1, Ordering::Relaxed))
        }
    }
}
