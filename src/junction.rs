//! Overarching structure to group `JoinPattern`s and their associated channels
//! together. Main structure for the public interface, used to create new
//! channels and construct `JoinPattern`s based on them.

use std::{
    any::Any,
    ops::Drop,
    sync::mpsc::{channel, RecvError, Sender},
};

use crate::{
    channels::{BidirChannel, RecvChannel, SendChannel},
    controller::{Controller, ControllerHandle},
    // join_pattern::JoinPattern,
    patterns::unary::{BidirPartialPattern, RecvPartialPattern, SendPartialPattern},
    types::{ids, Packet},
};

/// Struct managing the creation of new channels and Join Patterns.
///
/// This struct is used to group channels, such as `SendChannel`, which can
/// be used in conjunction to create new Join Patterns. As such, it offers
/// methods to create new channels which are then directly linked to this
/// `Junction`. It also offers methods to start off the creation of new
/// Join Patterns that rely on the channels created by this struct and can,
/// in fact, only consist of channels associated with this struct.
pub struct Junction {
    id: ids::JunctionId,
    controller_handle: Option<ControllerHandle>,
    sender: Sender<Packet>,
}

#[allow(clippy::new_without_default)]
impl Junction {
    /// Create a new `Junction` and start control thread in background.
    ///
    /// Create a new `Junction` and spawn a control thread in the background
    /// that will handle all incoming `Packet`s for this `Junction`. A
    /// `JoinHandle` to this control thread is stored alongside the `Junction`.
    pub fn new() -> Junction {
        let (sender, receiver) = channel::<Packet>();

        let controller = Controller::new();

        Junction {
            id: ids::JunctionId::new(),
            controller_handle: Some(controller.start(sender.clone(), receiver)),
            sender,
        }
    }

    /// Return handle to internal `Controller` if available.
    ///
    /// Each `Junction` has an associated control thread with a `Controller`
    /// running to handle incoming `Packet`s. This `Controller` is gracefully
    /// stopped and its thread joined as soon as the `Junction` goes out of
    /// scope. However, as this is sometimes undesired behavior, the user can
    /// retrieve the handle to the `Junction`'s `Controller` and its thread
    /// with this function and stop the `Controller` at a time of their
    /// choosing. Once this function has executed, the `Junction` will no
    /// long automatically stop its `Controller` and join the control thread
    /// upon going out of scope.
    ///
    /// Note that this handle can only be retrieved once.
    pub fn controller_handle(&mut self) -> Option<ControllerHandle> {
        self.controller_handle.take()
    }

    /// Create and return a new `SendChannel` on this `Junction`.
    ///
    /// The generic parameter `T` is used to determine the type of values
    /// that can be sent on this channel.
    ///
    /// # Panics
    ///
    /// Panics if it received an error while trying to receive a new
    /// channel ID from the control thread.
    pub fn send_channel<T>(&self) -> SendChannel<T>
    where
        T: Any + Send,
    {
        SendChannel::new(self.new_channel_id().unwrap(), self.id, self.sender.clone())
    }

    /// Create and return a new `RecvChannel` on this `Junction`.
    ///
    /// The generic parameter `R` is used to determine the type of values
    /// that can be received on this channel.
    ///
    /// # Panics
    ///
    /// Panics if it received an error while trying to receive a new
    /// channel ID from the control thread.
    pub fn recv_channel<R>(&self) -> RecvChannel<R>
    where
        R: Any + Send,
    {
        RecvChannel::new(self.new_channel_id().unwrap(), self.id, self.sender.clone())
    }

    /// Create and return a new `BidirChannel` on this `Junction`.
    ///
    /// The generic parameter `T` is used to determine the type of values
    /// that can be sent on this channel while `R` is used to determine
    /// the type of values that can be received on this channel.
    ///
    /// # Panics
    ///
    /// Panics if it received an error while trying to receive the new
    /// channel IDs from the control thread.
    pub fn bidir_channel<T, R>(&self) -> BidirChannel<T, R>
    where
        T: Any + Send,
        R: Any + Send,
    {
        BidirChannel::new(self.new_channel_id().unwrap(), self.id, self.sender.clone())
    }

    /// Request ID for a new channel from control thread.
    ///
    /// # Panics
    ///
    /// Panics if request for new channel id could not be sent to
    /// control thread.
    fn new_channel_id(&self) -> Result<ids::ChannelId, RecvError> {
        let (id_sender, id_receiver) = channel::<ids::ChannelId>();

        self.sender
            .send(Packet::NewChannelIdRequest {
                return_sender: id_sender,
            })
            .map_err(|e| log::error!("Failed to send NewChannelIdRequest: {e:?}"))
            .unwrap();

        id_receiver.recv()
    }
}

impl Junction {
    /// Create new partial Join Pattern starting with a `SendChannel`.
    ///
    /// # Panics
    ///
    /// Panics if the supplied `SendChannel` does not carry the same
    /// `JunctionID` as this `Junction`, i.e. has not been created by and is
    /// associated with this `Junction`.
    pub fn when<T>(&self, send_channel: &SendChannel<T>) -> SendPartialPattern<T>
    where
        T: Any + Send,
    {
        if send_channel.junction_id() == self.id {
            SendPartialPattern::new(self.id, send_channel.strip(), self.sender.clone())
        } else {
            panic!(
                "SendChannel is not associated with Junction! Please use \
                 a SendChannel created using the same Junction calling \
                 this function!"
            );
        }
    }

    /// Create new partial Join Pattern starting with a `RecvChannel`.
    ///
    /// # Panics
    ///
    /// Panics if the supplied `RecvChannel` does not carry the same
    /// `JunctionID` as this `Junction`, i.e. has not been created by and is
    /// associated with this `Junction`.
    pub fn when_recv<R>(&self, recv_channel: &RecvChannel<R>) -> RecvPartialPattern<R>
    where
        R: Any + Send,
    {
        if recv_channel.junction_id() == self.id {
            RecvPartialPattern::new(recv_channel.strip(), self.sender.clone())
        } else {
            panic!(
                "RecvChannel is not associated with Junction! Please use \
                 a RecvChannel created using the same Junction calling \
                 this function!"
            );
        }
    }

    /// Create a new partial Join Pattern starting with a `BidirChannel`.
    ///
    /// # Panics
    ///
    /// Panics if the supplied `BidirChannel` does not carry the same
    /// `JunctionID` as this `Junction`, i.e. has not been created by and is
    /// associated with this `Junction`.
    pub fn when_bidir<T, R>(&self, bidir_channel: &BidirChannel<T, R>) -> BidirPartialPattern<T, R>
    where
        T: Any + Send,
        R: Any + Send,
    {
        if bidir_channel.junction_id() == self.id {
            BidirPartialPattern::new(bidir_channel.strip(), self.sender.clone())
        } else {
            panic!(
                "BidirChannel is not associated with Junction! Please use \
                 a BidirChannel created using the same Junction calling \
                 this function!"
            );
        }
    }
}

impl Drop for Junction {
    /// Drop the `Junction` and free its resources.
    ///
    /// If there is a `ControllerHandle` still available, use it to stop the
    /// associated `Controller` and join the control thread. Otherwise, no
    /// action is needed.
    fn drop(&mut self) {
        log::debug!("Dropping Junction - Attempting to shutdown Controller");
        if self.controller_handle.is_some() {
            log::debug!("Controller has a ControllerHandle");
            self.controller_handle.take().unwrap().stop();
        } else {
            log::debug!("Controller didn't have a ControllerHandle");
        }
    }
}
