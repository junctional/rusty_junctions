use std::collections::LinkedList;

use crate::{
    controller::Controller,
    types::{
        ids::{ChannelId, JoinPatternId},
        JoinPattern,
    },
};

impl Controller {
    /// Return the `JoinPatternId`s of all alive `JoinPattern`s.
    ///
    /// A `JoinPattern` is considered alive if for each of the channels
    /// involved in it, there is at least one `Message` available.
    pub(in crate::controller) fn alive_join_patterns(
        &self,
        join_pattern_ids: &LinkedList<JoinPatternId>,
    ) -> Vec<JoinPatternId> {
        // We need clone the `JoinPatternId`s at this point to avoid
        // because need to avoid the issue of `peek_all` borrowing mutably,
        // but then needing to mutably borrow again later to update the
        // latest fired `JoinPatternId`.
        join_pattern_ids
            .iter()
            .filter(|&jp_id| self.is_alive(*jp_id))
            .cloned()
            .collect()
    }

    /// Return `true` if Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    fn is_alive(&self, join_pattern_id: JoinPatternId) -> bool {
        use JoinPattern::*;

        if let Some(join_pattern) = self.join_patterns.get(&join_pattern_id) {
            match join_pattern {
                UnarySend(jp) => self.is_unary_alive(jp.channel_id()),
                UnaryRecv(jp) => self.is_unary_alive(jp.channel_id()),
                UnaryBidir(jp) => self.is_unary_alive(jp.channel_id()),
                BinarySend(jp) => {
                    self.is_binary_alive(jp.first_send_channel_id(), jp.second_send_channel_id())
                }
                BinaryRecv(jp) => self.is_binary_alive(jp.send_channel_id(), jp.recv_channel_id()),
                BinaryBidir(jp) => {
                    self.is_binary_alive(jp.send_channel_id(), jp.bidir_channel_id())
                }
                TernarySend(jp) => self.is_ternary_alive(
                    jp.first_send_channel_id(),
                    jp.second_send_channel_id(),
                    jp.third_send_channel_id(),
                ),
                TernaryRecv(jp) => self.is_ternary_alive(
                    jp.first_send_channel_id(),
                    jp.second_send_channel_id(),
                    jp.recv_channel_id(),
                ),
                TernaryBidir(jp) => self.is_ternary_alive(
                    jp.first_send_channel_id(),
                    jp.second_send_channel_id(),
                    jp.bidir_channel_id(),
                ),
            }
        } else {
            false
        }
    }

    /// Return `true` if *unary* Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    fn is_unary_alive(&self, channel_id: ChannelId) -> bool {
        self.messages.count_items(&channel_id) >= 1
    }

    /// Return `true` if *binary* Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    ///
    /// For binary Join Patterns, we need to ensure that should both channels
    /// involved have the same `ChannelId`, we actually have at least two
    /// `Message`s available.
    fn is_binary_alive(&self, first_ch_id: ChannelId, second_ch_id: ChannelId) -> bool {
        if first_ch_id == second_ch_id {
            self.messages.count_items(&first_ch_id) >= 2
        } else {
            self.is_unary_alive(first_ch_id) && self.is_unary_alive(second_ch_id)
        }
    }

    /// Return `true` if *ternary* Join Pattern with given `JoinPatternId` is alive.
    ///
    /// A Join Pattern is considered alive if there is at least one `Message` for
    /// each of the channels involved in it.
    ///
    /// For ternary Join Patterns, we need to ensure that should more than one
    /// channel involved have the same `ChannelId`, we actually have enough
    /// `Message`s available.
    fn is_ternary_alive(
        &self,
        first_ch_id: ChannelId,
        second_ch_id: ChannelId,
        third_ch_id: ChannelId,
    ) -> bool {
        if first_ch_id == second_ch_id && second_ch_id == third_ch_id {
            self.messages.count_items(&first_ch_id) >= 3
        } else {
            self.is_binary_alive(first_ch_id, second_ch_id)
                && self.is_binary_alive(first_ch_id, third_ch_id)
                && self.is_binary_alive(second_ch_id, third_ch_id)
        }
    }
}
