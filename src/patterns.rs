//! Structs to implement different types of `JoinPatterns`.

use std::any::Any;
use std::sync::mpsc::Sender;
use std::thread;

use super::channels::{
    BidirChannel, RecvChannel, SendChannel, StrippedBidirChannel, StrippedRecvChannel,
    StrippedSendChannel,
};
use super::function_transforms;
use super::types::{functions, ids, JoinPattern, Message, Packet};

/// Structs for Join Patterns with one channel.
pub mod unary {
    use super::*;

    /**********************************
     * Send Join Pattern Construction *
     **********************************/

    /// `SendChannel` partial Join Pattern.
    pub struct SendPartialPattern<T> {
        junction_id: ids::JunctionId,
        send_channel: StrippedSendChannel<T>,
        sender: Sender<Packet>,
    }

    impl<T> SendPartialPattern<T>
    where
        T: Any + Send,
    {
        pub(crate) fn new(
            junction_id: ids::JunctionId,
            send_channel: StrippedSendChannel<T>,
            sender: Sender<Packet>,
        ) -> SendPartialPattern<T> {
            SendPartialPattern {
                junction_id,
                send_channel,
                sender,
            }
        }

        /// Create a binary partial Join Pattern with two send channels.
        ///
        /// Create a new binary partial Join Pattern that starts with the current
        /// pattern and includes a new `SendChannel` after that.
        ///
        /// # Panics
        ///
        /// Panics if the supplied `SendChannel` does not carry the same
        /// `JunctionID` as this `SendPartialPattern`, i.e. has not been created by
        /// and is associated with the same `Junction`.
        pub fn and<U>(self, send_channel: &SendChannel<U>) -> binary::SendPartialPattern<T, U>
        where
            U: Any + Send,
        {
            if send_channel.junction_id() == self.junction_id {
                binary::SendPartialPattern::new(
                    self.junction_id,
                    self.send_channel,
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

        /// Create a binary partial Join Pattern with a send and receive channel.
        ///
        /// Create a new binary partial Join Pattern that starts with the current
        /// pattern and includes a new `RecvChannel` after that.
        ///
        /// # Panics
        ///
        /// Panics if the supplied `RecvChannel` does not carry the same
        /// `JunctionID` as this `SendPartialPattern`, i.e. has not been created by
        /// and is associated with the same `Junction`.
        pub fn and_recv<R>(self, recv_channel: &RecvChannel<R>) -> binary::RecvPartialPattern<T, R>
        where
            R: Any + Send,
        {
            if recv_channel.junction_id() == self.junction_id {
                binary::RecvPartialPattern::new(
                    self.send_channel,
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

        /// Create a binary partial Join Pattern with a send and bidirectional channel.
        ///
        /// Create a new binary partial Join Pattern that starts with the current
        /// pattern and includes a new `BidirChannel` after that.
        ///
        /// # Panics
        ///
        /// Panics if the supplied `BidirChannel` does not carry the same
        /// `JunctionID` as this `SendPartialPattern`, i.e. has not been created by
        /// and is associated with the same `Junction`.
        pub fn and_bidir<U, R>(
            self,
            bidir_channel: &BidirChannel<U, R>,
        ) -> binary::BidirPartialPattern<T, U, R>
        where
            U: Any + Send,
            R: Any + Send,
        {
            if bidir_channel.junction_id() == self.junction_id {
                binary::BidirPartialPattern::new(
                    self.send_channel,
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
            F: Fn(T) -> () + Send + Clone + 'static,
        {
            let join_pattern = JoinPattern::UnarySend(SendJoinPattern::new(
                self.send_channel.id(),
                function_transforms::unary::transform_send(f),
            ));

            self.sender
                .send(Packet::AddJoinPatternRequest { join_pattern })
                .unwrap();
        }
    }

    /// `SendChannel` full Join Pattern.
    pub struct SendJoinPattern {
        channel_id: ids::ChannelId,
        f: functions::unary::FnBox,
    }

    impl SendJoinPattern {
        pub(crate) fn new(
            channel_id: ids::ChannelId,
            f: functions::unary::FnBox,
        ) -> SendJoinPattern {
            SendJoinPattern { channel_id, f }
        }

        /// Return the ID of the channel in this Join Pattern.
        pub(crate) fn channel_id(&self) -> ids::ChannelId {
            self.channel_id
        }

        /// Fire Join Pattern by running associated function in separate thread.
        pub(crate) fn fire(&self, arg: Message) {
            let f_clone = self.f.clone();

            thread::spawn(move || {
                (*f_clone)(arg);
            });
        }
    }

    /*************************************
     * Receive Join Pattern Construction *
     *************************************/

    /// `RecvChannel` partial Join Pattern.
    pub struct RecvPartialPattern<R> {
        recv_channel: StrippedRecvChannel<R>,
        sender: Sender<Packet>,
    }

    impl<R> RecvPartialPattern<R>
    where
        R: Any + Send,
    {
        pub(crate) fn new(
            recv_channel: StrippedRecvChannel<R>,
            sender: Sender<Packet>,
        ) -> RecvPartialPattern<R> {
            RecvPartialPattern {
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
            F: Fn() -> R + Send + Clone + 'static,
        {
            let join_pattern = JoinPattern::UnaryRecv(RecvJoinPattern::new(
                self.recv_channel.id(),
                function_transforms::unary::transform_recv(f),
            ));

            self.sender
                .send(Packet::AddJoinPatternRequest { join_pattern })
                .unwrap();
        }
    }

    /// `RecvChannel` full Join Pattern.
    ///
    /// N.B.: While this struct appears to be a duplicate of `SendJoinPattern`
    /// in terms of code, it is used to distinguish the capability of the
    /// Join Pattern within the `Junction` through its type.
    pub struct RecvJoinPattern {
        channel_id: ids::ChannelId,
        f: functions::unary::FnBox,
    }

    impl RecvJoinPattern {
        pub(crate) fn new(
            channel_id: ids::ChannelId,
            f: functions::unary::FnBox,
        ) -> RecvJoinPattern {
            RecvJoinPattern { channel_id, f }
        }

        /// Return the ID of the channel in this Join Pattern.
        pub(crate) fn channel_id(&self) -> ids::ChannelId {
            self.channel_id
        }

        /// Fire Join Pattern by running associated function in separate thread.
        pub(crate) fn fire(&self, return_sender: Message) {
            let f_clone = self.f.clone();

            thread::spawn(move || {
                (*f_clone)(return_sender);
            });
        }
    }

    /*******************************************
     * Bidirectional Join Pattern Construction *
     *******************************************/

    /// Bidirectional channel partial Join Pattern.
    pub struct BidirPartialPattern<T, R> {
        bidir_channel: StrippedBidirChannel<T, R>,
        sender: Sender<Packet>,
    }

    impl<T, R> BidirPartialPattern<T, R>
    where
        T: Any + Send,
        R: Any + Send,
    {
        pub(crate) fn new(
            bidir_channel: StrippedBidirChannel<T, R>,
            sender: Sender<Packet>,
        ) -> BidirPartialPattern<T, R> {
            BidirPartialPattern {
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
            F: Fn(T) -> R + Send + Clone + 'static,
        {
            let join_pattern = JoinPattern::UnaryBidir(BidirJoinPattern::new(
                self.bidir_channel.id(),
                function_transforms::unary::transform_bidir(f),
            ));

            self.sender
                .send(Packet::AddJoinPatternRequest { join_pattern })
                .unwrap();
        }
    }

    /// `BidirChannel` full Join Pattern.
    pub struct BidirJoinPattern {
        channel_id: ids::ChannelId,
        f: functions::unary::FnBox,
    }

    impl BidirJoinPattern {
        pub(crate) fn new(
            channel_id: ids::ChannelId,
            f: functions::unary::FnBox,
        ) -> BidirJoinPattern {
            BidirJoinPattern { channel_id, f }
        }

        /// Return the ID of the channel in this Join Pattern.
        pub(crate) fn channel_id(&self) -> ids::ChannelId {
            self.channel_id
        }

        /// Fire Join Pattern by running associated function in separate thread.
        pub(crate) fn fire(&self, arg_and_sender: Message) {
            let f_clone = self.f.clone();

            thread::spawn(move || {
                (*f_clone)(arg_and_sender);
            });
        }
    }
}

/// Structs for Join Patterns with two channels.
pub mod binary {
    use super::*;

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
        pub fn and<V>(self, send_channel: &SendChannel<V>) -> ternary::SendPartialPattern<T, U, V>
        where
            V: Any + Send,
        {
            if send_channel.junction_id() == self.junction_id {
                ternary::SendPartialPattern::new(
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
        ) -> ternary::RecvPartialPattern<T, U, R>
        where
            R: Any + Send,
        {
            if recv_channel.junction_id() == self.junction_id {
                ternary::RecvPartialPattern::new(
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
        ) -> ternary::BidirPartialPattern<T, U, V, R>
        where
            V: Any + Send,
            R: Any + Send,
        {
            if bidir_channel.junction_id() == self.junction_id {
                ternary::BidirPartialPattern::new(
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
}

/// Structs for Join Patterns with two channels.
pub mod ternary {
    use super::*;

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
}
