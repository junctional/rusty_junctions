use std::any::Any;
use std::sync::mpsc::Sender;
use std::thread;

use rusty_junctions_macro::JoinPattern;

use crate::{
    channels::{
        BidirChannel, RecvChannel, SendChannel, StrippedBidirChannel, StrippedRecvChannel,
        StrippedSendChannel,
    },
    function_transforms,
    join_pattern::JoinPattern,
    types::{ids, Message, Packet},
};

/*****************************************
 * Send & Send Join Pattern Construction *
 *****************************************/

#[derive(JoinPattern)]
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
        let join_pattern = SendJoinPattern {
            first_send_channel: self.first_send_channel.id(),
            second_send_channel: self.second_send_channel.id(),
            f: function_transforms::binary::transform_send(f),
        };

        join_pattern.add(self.sender);
    }
}

/********************************************
 * Send & Receive Join Pattern Construction *
 ********************************************/

#[derive(JoinPattern)]
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
        let join_pattern = RecvJoinPattern {
            send_channel: self.send_channel.id(),
            recv_channel: self.recv_channel.id(),
            f: function_transforms::binary::transform_recv(f),
        };

        join_pattern.add(self.sender);
    }
}

/**************************************************
 * Send & Bidirectional Join Pattern Construction *
 **************************************************/

#[derive(JoinPattern)]
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
        let join_pattern = BidirJoinPattern {
            send_channel: self.send_channel.id(),
            bidir_channel: self.bidir_channel.id(),
            f: function_transforms::binary::transform_bidir(f),
        };

        join_pattern.add(self.sender);
    }
}
