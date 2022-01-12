use std::collections::LinkedList;

use crate::{controller::Controller, types::ids::JoinPatternId};

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
        self.join_patterns
            .get(&join_pattern_id)
            .map_or(false, |jp| jp.is_alive(&self.messages))
    }
}
