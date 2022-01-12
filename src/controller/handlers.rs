use std::{
    collections::LinkedList,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    controller::Controller,
    join_pattern::JoinPattern,
    types::{
        ids::{ChannelId, JoinPatternId},
        Message, Packet,
    },
};

impl Controller {
    /// Handle incoming `Packet` from associated `Junction`.
    ///
    /// This function will continuously receive `Packet`s sent from structs
    /// associated with the `Junction` that created and started this `Controller`
    /// until a `Packet::ShutDownRequest` has been sent.
    pub(in crate::controller) fn handle_packets(&mut self, receiver: Receiver<Packet>) {
        use Packet::*;

        while let Ok(packet) = receiver.recv() {
            match packet {
                Message { channel_id, msg } => self.handle_message(channel_id, msg),
                NewChannelIdRequest { return_sender } => {
                    self.handle_new_channel_id_request(return_sender)
                }
                AddJoinPatternRequest { join_pattern } => {
                    self.handle_add_join_pattern_request(join_pattern)
                }
                ShutDownRequest => break,
            }
        }
    }

    /// Handle a received `Message` from a given channel.
    ///
    /// The first action taken in handling a `Message` is storing the received
    /// message in the `Message` bag of the `Controller`.
    ///
    /// The second action is to start determining if any of the Join Patterns stored
    /// with the `Controller` are alive and if so, which of these to fire.
    fn handle_message(&mut self, channel_id: ChannelId, msg: Message) {
        self.messages.add(channel_id, msg);
        self.message_counter.increment();

        self.handle_join_pattern_firing(channel_id);
    }

    /// Handle the firing of a `JoinPattern`, if possible.
    ///
    /// Determine which `JoinPattern`s contain the channel with the given
    /// `ChannelId`. For these, check which ones have at least one `Message`
    /// available for each of their channels, i.e. are alive, then select
    /// one `JoinPattern` to be fired. If at any point during this process
    /// no more `JoinPattern`s remain, nothing will be done.
    fn handle_join_pattern_firing(&mut self, channel_id: ChannelId) {
        let mut alive_join_patterns: Vec<JoinPatternId> = Vec::new();

        if let Some(jp_ids) = self.relevant_join_patterns(channel_id) {
            alive_join_patterns = self.alive_join_patterns(jp_ids);
        }

        if let Some(jp_id_to_fire) = self.select_to_fire(&mut alive_join_patterns) {
            self.fire_join_pattern(*jp_id_to_fire);
            self.reset_last_fired(*jp_id_to_fire);
        }
    }

    /// Send new, *unique* `ChannelId` back to the requesting `Junction`.
    ///
    /// # Panics
    ///
    /// Panics if the new `ChannelId` could not be sent to the requesting `Junction`.
    fn handle_new_channel_id_request(&mut self, return_sender: Sender<ChannelId>) {
        return_sender.send(self.new_channel_id()).unwrap();
    }

    /// Add new Join Pattern to `Controller` storage.
    fn handle_add_join_pattern_request(&mut self, join_pattern: JoinPattern) {
        let jp_id = self.new_join_pattern_id();

        self.initialize_last_fired(jp_id);

        self.insert_join_pattern(jp_id, join_pattern);
    }

    /// Return the `JoinPatternId`s of relevant Join Patterns for given `ChannelId`.
    ///
    /// A Join Pattern is considered relevant for a given `ChannelId` if at least
    /// one of its channels has the `ChannelId`.
    fn relevant_join_patterns(&self, channel_id: ChannelId) -> Option<&LinkedList<JoinPatternId>> {
        self.join_pattern_index.peek_all(&channel_id)
    }

    /// Insert Join Pattern into relevant internal storage.
    ///
    /// The given Join Pattern needs to be registered within the internal
    /// `InvertedIndex` for future look-up operations and then stored in the
    /// Join Pattern collection.
    fn insert_join_pattern(&mut self, join_pattern_id: JoinPatternId, join_pattern: JoinPattern) {
        join_pattern.channels().iter().for_each(|chan| {
            self.join_pattern_index
                .insert_single(*chan, join_pattern_id)
        });
        self.join_patterns.insert(join_pattern_id, join_pattern);
    }

    /// Generate new, *unique* `ChannelId`.
    fn new_channel_id(&mut self) -> ChannelId {
        let ch_id = self.latest_channel_id;
        self.latest_channel_id.increment();

        ch_id
    }

    /// Generate new, *unique* `JoinPatternId`.
    fn new_join_pattern_id(&mut self) -> JoinPatternId {
        let jp_id = self.latest_join_pattern_id;
        self.latest_join_pattern_id.increment();

        jp_id
    }
}
