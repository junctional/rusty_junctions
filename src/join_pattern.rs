use crate::types::{ids::ChannelId, Message};
use bag::Bag;
use itertools::Itertools;

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
