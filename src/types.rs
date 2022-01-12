//! Collection of types to increase readability and maintainability of the
//! crate.

use crate::types::ids::ChannelId;
use bag::Bag;
use std::{any::Any, sync::mpsc::Sender};

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
    /// Request the internal control thread managing the `Message`s to shut down.
    ShutDownRequest,
}

/// Enum defining all Join Patterns that can be added to a Junction using the
/// `AddJoinPatternRequest` in a `Packet`.
// pub struct JoinPattern<T: crate::join_pattern::JoinPattern>(T);
pub enum JoinPattern {
    /// Single channel Join Pattern.
    UnarySend(crate::patterns::unary::SendJoinPattern),
    /// Single `RecvChannel` Join Pattern.
    UnaryRecv(crate::patterns::unary::RecvJoinPattern),
    /// Single `BidirChannel` Join Pattern.
    UnaryBidir(crate::patterns::unary::BidirJoinPattern),
    /// Two `SendChannel` Join Pattern.
    BinarySend(crate::patterns::binary::SendJoinPattern),
    /// `SendChannel` and `RecvChannel` Join Pattern.
    BinaryRecv(crate::patterns::binary::RecvJoinPattern),
    /// `SendChannel` and `BidirChannel` Join Pattern.
    BinaryBidir(crate::patterns::binary::BidirJoinPattern),
    /// Three `SendChannel` Join Pattern.
    TernarySend(crate::patterns::ternary::SendJoinPattern),
    /// Two `SendChannel` and `RecvChannel` Join Pattern.
    TernaryRecv(crate::patterns::ternary::RecvJoinPattern),
    /// Two `SendChannel` and `BidirChannel` Join Pattern.
    TernaryBidir(crate::patterns::ternary::BidirJoinPattern),
}

impl JoinPattern {
    /// Return `true` if the Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    // TODO: Ensure this is a valid implementation of `is_valid` for any number of channels
    pub fn is_alive(&self, messages: &Bag<ChannelId, Message>) -> bool {
        use itertools::Itertools;
        let channels = self.channels();

        for (key, group) in &channels.into_iter().group_by(|chan| *chan) {
            let messages_for_channel = messages.count_items(&key);
            if messages_for_channel < group.collect::<Vec<ChannelId>>().len() {
                return false;
            }
        }

        true
    }

    /// Return a `Vec<ChannelId` for each of the channels in the Join Pattern
    pub fn channels(&self) -> Vec<ChannelId> {
        use JoinPattern::*;
        let channels = match self {
            UnarySend(jp) => jp.channels(),
            UnaryRecv(jp) => jp.channels(),
            UnaryBidir(jp) => jp.channels(),
            BinarySend(jp) => jp.channels(),
            BinaryRecv(jp) => jp.channels(),
            BinaryBidir(jp) => jp.channels(),
            TernarySend(jp) => jp.channels(),
            TernaryRecv(jp) => jp.channels(),
            TernaryBidir(jp) => jp.channels(),
        };

        channels
    }

    pub fn fire(&self, mut messages: Vec<Message>) {
        // TODO: Find a better way of doing this, consider using `swap_remove`
        use JoinPattern::*;
        match self {
            UnarySend(jp) => jp.fire(messages.remove(0)),
            UnaryRecv(jp) => jp.fire(messages.remove(0)),
            UnaryBidir(jp) => jp.fire(messages.remove(0)),
            BinarySend(jp) => jp.fire(messages.remove(0), messages.remove(0)),
            BinaryRecv(jp) => jp.fire(messages.remove(0), messages.remove(0)),
            BinaryBidir(jp) => jp.fire(messages.remove(0), messages.remove(0)),
            TernarySend(jp) => jp.fire(messages.remove(0), messages.remove(0), messages.remove(0)),
            TernaryRecv(jp) => jp.fire(messages.remove(0), messages.remove(0), messages.remove(0)),
            TernaryBidir(jp) => jp.fire(messages.remove(0), messages.remove(0), messages.remove(0)),
        };
    }
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
