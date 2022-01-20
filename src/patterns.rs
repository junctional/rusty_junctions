pub mod unary {
    use crate::channels::StrippedBidirChannel;
    use crate::channels::StrippedRecvChannel;
    use crate::channels::StrippedSendChannel;
    use crate::join_pattern::JoinPattern;

    #[derive(rusty_junctions_macro::PartialPattern, rusty_junctions_macro::JoinPattern)]
    /// `SendChannel` partial Join Pattern.
    pub struct SendPartialPattern<T> {
        junction_id: crate::types::ids::JunctionId,
        send_channel: crate::channels::StrippedSendChannel<T>,
        sender: std::sync::mpsc::Sender<crate::types::Packet>,
    }

    #[derive(rusty_junctions_macro::TerminalPartialPattern, rusty_junctions_macro::JoinPattern)]
    /// `RecvChannel` partial Join Pattern.
    pub struct RecvPartialPattern<R> {
        recv_channel: crate::channels::StrippedRecvChannel<R>,
        sender: std::sync::mpsc::Sender<crate::types::Packet>,
    }

    #[derive(rusty_junctions_macro::TerminalPartialPattern, rusty_junctions_macro::JoinPattern)]
    /// Bidirectional channel partial Join Pattern.
    pub struct BidirPartialPattern<T, R> {
        bidir_channel: crate::channels::StrippedBidirChannel<T, R>,
        sender: std::sync::mpsc::Sender<crate::types::Packet>,
    }
}

pub mod binary {
    use crate::channels::StrippedSendChannel;
    use crate::channels::StrippedBidirChannel;
    use crate::channels::StrippedRecvChannel;
    use crate::join_pattern::JoinPattern;

    #[derive(rusty_junctions_macro::PartialPattern, rusty_junctions_macro::JoinPattern)]
    /// Two `SendChannel` partial Join Pattern.
    pub struct SendPartialPattern<T, U> {
        junction_id: crate::types::ids::JunctionId,
        first_send_channel: crate::channels::StrippedSendChannel<T>,
        second_send_channel: crate::channels::StrippedSendChannel<U>,
        sender: std::sync::mpsc::Sender<crate::types::Packet>,
    }

    #[derive(rusty_junctions_macro::TerminalPartialPattern, rusty_junctions_macro::JoinPattern)]
    /// `SendChannel` & `RecvChannel` partial Join Pattern.
    pub struct RecvPartialPattern<T, R> {
        send_channel: crate::channels::StrippedSendChannel<T>,
        recv_channel: crate::channels::StrippedRecvChannel<R>,
        sender: std::sync::mpsc::Sender<crate::types::Packet>,
    }

    #[derive(rusty_junctions_macro::TerminalPartialPattern, rusty_junctions_macro::JoinPattern)]
    /// `SendChannel` & `BidirChannel` partial Join Pattern.
    pub struct BidirPartialPattern<T, U, R> {
        send_channel: crate::channels::StrippedSendChannel<T>,
        bidir_channel: crate::channels::StrippedBidirChannel<U, R>,
        sender: std::sync::mpsc::Sender<crate::types::Packet>,
    }
}

pub mod ternary {
    use crate::channels::StrippedBidirChannel;
    use crate::channels::StrippedRecvChannel;
    use crate::channels::StrippedSendChannel;
    use crate::join_pattern::JoinPattern;

    #[derive(rusty_junctions_macro::TerminalPartialPattern, rusty_junctions_macro::JoinPattern)]
    /// Three `SendChannel` partial Join Pattern.
    pub struct SendPartialPattern<T, U, V> {
        #[allow(dead_code)]
        junction_id: crate::types::ids::JunctionId,
        first_send_channel: crate::channels::StrippedSendChannel<T>,
        second_send_channel: crate::channels::StrippedSendChannel<U>,
        third_send_channel: crate::channels::StrippedSendChannel<V>,
        sender: std::sync::mpsc::Sender<crate::types::Packet>,
    }

    #[derive(rusty_junctions_macro::TerminalPartialPattern, rusty_junctions_macro::JoinPattern)]
    /// Two `SendChannel` & `RecvChannel` partial Join Pattern.
    pub struct RecvPartialPattern<T, U, R> {
        first_send_channel: crate::channels::StrippedSendChannel<T>,
        second_send_channel: crate::channels::StrippedSendChannel<U>,
        recv_channel: crate::channels::StrippedRecvChannel<R>,
        sender: std::sync::mpsc::Sender<crate::types::Packet>,
    }

    #[derive(rusty_junctions_macro::TerminalPartialPattern, rusty_junctions_macro::JoinPattern)]
    /// `SendChannel` & `BidirChannel` partial Join Pattern.
    pub struct BidirPartialPattern<T, U, V, R> {
        first_send_channel: crate::channels::StrippedSendChannel<T>,
        second_send_channel: crate::channels::StrippedSendChannel<U>,
        bidir_channel: crate::channels::StrippedBidirChannel<V, R>,
        sender: std::sync::mpsc::Sender<crate::types::Packet>,
    }
}
