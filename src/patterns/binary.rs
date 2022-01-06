use std::any::Any;
use std::sync::mpsc::Sender;
use std::thread;

use crate::{
    channels::{
        BidirChannel, RecvChannel, SendChannel, StrippedBidirChannel, StrippedRecvChannel,
        StrippedSendChannel,
    },
    function_transforms,
    types::{functions, ids, JoinPattern, Message, Packet},
};

/*****************************************
 * Send & Send Join Pattern Construction *
 *****************************************/

/// Two `SendChannel` partial Join Pattern.
pub struct SendPartialPattern<T, U> {
    junction_id: ids::JunctionId,
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    sender: Sender<Packet>,
}

impl<T, U> SendPartialPattern<T, U>
where
    T: Any + Send,
    U: Any + Send,
{
    pub(crate) fn new(
        junction_id: ids::JunctionId,
        first_send_channel: StrippedSendChannel<T>,
        second_send_channel: StrippedSendChannel<U>,
        sender: Sender<Packet>,
    ) -> SendPartialPattern<T, U> {
        SendPartialPattern {
            junction_id,
            first_send_channel,
            second_send_channel,
            sender,
        }
    }

    /// Create a ternary partial `JoinPattern` with three send channels.
    ///
    /// Create a new ternary partial `JoinPattern` that starts with the current
    /// pattern and includes a new `SendChannel` after that.
    ///
    /// # Panics
    ///
    /// Panics if the supplied `SendChannel` does not carry the same
    /// `JunctionID` as this `SendPartialPattern`, i.e. has not been created by
    /// and is associated with the same `Junction`.
    pub fn and<V>(
        self,
        send_channel: &SendChannel<V>,
    ) -> super::ternary::SendPartialPattern<T, U, V>
    where
        V: Any + Send,
    {
        if send_channel.junction_id() == self.junction_id {
            super::ternary::SendPartialPattern::new(
                self.junction_id,
                self.first_send_channel,
                self.second_send_channel,
                send_channel.strip(),
                self.sender,
            )
        } else {
            panic!(
                "SendChannel and SendPartialPattern not associated \
                    with same Junction! Please use a SendChannel created \
                    using the same Junction as this partially complete Join \
                    Pattern"
            );
        }
    }

    /// Create a ternary partial `JoinPattern` with two send and receive channel.
    ///
    /// Create a new ternary partial `JoinPattern` that starts with the current
    /// pattern and includes a new `RecvChannel` after that.
    ///
    /// # Panics
    ///
    /// Panics if the supplied `RecvChannel` does not carry the same
    /// `JunctionID` as this `SendPartialPattern`, i.e. has not been created by
    /// and is associated with the same `Junction`.
    pub fn and_recv<R>(
        self,
        recv_channel: &RecvChannel<R>,
    ) -> super::ternary::RecvPartialPattern<T, U, R>
    where
        R: Any + Send,
    {
        if recv_channel.junction_id() == self.junction_id {
            super::ternary::RecvPartialPattern::new(
                self.first_send_channel,
                self.second_send_channel,
                recv_channel.strip(),
                self.sender,
            )
        } else {
            panic!(
                "RecvChannel and SendPartialPattern not associated \
                    with same Junction! Please use a RecvChannel created \
                    using the same Junction as this partially complete Join \
                    Pattern"
            );
        }
    }

    /// Create a ternary partial `JoinPattern` with two send and bidirectional channel.
    ///
    /// Create a new ternary partial Join Pattern that starts with the current
    /// pattern and includes a new `BidirChannel` after that.
    ///
    /// # Panics
    ///
    /// Panics if the supplied `BidirChannel` does not carry the same
    /// `JunctionID` as this `SendPartialPattern`, i.e. has not been created by
    /// and is associated with the same `Junction`.
    pub fn and_bidir<V, R>(
        self,
        bidir_channel: &BidirChannel<V, R>,
    ) -> super::ternary::BidirPartialPattern<T, U, V, R>
    where
        V: Any + Send,
        R: Any + Send,
    {
        if bidir_channel.junction_id() == self.junction_id {
            super::ternary::BidirPartialPattern::new(
                self.first_send_channel,
                self.second_send_channel,
                bidir_channel.strip(),
                self.sender,
            )
        } else {
            panic!(
                "BidirChannel and SendPartialPattern not associated \
                    with same Junction! Please use a BidirChannel created \
                    using the same Junction as this partially complete Join \
                    Pattern"
            );
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
        F: Fn(T, U) -> () + Send + Clone + 'static,
    {
        let join_pattern = JoinPattern::BinarySend(SendJoinPattern::new(
            self.first_send_channel.id(),
            self.second_send_channel.id(),
            function_transforms::binary::transform_send(f),
        ));

        self.sender
            .send(Packet::AddJoinPatternRequest { join_pattern })
            .unwrap();
    }
}

/// `SendChannel` & `SendChannel` full Join Pattern.
pub struct SendJoinPattern {
    first_send_channel_id: ids::ChannelId,
    second_send_channel_id: ids::ChannelId,
    f: functions::binary::FnBox,
}

impl SendJoinPattern {
    pub(crate) fn new(
        first_send_channel_id: ids::ChannelId,
        second_send_channel_id: ids::ChannelId,
        f: functions::binary::FnBox,
    ) -> SendJoinPattern {
        SendJoinPattern {
            first_send_channel_id,
            second_send_channel_id,
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

    /// Fire Join Pattern by running associated function in separate thread.
    pub(crate) fn fire(&self, arg_1: Message, arg_2: Message) {
        let f_clone = self.f.clone();

        thread::spawn(move || {
            (*f_clone)(arg_1, arg_2);
        });
    }
}

/********************************************
 * Send & Receive Join Pattern Construction *
 ********************************************/

/// `SendChannel` & `RecvChannel` partial Join Pattern.
pub struct RecvPartialPattern<T, R> {
    send_channel: StrippedSendChannel<T>,
    recv_channel: StrippedRecvChannel<R>,
    sender: Sender<Packet>,
}

impl<T, R> RecvPartialPattern<T, R>
where
    T: Any + Send,
    R: Any + Send,
{
    pub(crate) fn new(
        send_channel: StrippedSendChannel<T>,
        recv_channel: StrippedRecvChannel<R>,
        sender: Sender<Packet>,
    ) -> RecvPartialPattern<T, R> {
        RecvPartialPattern {
            send_channel,
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
        F: Fn(T) -> R + Send + Clone + 'static,
    {
        let join_pattern = JoinPattern::BinaryRecv(RecvJoinPattern::new(
            self.send_channel.id(),
            self.recv_channel.id(),
            function_transforms::binary::transform_recv(f),
        ));

        self.sender
            .send(Packet::AddJoinPatternRequest { join_pattern })
            .unwrap();
    }
}

/// `SendChannel` & `RecvChannel` full Join Pattern.
///
/// N.B.: While this struct appears to be a duplicate of `SendJoinPattern`
/// in terms of code, it is used to distinguish the capability of the
/// Join Pattern within the `Junction` through its type.
pub struct RecvJoinPattern {
    send_channel_id: ids::ChannelId,
    recv_channel_id: ids::ChannelId,
    f: functions::binary::FnBox,
}

impl RecvJoinPattern {
    pub(crate) fn new(
        send_channel_id: ids::ChannelId,
        recv_channel_id: ids::ChannelId,
        f: functions::binary::FnBox,
    ) -> RecvJoinPattern {
        RecvJoinPattern {
            send_channel_id,
            recv_channel_id,
            f,
        }
    }

    /// Return the ID of the `SendChannel` in this Join Pattern.
    pub(crate) fn send_channel_id(&self) -> ids::ChannelId {
        self.send_channel_id
    }

    /// Return the ID of the `RecvChannel` in this Join Pattern.
    pub(crate) fn recv_channel_id(&self) -> ids::ChannelId {
        self.recv_channel_id
    }

    /// Fire Join Pattern by running associated function in separate thread.
    pub(crate) fn fire(&self, msg: Message, return_sender: Message) {
        let f_clone = self.f.clone();

        thread::spawn(move || {
            (*f_clone)(msg, return_sender);
        });
    }
}

/**************************************************
 * Send & Bidirectional Join Pattern Construction *
 **************************************************/

/// `SendChannel` & `BidirChannel` partial Join Pattern.
pub struct BidirPartialPattern<T, U, R> {
    send_channel: StrippedSendChannel<T>,
    bidir_channel: StrippedBidirChannel<U, R>,
    sender: Sender<Packet>,
}

impl<T, U, R> BidirPartialPattern<T, U, R>
where
    T: Any + Send,
    U: Any + Send,
    R: Any + Send,
{
    pub(crate) fn new(
        send_channel: StrippedSendChannel<T>,
        bidir_channel: StrippedBidirChannel<U, R>,
        sender: Sender<Packet>,
    ) -> BidirPartialPattern<T, U, R> {
        BidirPartialPattern {
            send_channel,
            bidir_channel,
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
        let join_pattern = JoinPattern::BinaryBidir(BidirJoinPattern::new(
            self.send_channel.id(),
            self.bidir_channel.id(),
            function_transforms::binary::transform_bidir(f),
        ));

        self.sender
            .send(Packet::AddJoinPatternRequest { join_pattern })
            .unwrap();
    }
}

/// `SendChannel` & `BidirChannel` full Join Pattern.
pub struct BidirJoinPattern {
    send_channel_id: ids::ChannelId,
    bidir_channel_id: ids::ChannelId,
    f: functions::binary::FnBox,
}

impl BidirJoinPattern {
    pub(crate) fn new(
        send_channel_id: ids::ChannelId,
        bidir_channel_id: ids::ChannelId,
        f: functions::binary::FnBox,
    ) -> BidirJoinPattern {
        BidirJoinPattern {
            send_channel_id,
            bidir_channel_id,
            f,
        }
    }

    /// Return the ID of the `SendChannel` in this Join Pattern.
    pub(crate) fn send_channel_id(&self) -> ids::ChannelId {
        self.send_channel_id
    }

    /// Return the ID of the `BidirChannel` in this Join Pattern.
    pub(crate) fn bidir_channel_id(&self) -> ids::ChannelId {
        self.bidir_channel_id
    }

    /// Fire Join Pattern by running associated function in separate thread.
    pub(crate) fn fire(&self, arg_1: Message, arg_2_and_sender: Message) {
        let f_clone = self.f.clone();

        thread::spawn(move || {
            (*f_clone)(arg_1, arg_2_and_sender);
        });
    }
}
