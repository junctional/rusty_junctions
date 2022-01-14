//! Collection of types to increase readability and maintainability of the
//! crate.

use crate::join_pattern::JoinPattern;
use std::{any::Any, sync::mpsc::Sender, marker::Send};

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
    // TODO: Currently dynamic dispatch is being used
    AddJoinPatternRequest { join_pattern: Box<dyn JoinPattern + Send> },
    /// Request the internal control thread managing the `Message`s to shut down.
    ShutDownRequest,
}

/// Function types related to various kind of functions that can be stored and
/// executed with Join Patterns.
pub mod functions {
    // Types and Traits for functions which take one argument.
    rusty_junctions_macro::function_types!(unary; 1);

    // Types and Traits for functions which take two arguments.
    rusty_junctions_macro::function_types!(binary; 2);

    // Types and Traits for functions which take three arguments.
    rusty_junctions_macro::function_types!(ternary; 3);
}

/// Adds specific ID types for the various IDs that are used in the crate.
pub mod ids {
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// ID to identify a channel within a Join Pattern.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct ChannelId(usize);

    impl ChannelId {
        // pub(crate) fn new(value: usize) -> ChannelId {
        //     ChannelId(value)
        // }

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
