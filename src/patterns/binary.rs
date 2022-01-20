use std::sync::mpsc::Sender;
use rusty_junctions_macro::{JoinPattern, PartialPattern, TerminalPartialPattern};
use crate::{
    channels::{StrippedBidirChannel, StrippedRecvChannel, StrippedSendChannel},
    join_pattern::JoinPattern,
    types::{ids::JunctionId, Packet},
};

#[derive(PartialPattern, JoinPattern)]
/// Two `SendChannel` partial Join Pattern.
pub struct SendPartialPattern<T, U> {
    junction_id: JunctionId,
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    sender: Sender<Packet>,
}

#[derive(TerminalPartialPattern, JoinPattern)]
/// `SendChannel` & `RecvChannel` partial Join Pattern.
pub struct RecvPartialPattern<T, R> {
    send_channel: StrippedSendChannel<T>,
    recv_channel: StrippedRecvChannel<R>,
    sender: Sender<Packet>,
}

#[derive(TerminalPartialPattern, JoinPattern)]
/// `SendChannel` & `BidirChannel` partial Join Pattern.
pub struct BidirPartialPattern<T, U, R> {
    send_channel: StrippedSendChannel<T>,
    bidir_channel: StrippedBidirChannel<U, R>,
    sender: Sender<Packet>,
}
