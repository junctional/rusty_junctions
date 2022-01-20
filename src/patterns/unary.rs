use std::sync::mpsc::Sender;
use rusty_junctions_macro::{JoinPattern, PartialPattern, TerminalPartialPattern};
use crate::{
    channels::{
        StrippedBidirChannel, StrippedRecvChannel,
        StrippedSendChannel,
    },
    join_pattern::JoinPattern,
    types::{ids::JunctionId, Packet},
};

#[derive(PartialPattern, JoinPattern)]
/// `SendChannel` partial Join Pattern.
pub struct SendPartialPattern<T> {
    junction_id: JunctionId,
    send_channel: StrippedSendChannel<T>,
    sender: Sender<Packet>,
}

#[derive(TerminalPartialPattern, JoinPattern)]
/// `RecvChannel` partial Join Pattern.
pub struct RecvPartialPattern<R> {
    recv_channel: StrippedRecvChannel<R>,
    sender: Sender<Packet>,
}

#[derive(TerminalPartialPattern, JoinPattern)]
/// Bidirectional channel partial Join Pattern.
pub struct BidirPartialPattern<T, R> {
    bidir_channel: StrippedBidirChannel<T, R>,
    sender: Sender<Packet>,
}
