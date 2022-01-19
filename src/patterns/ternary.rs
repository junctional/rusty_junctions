use std::sync::mpsc::Sender;
use rusty_junctions_macro::{JoinPattern, TerminalPartialPattern};
use crate::{
    join_pattern::JoinPattern,
    channels::{StrippedBidirChannel, StrippedRecvChannel, StrippedSendChannel},
    types::{Packet, ids::JunctionId},
};

/****************************************
 * Three Send Join Pattern Construction *
 ****************************************/

#[derive(JoinPattern)]
/// Three `SendChannel` partial Join Pattern.
pub struct SendPartialPattern<T, U, V> {
    junction_id: JunctionId,
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    third_send_channel: StrippedSendChannel<V>,
    sender: Sender<Packet>,
}

impl<T, U, V> SendPartialPattern<T, U, V>
where
    T: Any + Send,
    U: Any + Send,
    V: Any + Send,
{
    pub(crate) fn new(
        junction_id: ids::JunctionId,
        first_send_channel: StrippedSendChannel<T>,
        second_send_channel: StrippedSendChannel<U>,
        third_send_channel: StrippedSendChannel<V>,
        sender: Sender<Packet>,
    ) -> SendPartialPattern<T, U, V> {
        SendPartialPattern {
            junction_id,
            first_send_channel,
            second_send_channel,
            third_send_channel,
            sender,
        }
    }

    /// Create full Join Pattern and send request to add it to `Junction`.
    ///
    /// Create a full Join Pattern by taking the channels that are part of
    /// the partial pattern and adding a function to be executed when there
    /// is at least one message sent on each channel. Attempt to add the
    /// Join Pattern to the `Junction` after creation.
    ///
    /// # Panics
    ///
    /// Panics if it was not possible to send the request to add the newly
    /// create Join Pattern to the `Junction`.
    pub fn then_do<F>(self, f: F)
    where
        F: Fn(T, U, V) -> () + Send + Clone + 'static,
    {
        let join_pattern = SendJoinPattern {
            first_send_channel: self.first_send_channel.id(),
            second_send_channel: self.second_send_channel.id(),
            third_send_channel: self.third_send_channel.id(),
            f: function_transforms::ternary::transform_send(f),
        };

        join_pattern.add(self.sender);
    }
}

/********************************************
 * Send & Receive Join Pattern Construction *
 ********************************************/

#[derive(JoinPattern)]
/// Two `SendChannel` & `RecvChannel` partial Join Pattern.
pub struct RecvPartialPattern<T, U, R> {
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    recv_channel: StrippedRecvChannel<R>,
    sender: Sender<Packet>,
}

impl<T, U, R> RecvPartialPattern<T, U, R>
where
    T: Any + Send,
    U: Any + Send,
    R: Any + Send,
{
    pub(crate) fn new(
        first_send_channel: StrippedSendChannel<T>,
        second_send_channel: StrippedSendChannel<U>,
        recv_channel: StrippedRecvChannel<R>,
        sender: Sender<Packet>,
    ) -> RecvPartialPattern<T, U, R> {
        RecvPartialPattern {
            first_send_channel,
            second_send_channel,
            recv_channel,
            sender,
        }
    }

    /// Create full Join Pattern and send request to add it to `Junction`.
    ///
    /// Create a full Join Pattern by taking the channels that are part of
    /// the partial pattern and adding a function to be executed when there
    /// is at least one message sent on each channel. Attempt to add the
    /// Join Pattern to the `Junction` after creation.
    ///
    /// # Panics
    ///
    /// Panics if it was not possible to send the request to add the newly
    /// create Join Pattern to the `Junction`.
    pub fn then_do<F>(self, f: F)
    where
        F: Fn(T, U) -> R + Send + Clone + 'static,
    {
        let join_pattern = RecvJoinPattern {
            first_send_channel: self.first_send_channel.id(),
            second_send_channel: self.second_send_channel.id(),
            recv_channel: self.recv_channel.id(),
            f: function_transforms::ternary::transform_recv(f),
        };

        join_pattern.add(self.sender);
    }
}

/**************************************************
 * Send & Bidirectional Join Pattern Construction *
 **************************************************/

#[derive(JoinPattern)]
/// `SendChannel` & `BidirChannel` partial Join Pattern.
pub struct BidirPartialPattern<T, U, V, R> {
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    bidir_channel: StrippedBidirChannel<V, R>,
    sender: Sender<Packet>,
}

impl<T, U, V, R> BidirPartialPattern<T, U, V, R>
where
    T: Any + Send,
    U: Any + Send,
    V: Any + Send,
    R: Any + Send,
{
    pub(crate) fn new(
        first_send_channel: StrippedSendChannel<T>,
        second_send_channel: StrippedSendChannel<U>,
        bidir_channel: StrippedBidirChannel<V, R>,
        sender: Sender<Packet>,
    ) -> BidirPartialPattern<T, U, V, R> {
        BidirPartialPattern {
            first_send_channel,
            second_send_channel,
            bidir_channel,
            sender,
        }
    }

    /// Create full `JoinPattern` and send request to add it to `Junction`.
    ///
    /// Create a full Join Pattern by taking the channels that are part of
    /// the partial pattern and adding a function to be executed when there
    /// is at least one message sent on each channel. Attempt to add the
    /// Join Pattern to the `Junction` after creation.
    ///
    /// # Panics
    ///
    /// Panics if it was not possible to send the request to add the newly
    /// create Join Pattern to the `Junction`.
    pub fn then_do<F>(self, f: F)
    where
        F: Fn(T, U, V) -> R + Send + Clone + 'static,
    {
        let join_pattern = BidirJoinPattern {
            first_send_channel: self.first_send_channel.id(),
            second_send_channel: self.second_send_channel.id(),
            bidir_channel: self.bidir_channel.id(),
            f: function_transforms::ternary::transform_bidir(f),
        };

        join_pattern.add(self.sender);
    }
}
