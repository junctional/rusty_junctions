use crate::types::{ids::ChannelId, Message, Packet};
use bag::Bag;
use std::{
    marker::{Send, Sized},
    sync::mpsc::Sender,
};

pub trait JoinPattern: Send {
    /// Return `true` if the Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    // TODO: Ensure this is a valid implementation of `is_valid` for any number of channels
    // TODO: The hashmap might be able to be precomputed in the macro
    fn is_alive(&self, messages: &Bag<ChannelId, Message>) -> bool {
        // Create a hashmap associating each `ChannelId` with its number of
        // occurrences
        let mut threshold_channels = std::collections::HashMap::new();
        self.channels().into_iter().for_each(|chan| {
            let counter = threshold_channels.entry(chan).or_insert(0);
            *counter += 1;
        });

        // Check if there is a sufficient number of messages for each channel
        for (channel, num) in threshold_channels.into_iter() {
            let messages_for_channel = messages.count_items(&channel);
            if messages_for_channel < num {
                return false;
            }
        }

        true
    }

    fn add(self, sender: Sender<Packet>)
    where
        Self: Sized + Send + 'static,
    {
        sender
            .send(Packet::AddJoinPatternRequest {
                join_pattern: Box::new(self),
            })
            .unwrap();
    }

    /// Return a `Vec<ChannelId` for each of the channels in the Join Pattern
    fn channels(&self) -> Vec<ChannelId>;

    /// Given the `Message` for each of the channels in the pattern - fire.
    fn fire(&self, messages: Vec<Message>) -> std::thread::JoinHandle<()>;
}
