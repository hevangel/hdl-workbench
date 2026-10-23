//! Property-valued until operands (IEEE 1800 §16.12.12).

use super::*;

impl Simulator {
    pub(super) fn sva_advance_until(
        &mut self,
        left: &SvaNode,
        right: &SvaNode,
        env: &SvaEnv,
        ticks: &mut Vec<SvaUntilTick>,
        closed: &mut bool,
    ) -> Option<SvaOutcome> {
        if !*closed {
            ticks.push(SvaUntilTick {
                left: Self::sva_new_state(left, env),
                right: Self::sva_new_state(right, env),
                left_result: None,
                right_result: None,
            });
        }
        for tick in ticks.iter_mut() {
            if tick.right_result.is_none() {
                tick.right_result = self.sva_advance(right, &mut tick.right);
            }
            if tick.left_result.is_none() {
                tick.left_result = self.sva_advance(left, &mut tick.left);
            }
        }
        // A right-hand property beginning at tick k requires every left-hand
        // property beginning BEFORE k to succeed. Completion times may differ:
        // an operand still pending is an obligation, not a successful match.
        let mut prefix_pending = false;
        let mut possible = false;
        let mut nonvacuous = false;
        for tick in ticks.iter() {
            // §16.14.8(v/w): either operand may make an eligible tick
            // nonvacuous; two vacuous implications remain vacuous.
            nonvacuous |= matches!(tick.left_result, Some(SvaOutcome::Pass | SvaOutcome::Fail))
                || matches!(tick.right_result, Some(SvaOutcome::Pass | SvaOutcome::Fail))
                || (tick.left_result.is_none() && matches!(left, SvaNode::Seq(..)))
                || (tick.right_result.is_none() && matches!(right, SvaNode::Seq(..)));
            if tick.right_result != Some(SvaOutcome::Fail) {
                possible = true;
                if tick.right_result.is_some() && !prefix_pending {
                    return Some(if nonvacuous {
                        SvaOutcome::Pass
                    } else {
                        SvaOutcome::Vacuous
                    });
                }
            }
            match tick.left_result {
                Some(SvaOutcome::Fail) => {
                    *closed = true;
                    break;
                }
                None => prefix_pending = true,
                _ => {}
            }
        }
        if *closed && !possible {
            Some(SvaOutcome::Fail)
        } else {
            None
        }
    }

    pub(super) fn sva_until_fails_at_end(
        left: &SvaNode,
        right: &SvaNode,
        strong: bool,
        ticks: &[SvaUntilTick],
    ) -> bool {
        let mut prefix_holds = true;
        for tick in ticks {
            let right_holds = match tick.right_result {
                Some(result) => result != SvaOutcome::Fail,
                None => !Self::sva_fails_at_end(right, &tick.right),
            };
            if prefix_holds && right_holds {
                return false;
            }
            prefix_holds &= match tick.left_result {
                Some(result) => result != SvaOutcome::Fail,
                None => !Self::sva_fails_at_end(left, &tick.left),
            };
        }
        strong || !prefix_holds
    }
}
