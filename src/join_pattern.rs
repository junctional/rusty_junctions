use crate::types::{ids::ChannelId, Message, Packet};
use bag::Bag;
use itertools::Itertools;
use std::{
    marker::{Send, Sized},
    sync::mpsc::Sender,
};

pub trait JoinPattern {
    /// Return `true` if the Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    // TODO: Ensure this is a valid implementation of `is_valid` for any number of channels
    fn is_alive(&self, messages: &Bag<ChannelId, Message>) -> bool {
        let channels = self.channels();
        for (key, group) in &channels.into_iter().group_by(|chan| *chan) {
            let messages_for_channel = messages.count_items(&key);
            if messages_for_channel < group.collect::<Vec<ChannelId>>().len() {
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
    fn fire(&self, messages: Vec<Message>);
}
