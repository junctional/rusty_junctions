use crate::types::{ids::ChannelId, Message, Packet};
use bag::Bag;
use itertools::Itertools;
use std::{marker::Sized, sync::mpsc::Sender};

/// Enum defining all Join Patterns that can be added to a Junction using the
/// `AddJoinPatternRequest` in a `Packet`.
// pub struct JoinPattern<T: crate::join_pattern::JoinPattern>(T);
// pub enum JoinPattern {
//     /// Single channel Join Pattern.
//     UnarySend(crate::patterns::unary::SendJoinPattern),
//     /// Single `RecvChannel` Join Pattern.
//     UnaryRecv(crate::patterns::unary::RecvJoinPattern),
//     /// Single `BidirChannel` Join Pattern.
//     UnaryBidir(crate::patterns::unary::BidirJoinPattern),
//     /// Two `SendChannel` Join Pattern.
//     BinarySend(crate::patterns::binary::SendJoinPattern),
//     /// `SendChannel` and `RecvChannel` Join Pattern.
//     BinaryRecv(crate::patterns::binary::RecvJoinPattern),
//     /// `SendChannel` and `BidirChannel` Join Pattern.
//     BinaryBidir(crate::patterns::binary::BidirJoinPattern),
//     /// Three `SendChannel` Join Pattern.
//     TernarySend(crate::patterns::ternary::SendJoinPattern),
//     /// Two `SendChannel` and `RecvChannel` Join Pattern.
//     TernaryRecv(crate::patterns::ternary::RecvJoinPattern),
//     /// Two `SendChannel` and `BidirChannel` Join Pattern.
//     TernaryBidir(crate::patterns::ternary::BidirJoinPattern),
// }

// impl JoinPattern {
pub trait JoinPattern: Sized {
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

    fn add(self, sender: Sender<Packet<Self>>) {
        sender
            .send(Packet::AddJoinPatternRequest { join_pattern: self })
            .unwrap();
    }

    /// Return a `Vec<ChannelId` for each of the channels in the Join Pattern
    fn channels(&self) -> Vec<ChannelId>;

    /// Given the `Message` for each of the channels in the pattern - fire.
    fn fire(&self, messages: Vec<Message>);
}

// use crate::types::ids::ChannelId;

// pub trait JoinPattern {}

// pub struct SendJoinPattern {
//     send_channels: Vec<ChannelId>,
// }

// pub struct RecvJoinPattern {
//     send_channels: Vec<ChannelId>,
//     recv_channel: ChannelId,
// }

// pub struct BidirJoinPattern {
//     send_channels: Vec<ChannelId>,
//     bidir_channel: ChannelId,
// }
