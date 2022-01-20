use std::cmp::Ordering;

use crate::{
    controller::Controller,
    // join_pattern::JoinPattern,
    types::{ids::JoinPatternId, Message},
};

impl Controller {
    /// Select which `JoinPattern` should be fired.
    ///
    /// In order to avoid certain scenarious in which one `JoinPattern` would
    /// block the execution of another, because it for instance has a subset of
    /// the other's channels, we need to ensure that from the `JoinPattern`s
    /// that are alive simultaneously, we select the one to be fired that has
    /// been waiting for the longest time.
    ///
    /// Specifically, we record the `Counter` of the `Message` that each
    /// `JoinPattern` has last been fired at. We use this as a pseudo-time and
    /// simply order the `JoinPattern`s by their `Counter` values, then take
    /// the one with the smallest.
    ///
    /// Note that this procedure should ensure a certain form of *fairness*,
    /// by which if a `JoinPattern` has been alive an infinite amount of times,
    /// it will fire at least once. In practice, this should amount to each
    /// `JoinPattern` being incapable of getting deadlocked by others.
    pub(in crate::controller) fn select_to_fire<'a>(
        &self,
        alive_jp_ids: &'a mut Vec<JoinPatternId>,
    ) -> Option<&'a JoinPatternId> {
        alive_jp_ids
            .sort_unstable_by(|&jp_id_1, &jp_id_2| self.compare_last_fired(jp_id_1, jp_id_2));

        alive_jp_ids.first()
    }

    /// Compare when the Join Patterns with given `JoinPatternId`s were last alive at.
    ///
    /// Rules for Order:
    /// 1. If neither `JoinPatternId` has a last alive `Counter`, then neither
    /// has been fired yet, so they can be viewed as equal in this ordering.
    /// 2. If only one `JoinPatternId` has no last alive `Counter`, then that
    /// one has to be ordered as less than the other since having been fired
    /// at least once will always be a later point of firing than not having
    /// been fired yet.
    /// 3. If both `JoinPatternId`s have last alive `Counter`s, use the ordering
    /// of these.
    ///
    /// # Panics
    ///
    /// For simplicity, this function panics if the either of the given
    /// `JoinPatternId`s is not registered in the internal map of `JoinPatternId`s
    /// to `Instant`s that describe the last `Instant` at which a particular Join
    /// Pattern was alive. That is to say, this function should only be called on
    /// `JoinPatternId`s which are definitely stored in the calling `Controller`.
    pub(in crate::controller) fn compare_last_fired(
        &self,
        jp_id_1: JoinPatternId,
        jp_id_2: JoinPatternId,
    ) -> Ordering {
        // TODO: Can we sensibly use `Option::flatten` here?
        let last_fired_1 = self.join_pattern_last_fired.get(&jp_id_1).unwrap();
        let last_fired_2 = self.join_pattern_last_fired.get(&jp_id_2).unwrap();

        if last_fired_1.is_none() && last_fired_2.is_none() {
            Ordering::Equal
        } else if last_fired_1.is_none() && last_fired_2.is_some() {
            Ordering::Less
        } else if last_fired_1.is_some() && last_fired_2.is_none() {
            Ordering::Greater
        } else {
            last_fired_1.cmp(last_fired_2)
        }
    }

    /// Fire the `JoinPattern` corresponding to the given `JoinPatternId`.
    ///
    /// The processs of firing a `JoinPattern` consists of first retrieving
    /// a `Message` for each of the channels involved in the `JoinPattern`,
    /// then passing these `Messages`s to the `JoinPattern` to handle the
    /// firing.
    ///
    /// # Panics
    ///
    /// Panics when there is no `JoinPattern` stored for the given
    /// `JoinPatternId`.
    pub(in crate::controller) fn fire_join_pattern(&mut self, join_pattern_id: JoinPatternId) {
        let join_pattern = self.join_patterns.get(&join_pattern_id).unwrap();

        let mut messages_for_channels: Vec<Message> = Vec::new();
        for chan in join_pattern.channels() {
            let message = self.messages.retrieve(&chan).unwrap();
            messages_for_channels.push(message);
        }

        join_pattern.fire(messages_for_channels);
    }

    /// Reset the `Counter` at which the given Join Pattern has last been fired.
    pub(in crate::controller) fn reset_last_fired(&mut self, join_pattern_id: JoinPatternId) {
        self.join_pattern_last_fired
            .insert(join_pattern_id, Some(self.message_counter.clone()));
    }

    /// Initialize the `Instant` at which Join Pattern was last alive.
    pub(in crate::controller) fn initialize_last_fired(&mut self, join_pattern_id: JoinPatternId) {
        self.join_pattern_last_fired.insert(join_pattern_id, None);
    }
}
