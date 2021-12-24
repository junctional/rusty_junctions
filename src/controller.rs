//! Control structure started by any new `Junction`, running in a background thread
//! to handle the coordination of Join Pattern creation and execution.

use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::{cmp::Ordering, collections::HashMap, collections::LinkedList, vec::Vec};

use bag::Bag;
use counter::Counter;
use inverted_index::InvertedIndex;
use super::types::ids::{ChannelId, JoinPatternId};
use super::types::{ControllerHandle, JoinPattern, Message, Packet};

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
    join_patterns: HashMap<JoinPatternId, JoinPattern>,
    /// Map of `JoinPatternId`s to the message count at which they were last
    /// fired, `None` if the Join Pattern has never been fired. Used to
    /// determine precedence of Join Patterns that have not been fired in a
    /// while when needing to choose which of the alive Join Patterns to fire.
    join_pattern_last_fired: HashMap<JoinPatternId, Option<Counter>>,
    /// `InvertedIndex` matching `ChannelId`s to all Join Patterns they appear in.
    /// Used to easily determine which Join Patterns are relevant any time a new
    /// message comes in.
    join_pattern_index: InvertedIndex<ChannelId, JoinPatternId>,
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
        }
    }

    /// Start thread to handle incoming `Packet`s from `Junction` user.
    ///
    /// Start new thread in the background to handle incoming `Packet`s sent from
    /// the user of the `Junction` that created this `Controller`. Return a
    /// `ControlThreadHandle` so that this control thread can be joint at any future
    /// point.
    pub(crate) fn start(
        mut self,
        sender: Sender<Packet>,
        receiver: Receiver<Packet>,
    ) -> ControllerHandle {
        ControllerHandle::new(sender, thread::spawn(move || self.handle_packets(receiver)))
    }

    /// Handle incoming `Packet` from associated `Junction`.
    ///
    /// This function will continuously receive `Packet`s sent from structs
    /// associated with the `Junction` that created and started this `Controller`
    /// until a `Packet::ShutDownRequest` has been sent.
    fn handle_packets(&mut self, receiver: Receiver<Packet>) {
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

    /// Return the `JoinPatternId`s of relevant Join Patterns for given `ChannelId`.
    ///
    /// A Join Pattern is considered relevant for a given `ChannelId` if at least
    /// one of its channels has the `ChannelId`.
    fn relevant_join_patterns(&self, channel_id: ChannelId) -> Option<&LinkedList<JoinPatternId>> {
        self.join_pattern_index.peek_all(&channel_id)
    }

    /// Return the `JoinPatternId`s of all alive `JoinPattern`s.
    ///
    /// A `JoinPattern` is considered alive if for each of the channels
    /// involved in it, there is at least one `Message` available.
    fn alive_join_patterns(
        &self,
        join_pattern_ids: &LinkedList<JoinPatternId>,
    ) -> Vec<JoinPatternId> {
        // We need clone the `JoinPatternId`s at this point to avoid
        // because need to avoid the issue of `peek_all` borrowing mutably,
        // but then needing to mutably borrow again later to update the
        // latest fired `JoinPatternId`.
        join_pattern_ids
            .iter()
            .filter(|&jp_id| self.is_alive(*jp_id))
            .cloned()
            .collect()
    }

    /// Select which `JoinPattern` should be fired.
    ///
    /// In order to avoid certain scenarious in which one `JoinPattern` would
    /// block the execution of another, because it for instance has a subset of
    /// the other's channels, we need to ensure that from the `JoinPattern`s
    /// that are alive simultaneously, we select the one to be fired that has
    /// been waiting for the longest time.
    ///
    /// Specifically, we record the `Counter` of the `Message` that each
    /// `JoinPattern` has last been fired at. We use this as a pseudo-time and
    /// simply order the `JoinPattern`s by their `Counter` values, then take
    /// the one with the smallest.
    ///
    /// Note that this procedure should ensure a certain form of *fairness*,
    /// by which if a `JoinPattern` has been alive an infinite amount of times,
    /// it will fire at least once. In practice, this should amount to each
    /// `JoinPattern` being incapable of getting deadlocked by others.
    fn select_to_fire<'a>(
        &self,
        alive_jp_ids: &'a mut Vec<JoinPatternId>,
    ) -> Option<&'a JoinPatternId> {
        alive_jp_ids
            .sort_unstable_by(|&jp_id_1, &jp_id_2| self.compare_last_fired(jp_id_1, jp_id_2));

        alive_jp_ids.first()
    }

    /// Return `true` if Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    fn is_alive(&self, join_pattern_id: JoinPatternId) -> bool {
        use JoinPattern::*;

        if let Some(join_pattern) = self.join_patterns.get(&join_pattern_id) {
            match join_pattern {
                UnarySend(jp) => self.is_unary_alive(jp.channel_id()),
                UnaryRecv(jp) => self.is_unary_alive(jp.channel_id()),
                UnaryBidir(jp) => self.is_unary_alive(jp.channel_id()),
                BinarySend(jp) => {
                    self.is_binary_alive(jp.first_send_channel_id(), jp.second_send_channel_id())
                }
                BinaryRecv(jp) => self.is_binary_alive(jp.send_channel_id(), jp.recv_channel_id()),
                BinaryBidir(jp) => {
                    self.is_binary_alive(jp.send_channel_id(), jp.bidir_channel_id())
                }
                TernarySend(jp) => self.is_ternary_alive(
                    jp.first_send_channel_id(),
                    jp.second_send_channel_id(),
                    jp.third_send_channel_id(),
                ),
                TernaryRecv(jp) => self.is_ternary_alive(
                    jp.first_send_channel_id(),
                    jp.second_send_channel_id(),
                    jp.recv_channel_id(),
                ),
                TernaryBidir(jp) => self.is_ternary_alive(
                    jp.first_send_channel_id(),
                    jp.second_send_channel_id(),
                    jp.bidir_channel_id(),
                ),
            }
        } else {
            false
        }
    }

    /// Return `true` if *unary* Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    fn is_unary_alive(&self, channel_id: ChannelId) -> bool {
        self.messages.count_items(&channel_id) >= 1
    }

    /// Return `true` if *binary* Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    ///
    /// For binary Join Patterns, we need to ensure that should both channels
    /// involved have the same `ChannelId`, we actually have at least two
    /// `Message`s available.
    fn is_binary_alive(&self, first_ch_id: ChannelId, second_ch_id: ChannelId) -> bool {
        if first_ch_id == second_ch_id {
            self.messages.count_items(&first_ch_id) >= 2
        } else {
            self.is_unary_alive(first_ch_id) && self.is_unary_alive(second_ch_id)
        }
    }

    /// Return `true` if *ternary* Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    ///
    /// For ternary Join Patterns, we need to ensure that should more than one
    /// channel involved have the same `ChannelId`, we actually have enough
    /// `Message`s available.
    fn is_ternary_alive(
        &self,
        first_ch_id: ChannelId,
        second_ch_id: ChannelId,
        third_ch_id: ChannelId,
    ) -> bool {
        if first_ch_id == second_ch_id && second_ch_id == third_ch_id {
            self.messages.count_items(&first_ch_id) >= 3
        } else {
            self.is_binary_alive(first_ch_id, second_ch_id)
                && self.is_binary_alive(first_ch_id, third_ch_id)
                && self.is_binary_alive(second_ch_id, third_ch_id)
        }
    }

    /// Compare when the Join Patterns with given `JoinPatternId`s were last alive at.
    ///
    /// Rules for Order:
    /// 1. If neither `JoinPatternId` has a last alive `Counter`, then neither
    /// has been fired yet, so they can be viewed as equal in this ordering.
    /// 2. If only one `JoinPatternId` has no last alive `Counter`, then that
    /// one has to be ordered as less than the other since having been fired
    /// at least once will always be a later point of firing than not having
    /// been fired yet.
    /// 3. If both `JoinPatternId`s have last alive `Counter`s, use the ordering
    /// of these.
    ///
    /// # Panics
    ///
    /// For simplicity, this function panics if the either of the given
    /// `JoinPatternId`s is not registered in the internal map of `JoinPatternId`s
    /// to `Instant`s that describe the last `Instant` at which a particular Join
    /// Pattern was alive. That is to say, this function should only be called on
    /// `JoinPatternId`s which are definitely stored in the calling `Controller`.
    fn compare_last_fired(&self, jp_id_1: JoinPatternId, jp_id_2: JoinPatternId) -> Ordering {
        // TODO: Can we sensibly use `Option::flatten` here?
        let last_fired_1 = self.join_pattern_last_fired.get(&jp_id_1).unwrap();
        let last_fired_2 = self.join_pattern_last_fired.get(&jp_id_2).unwrap();

        if last_fired_1.is_none() && last_fired_2.is_none() {
            Ordering::Equal
        } else if last_fired_1.is_none() && last_fired_2.is_some() {
            Ordering::Less
        } else if last_fired_1.is_some() && last_fired_2.is_none() {
            Ordering::Greater
        } else {
            last_fired_1.cmp(last_fired_2)
        }
    }

    /// Fire the `JoinPattern` corresponding to the given `JoinPatternId`.
    ///
    /// The processs of firing a `JoinPattern` consists of first retrieving
    /// a `Message` for each of the channels involved in the `JoinPattern`,
    /// then passing these `Messages`s to the `JoinPattern` to handle the
    /// firing.
    ///
    /// # Panics
    ///
    /// Panics when there is no `JoinPattern` stored for the given
    /// `JoinPatternId`.
    fn fire_join_pattern(&mut self, join_pattern_id: JoinPatternId) {
        use JoinPattern::*;

        let join_pattern = self.join_patterns.get(&join_pattern_id).unwrap();

        match join_pattern {
            UnarySend(jp) => {
                let arg = self.messages.retrieve(&jp.channel_id()).unwrap();

                jp.fire(arg);
            }
            UnaryRecv(jp) => {
                let return_sender = self.messages.retrieve(&jp.channel_id()).unwrap();

                jp.fire(return_sender);
            }
            UnaryBidir(jp) => {
                let arg_and_sender = self.messages.retrieve(&jp.channel_id()).unwrap();

                jp.fire(arg_and_sender);
            }
            BinarySend(jp) => {
                let arg_1 = self.messages.retrieve(&jp.first_send_channel_id()).unwrap();
                let arg_2 = self
                    .messages
                    .retrieve(&jp.second_send_channel_id())
                    .unwrap();

                jp.fire(arg_1, arg_2);
            }
            BinaryRecv(jp) => {
                let arg = self.messages.retrieve(&jp.send_channel_id()).unwrap();
                let return_sender = self.messages.retrieve(&jp.recv_channel_id()).unwrap();

                jp.fire(arg, return_sender);
            }
            BinaryBidir(jp) => {
                let arg_1 = self.messages.retrieve(&jp.send_channel_id()).unwrap();
                let arg_2_and_sender = self.messages.retrieve(&jp.bidir_channel_id()).unwrap();

                jp.fire(arg_1, arg_2_and_sender);
            }
            TernarySend(jp) => {
                let arg_1 = self.messages.retrieve(&jp.first_send_channel_id()).unwrap();
                let arg_2 = self
                    .messages
                    .retrieve(&jp.second_send_channel_id())
                    .unwrap();
                let arg_3 = self.messages.retrieve(&jp.third_send_channel_id()).unwrap();

                jp.fire(arg_1, arg_2, arg_3);
            }
            TernaryRecv(jp) => {
                let arg_1 = self.messages.retrieve(&jp.first_send_channel_id()).unwrap();
                let arg_2 = self
                    .messages
                    .retrieve(&jp.second_send_channel_id())
                    .unwrap();
                let return_sender = self.messages.retrieve(&jp.recv_channel_id()).unwrap();

                jp.fire(arg_1, arg_2, return_sender);
            }
            TernaryBidir(jp) => {
                let arg_1 = self.messages.retrieve(&jp.first_send_channel_id()).unwrap();
                let arg_2 = self
                    .messages
                    .retrieve(&jp.second_send_channel_id())
                    .unwrap();
                let arg_3_and_sender = self.messages.retrieve(&jp.bidir_channel_id()).unwrap();

                jp.fire(arg_1, arg_2, arg_3_and_sender);
            }
        }
    }

    /// Reset the `Counter` at which the given Join Pattern has last been fired.
    fn reset_last_fired(&mut self, join_pattern_id: JoinPatternId) {
        self.join_pattern_last_fired
            .insert(join_pattern_id, Some(self.message_counter.clone()));
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

    /// Initialize the `Instant` at which Join Pattern was last alive.
    fn initialize_last_fired(&mut self, join_pattern_id: JoinPatternId) {
        self.join_pattern_last_fired.insert(join_pattern_id, None);
    }

    /// Insert Join Pattern into relevant internal storage.
    ///
    /// The given Join Pattern needs to be registered within the internal
    /// `InvertedIndex` for future look-up operations and then stored in the
    /// Join Pattern collection.
    fn insert_join_pattern(&mut self, join_pattern_id: JoinPatternId, join_pattern: JoinPattern) {
        use JoinPattern::*;

        match join_pattern {
            UnarySend(jp) => {
                self.join_pattern_index
                    .insert_single(jp.channel_id(), join_pattern_id);

                self.join_patterns.insert(join_pattern_id, UnarySend(jp));
            }
            UnaryRecv(jp) => {
                self.join_pattern_index
                    .insert_single(jp.channel_id(), join_pattern_id);

                self.join_patterns.insert(join_pattern_id, UnaryRecv(jp));
            }
            UnaryBidir(jp) => {
                self.join_pattern_index
                    .insert_single(jp.channel_id(), join_pattern_id);

                self.join_patterns.insert(join_pattern_id, UnaryBidir(jp));
            }
            BinarySend(jp) => {
                self.join_pattern_index
                    .insert_single(jp.first_send_channel_id(), join_pattern_id);
                self.join_pattern_index
                    .insert_single(jp.second_send_channel_id(), join_pattern_id);

                self.join_patterns.insert(join_pattern_id, BinarySend(jp));
            }
            BinaryRecv(jp) => {
                self.join_pattern_index
                    .insert_single(jp.send_channel_id(), join_pattern_id);
                self.join_pattern_index
                    .insert_single(jp.recv_channel_id(), join_pattern_id);

                self.join_patterns.insert(join_pattern_id, BinaryRecv(jp));
            }
            BinaryBidir(jp) => {
                self.join_pattern_index
                    .insert_single(jp.send_channel_id(), join_pattern_id);
                self.join_pattern_index
                    .insert_single(jp.bidir_channel_id(), join_pattern_id);

                self.join_patterns.insert(join_pattern_id, BinaryBidir(jp));
            }
            TernarySend(jp) => {
                self.join_pattern_index
                    .insert_single(jp.first_send_channel_id(), join_pattern_id);
                self.join_pattern_index
                    .insert_single(jp.second_send_channel_id(), join_pattern_id);
                self.join_pattern_index
                    .insert_single(jp.third_send_channel_id(), join_pattern_id);

                self.join_patterns.insert(join_pattern_id, TernarySend(jp));
            }
            TernaryRecv(jp) => {
                self.join_pattern_index
                    .insert_single(jp.first_send_channel_id(), join_pattern_id);
                self.join_pattern_index
                    .insert_single(jp.second_send_channel_id(), join_pattern_id);
                self.join_pattern_index
                    .insert_single(jp.recv_channel_id(), join_pattern_id);

                self.join_patterns.insert(join_pattern_id, TernaryRecv(jp));
            }
            TernaryBidir(jp) => {
                self.join_pattern_index
                    .insert_single(jp.first_send_channel_id(), join_pattern_id);
                self.join_pattern_index
                    .insert_single(jp.second_send_channel_id(), join_pattern_id);
                self.join_pattern_index
                    .insert_single(jp.bidir_channel_id(), join_pattern_id);

                self.join_patterns.insert(join_pattern_id, TernaryBidir(jp));
            }
        }
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
