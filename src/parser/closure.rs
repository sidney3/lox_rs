use super::context::{Context, Item, ItemVec, ProductionId};
use std::collections::HashSet;

pub struct Closure {
    active_productions: HashSet<ProductionId>,
    worklist: Vec<ProductionId>,
}

impl Closure {
    pub fn new() -> Self {
        Self {
            active_productions: HashSet::new(),
            worklist: Vec::new(),
        }
    }

    pub fn compute(&mut self, ctx: &Context, items: &ItemVec) -> ItemVec {
        self.active_productions.clear();
        self.active_productions
            .extend(items.iter().map(|i| i.production));
        self.worklist.clear();
        self.worklist.extend(
            items
                .iter()
                .flat_map(|i| ctx.symbol_production(ctx.item_head(*i))),
        );

        let mut curr_items = ItemVec::new();

        while let Some(production_id) = self.worklist.pop() {
            curr_items.push(Item {
                production: production_id,
                position: 0,
            });
            self.worklist.extend(
                ctx.productions
                    .get_left(production_id)
                    .first()
                    .iter()
                    .flat_map(|symbol| ctx.symbol_production(**symbol))
                    .copied(),
            );
        }

        curr_items
    }
}
