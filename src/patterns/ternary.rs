use std::sync::mpsc::Sender;
use rusty_junctions_macro::{JoinPattern, TerminalPartialPattern};
use crate::{
    join_pattern::JoinPattern,
    channels::{StrippedBidirChannel, StrippedRecvChannel, StrippedSendChannel},
    types::{Packet, ids::JunctionId},
};

#[derive(TerminalPartialPattern, JoinPattern)]
/// Three `SendChannel` partial Join Pattern.
pub struct SendPartialPattern<T, U, V> {
    junction_id: JunctionId,
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    third_send_channel: StrippedSendChannel<V>,
    sender: Sender<Packet>,
}

#[derive(TerminalPartialPattern, JoinPattern)]
/// Two `SendChannel` & `RecvChannel` partial Join Pattern.
pub struct RecvPartialPattern<T, U, R> {
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    recv_channel: StrippedRecvChannel<R>,
    sender: Sender<Packet>,
}

#[derive(TerminalPartialPattern, JoinPattern)]
/// `SendChannel` & `BidirChannel` partial Join Pattern.
pub struct BidirPartialPattern<T, U, V, R> {
    first_send_channel: StrippedSendChannel<T>,
    second_send_channel: StrippedSendChannel<U>,
    bidir_channel: StrippedBidirChannel<V, R>,
    sender: Sender<Packet>,
}
