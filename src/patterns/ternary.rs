use std::any::Any;
use std::sync::mpsc::Sender;
use std::thread;

use crate::{
    channels::{StrippedBidirChannel, StrippedRecvChannel, StrippedSendChannel},
    function_transforms,
    join_pattern::JoinPattern,
    types::{functions, ids, Message, Packet},
};

/****************************************
 * Three Send Join Pattern Construction *
 ****************************************/

/// Three `SendChannel` partial Join Pattern.
pub struct SendPartialPattern<T, U, V> {
    #[allow(dead_code)]
    junction_id: ids::JunctionId,
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    third_send_channel: StrippedSendChannel<V>,
    sender: Sender<Packet<SendJoinPattern>>,
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
        sender: Sender<Packet<SendJoinPattern>>,
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
            first_send_channel_id: self.first_send_channel.id(),
            second_send_channel_id: self.second_send_channel.id(),
            third_send_channel_id: self.third_send_channel.id(),
            f: function_transforms::ternary::transform_send(f),
        };

        join_pattern.add(self.sender);
    }
}

/// Three `SendChannel` full `JoinPattern`.
pub struct SendJoinPattern {
    first_send_channel_id: ids::ChannelId,
    second_send_channel_id: ids::ChannelId,
    third_send_channel_id: ids::ChannelId,
    f: functions::ternary::FnBox,
}

impl JoinPattern for SendJoinPattern {
    fn channels(&self) -> Vec<ids::ChannelId> {
        vec![
            self.first_send_channel_id,
            self.second_send_channel_id,
            self.third_send_channel_id,
        ]
    }

    /// Fire Join Pattern by running associated function in separate thread.
    fn fire(&self, messages: Vec<Message>) {
        let f_clone = self.f.clone();

        thread::spawn(move || {
            (*f_clone)(messages.remove(0), messages.remove(0), messages.remove(0));
        });
    }
}

/********************************************
 * Send & Receive Join Pattern Construction *
 ********************************************/

/// Two `SendChannel` & `RecvChannel` partial Join Pattern.
pub struct RecvPartialPattern<T, U, R> {
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    recv_channel: StrippedRecvChannel<R>,
    sender: Sender<Packet<RecvJoinPattern>>,
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
        sender: Sender<Packet<RecvJoinPattern>>,
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
            first_send_channel_id: self.first_send_channel.id(),
            second_send_channel_id: self.second_send_channel.id(),
            recv_channel_id: self.recv_channel.id(),
            f: function_transforms::ternary::transform_recv(f),
        };

        join_pattern.add(self.sender);
    }
}

/// Two `SendChannel` & `RecvChannel` full `JoinPattern`.
///
/// N.B.: While this struct appears to be a duplicate of `SendJoinPattern`
/// in terms of code, it is used to distinguish the capability of the
/// Join Pattern within the `Junction` through its type.
pub struct RecvJoinPattern {
    first_send_channel_id: ids::ChannelId,
    second_send_channel_id: ids::ChannelId,
    recv_channel_id: ids::ChannelId,
    f: functions::ternary::FnBox,
}

impl JoinPattern for RecvJoinPattern {
    fn channels(&self) -> Vec<ids::ChannelId> {
        vec![
            self.first_send_channel_id,
            self.second_send_channel_id,
            self.recv_channel_id,
        ]
    }

    /// Fire `JoinPattern` by running associated function in separate thread.
    fn fire(&self, messages: Vec<Message>) {
        let f_clone = self.f.clone();

        thread::spawn(move || {
            (*f_clone)(messages.remove(0), messages.remove(0), messages.remove(0));
        });
    }
}

/**************************************************
 * Send & Bidirectional Join Pattern Construction *
 **************************************************/

/// `SendChannel` & `BidirChannel` partial Join Pattern.
pub struct BidirPartialPattern<T, U, V, R> {
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    bidir_channel: StrippedBidirChannel<V, R>,
    sender: Sender<Packet<BidirJoinPattern>>,
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
        sender: Sender<Packet<BidirJoinPattern>>,
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
            first_send_channel_id: self.first_send_channel.id(),
            second_send_channel_id: self.second_send_channel.id(),
            bidir_channel_id: self.bidir_channel.id(),
            f: function_transforms::ternary::transform_bidir(f),
        };

        join_pattern.add(self.sender);
    }
}

/// Two `SendChannel` & `BidirChannel` full `JoinPattern`.
pub struct BidirJoinPattern {
    first_send_channel_id: ids::ChannelId,
    second_send_channel_id: ids::ChannelId,
    bidir_channel_id: ids::ChannelId,
    f: functions::ternary::FnBox,
}

impl JoinPattern for BidirJoinPattern {
    fn channels(&self) -> Vec<ids::ChannelId> {
        vec![
            self.first_send_channel_id,
            self.second_send_channel_id,
            self.bidir_channel_id,
        ]
    }

    /// Fire Join Pattern by running associated function in separate thread.
    fn fire(&self, messages: Vec<Message>) {
        let f_clone = self.f.clone();

        thread::spawn(move || {
            (*f_clone)(messages.remove(0), messages.remove(0), messages.remove(0));
        });
    }
}
