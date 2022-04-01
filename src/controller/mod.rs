//! Control structure started by any new `Junction`, running in a background thread
//! to handle the coordination of Join Pattern creation and execution.
use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use crate::{
    join_pattern::JoinPattern,
    types::{
        ids::{ChannelId, JoinPatternId},
        Message, Packet,
    },
};

use bag::Bag;
use counter::Counter;
use inverted_index::InvertedIndex;

mod alive;
mod fire;
mod handle;
mod handlers;

pub use handle::ControllerHandle;

/// Struct to handle `Packet`s sent from the user in the background.
///
/// This struct holds all the information required to store and fire
/// `JoinPattern`s once all requirements have been met. It is created by a
/// `Junction` in a separate control thread, where it continuously listens
/// for `Packet`s sent by user code and reacts accordingly.
pub(crate) struct Controller {
    latest_channel_id: ChannelId,
    latest_join_pattern_id: JoinPatternId,
    /// Counter for how many messages have arrived since creation.
    message_counter: Counter,
    /// Collection of all currently available messages.
    messages: Bag<ChannelId, Message>,
    /// Collection of all available Join Patterns for the `Junction` associated with
    /// this `Controller`.
    join_patterns: HashMap<JoinPatternId, Box<dyn JoinPattern>>,
    /// Map of `JoinPatternId`s to the message count at which they were last
    /// fired, `None` if the Join Pattern has never been fired. Used to
    /// determine precedence of Join Patterns that have not been fired in a
    /// while when needing to choose which of the alive Join Patterns to fire.
    join_pattern_last_fired: HashMap<JoinPatternId, Option<Counter>>,
    /// `InvertedIndex` matching `ChannelId`s to all Join Patterns they appear in.
    /// Used to easily determine which Join Patterns are relevant any time a new
    /// message comes in.
    join_pattern_index: InvertedIndex<ChannelId, JoinPatternId>,
    /// A store of all of the `JoinPattern` that are currently firing. When
    /// the `stop` directive is given to the controller, we can join all of
    /// the `JoinHandle`s to ensure the computation being performed by each
    /// thread is given time to complete.
    firing_join_patterns: Vec<JoinHandle<()>>,
}

impl Controller {
    pub(crate) fn new() -> Controller {
        Controller {
            latest_channel_id: ChannelId::default(),
            latest_join_pattern_id: JoinPatternId::default(),
            message_counter: Counter::default(),
            messages: Bag::new(),
            join_patterns: HashMap::new(),
            join_pattern_last_fired: HashMap::new(),
            join_pattern_index: InvertedIndex::new(),
            firing_join_patterns: Vec::new(),
        }
    }

    /// Start thread to handle incoming `Packet`s from `Junction` user.
    ///
    /// Start new thread in the background to handle incoming `Packet`s sent from
    /// the user of the `Junction` that created this `Controller`. Return a
    /// `ControlThreadHandle` so that this control thread can be joint at any future
    /// point.
    pub(crate) fn start(
        self,
        sender: Sender<Packet>,
        receiver: Receiver<Packet>,
    ) -> ControllerHandle {
        ControllerHandle::new(sender, thread::spawn(move || self.handle_packets(receiver)))
    }
}
