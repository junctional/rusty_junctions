use std::any::Any;
use std::sync::mpsc::Sender;
use std::thread;

use crate::{
    channels::{StrippedBidirChannel, StrippedRecvChannel, StrippedSendChannel},
    function_transforms,
    types::{functions, ids, JoinPattern, Message, Packet},
};

/****************************************
 * Three Send Join Pattern Construction *
 ****************************************/

/// Three `SendChannel` partial Join Pattern.
pub struct SendPartialPattern<T, U, V> {
    junction_id: ids::JunctionId,
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
        let join_pattern = JoinPattern::TernarySend(SendJoinPattern::new(
            self.first_send_channel.id(),
            self.second_send_channel.id(),
            self.third_send_channel.id(),
            function_transforms::ternary::transform_send(f),
        ));

        self.sender
            .send(Packet::AddJoinPatternRequest { join_pattern })
            .unwrap();
    }
}

/// Three `SendChannel` full `JoinPattern`.
pub struct SendJoinPattern {
    first_send_channel_id: ids::ChannelId,
    second_send_channel_id: ids::ChannelId,
    third_send_channel_id: ids::ChannelId,
    f: functions::ternary::FnBox,
}

impl SendJoinPattern {
    pub(crate) fn new(
        first_send_channel_id: ids::ChannelId,
        second_send_channel_id: ids::ChannelId,
        third_send_channel_id: ids::ChannelId,
        f: functions::ternary::FnBox,
    ) -> SendJoinPattern {
        SendJoinPattern {
            first_send_channel_id,
            second_send_channel_id,
            third_send_channel_id,
            f,
        }
    }

    /// Return the ID of the first `SendChannel` in this Join Pattern.
    pub(crate) fn first_send_channel_id(&self) -> ids::ChannelId {
        self.first_send_channel_id
    }

    /// Return the ID of the second `SendChannel` in this Join Pattern.
    pub(crate) fn second_send_channel_id(&self) -> ids::ChannelId {
        self.second_send_channel_id
    }

    /// Return the ID of the third `SendChannel` in this Join Pattern.
    pub(crate) fn third_send_channel_id(&self) -> ids::ChannelId {
        self.third_send_channel_id
    }

    /// Fire Join Pattern by running associated function in separate thread.
    pub(crate) fn fire(&self, arg_1: Message, arg_2: Message, arg_3: Message) {
        let f_clone = self.f.clone();

        thread::spawn(move || {
            (*f_clone)(arg_1, arg_2, arg_3);
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
        let join_pattern = JoinPattern::TernaryRecv(RecvJoinPattern::new(
            self.first_send_channel.id(),
            self.second_send_channel.id(),
            self.recv_channel.id(),
            function_transforms::ternary::transform_recv(f),
        ));

        self.sender
            .send(Packet::AddJoinPatternRequest { join_pattern })
            .unwrap();
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

impl RecvJoinPattern {
    pub(crate) fn new(
        first_send_channel_id: ids::ChannelId,
        second_send_channel_id: ids::ChannelId,
        recv_channel_id: ids::ChannelId,
        f: functions::ternary::FnBox,
    ) -> RecvJoinPattern {
        RecvJoinPattern {
            first_send_channel_id,
            second_send_channel_id,
            recv_channel_id,
            f,
        }
    }

    /// Return the ID of first `SendChannel` in this `JoinPattern`.
    pub(crate) fn first_send_channel_id(&self) -> ids::ChannelId {
        self.first_send_channel_id
    }

    /// Return the ID of second `SendChannel` in this `JoinPattern`.
    pub(crate) fn second_send_channel_id(&self) -> ids::ChannelId {
        self.second_send_channel_id
    }

    /// Return the ID of the `RecvChannel` in this `JoinPattern`.
    pub(crate) fn recv_channel_id(&self) -> ids::ChannelId {
        self.recv_channel_id
    }

    /// Fire `JoinPattern` by running associated function in separate thread.
    pub(crate) fn fire(&self, arg_1: Message, arg_2: Message, return_sender: Message) {
        let f_clone = self.f.clone();

        thread::spawn(move || {
            (*f_clone)(arg_1, arg_2, return_sender);
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
        let join_pattern = JoinPattern::TernaryBidir(BidirJoinPattern::new(
            self.first_send_channel.id(),
            self.second_send_channel.id(),
            self.bidir_channel.id(),
            function_transforms::ternary::transform_bidir(f),
        ));

        self.sender
            .send(Packet::AddJoinPatternRequest { join_pattern })
            .unwrap();
    }
}

/// Two `SendChannel` & `BidirChannel` full `JoinPattern`.
pub struct BidirJoinPattern {
    first_send_channel_id: ids::ChannelId,
    second_send_channel_id: ids::ChannelId,
    bidir_channel_id: ids::ChannelId,
    f: functions::ternary::FnBox,
}

impl BidirJoinPattern {
    pub(crate) fn new(
        first_send_channel_id: ids::ChannelId,
        second_send_channel_id: ids::ChannelId,
        bidir_channel_id: ids::ChannelId,
        f: functions::ternary::FnBox,
    ) -> BidirJoinPattern {
        BidirJoinPattern {
            first_send_channel_id,
            second_send_channel_id,
            bidir_channel_id,
            f,
        }
    }

    /// Return the ID of first `SendChannel` in this Join Pattern.
    pub(crate) fn first_send_channel_id(&self) -> ids::ChannelId {
        self.first_send_channel_id
    }

    /// Return the ID of second `SendChannel` in this Join Pattern.
    pub(crate) fn second_send_channel_id(&self) -> ids::ChannelId {
        self.second_send_channel_id
    }

    /// Return the ID of the `BidirChannel` in this Join Pattern.
    pub(crate) fn bidir_channel_id(&self) -> ids::ChannelId {
        self.bidir_channel_id
    }

    /// Fire Join Pattern by running associated function in separate thread.
    pub(crate) fn fire(&self, arg_1: Message, arg_2: Message, arg_3_and_sender: Message) {
        let f_clone = self.f.clone();

        thread::spawn(move || {
            (*f_clone)(arg_1, arg_2, arg_3_and_sender);
        });
    }
}
