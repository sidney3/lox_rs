use super::closure::Closure;
use super::context::*;
use either::Either;
use itertools::Itertools;
use ndarray::Array2;
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};

// goto[state_id, symbol_id] -> state_id
pub fn make_goto(mut ctx: &mut Context, start: RuleId) -> Array2<Option<StateId>> {
    let mut closure = Closure::new();

    let start_state = State {
        in_progress: closure.compute(
            ctx,
            &ctx.rule_definition(start)
                .iter()
                .map(|production_id| Item {
                    production: *production_id,
                    position: 0,
                })
                .collect(),
        ),
        done: SmallVec::new(),
    };

    let start_state_id = ctx.states.intern(start_state);

    let mut worklist: Vec<StateId> = vec![start_state_id];
    let mut visited: HashSet<StateId> = HashSet::new();
    let mut wip_goto: HashMap<(StateId, Symbol), StateId> = HashMap::new();

    while let Some(state_id) = worklist.pop() {
        visited.insert(state_id);
        let edge_symbols: SymbolVec = ctx
            .states
            .get_left(state_id)
            .in_progress
            .iter()
            .map(|item| ctx.item_head(*item))
            .unique()
            .collect();

        for symbol_id in edge_symbols {
            let next_state_id = goto(&mut ctx, &mut closure, state_id, symbol_id);

            wip_goto.insert((state_id, symbol_id), next_state_id);

            if !visited.contains(&next_state_id) {
                worklist.push(next_state_id);
            }
        }
    }

    let num_states = ctx.states.len();
    let num_symbols = ctx.num_symbols();

    let mut res = Array2::from_elem((num_states, num_symbols), None::<StateId>);

    for ((src_state_id, symbol), dst_state_id) in wip_goto {
        let symbol_id = ctx.symbol_id(symbol);
        res[[usize::from(src_state_id), usize::from(symbol_id)]] = Option::Some(dst_state_id);
    }

    res
}

fn goto(ctx: &mut Context, closure: &mut Closure, state: StateId, edge: Symbol) -> StateId {
    let (finished_symbols, next_items): (RuleVec, ItemVec) = ctx
        .states
        .get_left(state)
        .in_progress
        .iter()
        .filter(|i| ctx.item_head(**i) == edge)
        .partition_map(|item| {
            let definition = ctx.production_definition(item.production);
            assert!(item.position <= definition.len());

            if definition.len() == item.position {
                Either::Left(ctx.production_rule(item.production))
            } else {
                Either::Right(Item {
                    production: item.production,
                    position: item.position + 1,
                })
            }
        });

    ctx.states.intern(State {
        in_progress: closure.compute(ctx, &next_items),
        done: finished_symbols,
    })
}
