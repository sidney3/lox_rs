use super::context::*;
use ndarray::Array2;

enum Action {
    Shift(StateId),       // shift and go to this state
    Reduce(ProductionId), // reduce to this production
    Accept,
    Error,
}

// In lr0, your actions are entirely a function of your state.
pub fn make_action(ctx: &Context, start: RuleId) -> Vec<Action> {
    let mut out = Vec::new();

    out
}
