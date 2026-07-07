//! Rollback-as-a-service (TASK 049).
//!
//! A micro-project's execution is a cascade of steps, each with a concrete
//! **inverse**. If execution is rejected or fails, the inverse actions are applied
//! in **reverse order** (undo most-recent first). Pure domain logic — the
//! application layer decides when to roll back and the presentation narrates it.

/// One cascade step and the action that undoes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CascadeStep {
    /// The forward action.
    pub forward: String,
    /// The action that reverses [`CascadeStep::forward`].
    pub inverse: String,
}

impl CascadeStep {
    /// Construct a step from its forward and inverse actions.
    pub fn new(forward: impl Into<String>, inverse: impl Into<String>) -> Self {
        Self {
            forward: forward.into(),
            inverse: inverse.into(),
        }
    }
}

/// An ordered rollback plan built as the cascade proceeds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RollbackPlan {
    /// Steps in the order they were applied.
    pub steps: Vec<CascadeStep>,
}

impl RollbackPlan {
    /// An empty plan.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a step as it is applied.
    pub fn record(&mut self, step: CascadeStep) {
        self.steps.push(step);
    }

    /// Whether the plan has no steps.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// The number of recorded steps.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// The inverse actions in reverse order (undo the most recent step first).
    pub fn inverse_order(&self) -> Vec<String> {
        self.steps.iter().rev().map(|s| s.inverse.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverse_order_reverses_the_cascade() {
        let mut plan = RollbackPlan::new();
        plan.record(CascadeStep::new("apply A", "revert A"));
        plan.record(CascadeStep::new("apply B", "revert B"));
        plan.record(CascadeStep::new("apply C", "revert C"));
        assert_eq!(
            plan.inverse_order(),
            vec![
                "revert C".to_string(),
                "revert B".to_string(),
                "revert A".to_string()
            ]
        );
    }

    #[test]
    fn empty_plan_has_no_inverses() {
        let plan = RollbackPlan::new();
        assert!(plan.is_empty());
        assert_eq!(plan.len(), 0);
        assert!(plan.inverse_order().is_empty());
    }
}
